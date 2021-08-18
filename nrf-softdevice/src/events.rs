use core::convert::TryFrom;
use core::mem::MaybeUninit;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::pac::interrupt;
use crate::raw;
use crate::util::Signal;
use crate::RawError;

static SWI2_SIGNAL: Signal<()> = Signal::new();

#[rustfmt::skip]
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum SocEvent {
    Hfclkstarted = raw::NRF_SOC_EVTS_NRF_EVT_HFCLKSTARTED,
    PowerFailureWarning = raw::NRF_SOC_EVTS_NRF_EVT_POWER_FAILURE_WARNING,
    FlashOperationSuccess = raw::NRF_SOC_EVTS_NRF_EVT_FLASH_OPERATION_SUCCESS,
    FlashOperationError = raw::NRF_SOC_EVTS_NRF_EVT_FLASH_OPERATION_ERROR,
    RadioBlocked = raw::NRF_SOC_EVTS_NRF_EVT_RADIO_BLOCKED,
    RadioCanceled = raw::NRF_SOC_EVTS_NRF_EVT_RADIO_CANCELED,
    RadioSignalCallbackInvalidReturn = raw::NRF_SOC_EVTS_NRF_EVT_RADIO_SIGNAL_CALLBACK_INVALID_RETURN,
    RadioSessionIdle = raw::NRF_SOC_EVTS_NRF_EVT_RADIO_SESSION_IDLE,
    RadioSessionClosed = raw::NRF_SOC_EVTS_NRF_EVT_RADIO_SESSION_CLOSED,
    #[cfg(any(feature="s113", feature="s122", feature="s140"))]
    PowerUsbPowerReady = raw::NRF_SOC_EVTS_NRF_EVT_POWER_USB_POWER_READY,
    #[cfg(any(feature="s113", feature="s122", feature="s140"))]
    PowerUsbDetected = raw::NRF_SOC_EVTS_NRF_EVT_POWER_USB_DETECTED,
    #[cfg(any(feature="s113", feature="s122", feature="s140"))]
    PowerUsbRemoved = raw::NRF_SOC_EVTS_NRF_EVT_POWER_USB_REMOVED,
}

fn on_soc_evt(evt: u32) {
    let evt = match SocEvent::try_from(evt) {
        Ok(evt) => evt,
        Err(_) => panic!("Unknown soc evt {:?}", evt),
    };

    info!("soc evt {:?}", evt);
    match evt {
        SocEvent::FlashOperationError => crate::flash::on_flash_error(),
        SocEvent::FlashOperationSuccess => crate::flash::on_flash_success(),
        _ => {}
    }
}

// TODO actually derive this from the headers + the ATT_MTU
const BLE_EVT_MAX_SIZE: u16 = 128;

pub(crate) async fn run() {
    loop {
        SWI2_SIGNAL.wait().await;

        unsafe {
            let mut evt: u32 = 0;
            loop {
                match RawError::convert(raw::sd_evt_get(&mut evt as _)) {
                    Ok(()) => on_soc_evt(evt),
                    Err(RawError::NotFound) => break,
                    Err(err) => panic!("sd_evt_get err {:?}", err),
                }
            }

            // Using u32 since the buffer has to be aligned to 4
            let mut evt: MaybeUninit<[u32; BLE_EVT_MAX_SIZE as usize / 4]> = MaybeUninit::uninit();

            loop {
                let mut len: u16 = BLE_EVT_MAX_SIZE;
                let ret = raw::sd_ble_evt_get(evt.as_mut_ptr() as *mut u8, &mut len as _);
                match RawError::convert(ret) {
                    Ok(()) => crate::ble::on_evt(evt.as_ptr() as *const raw::ble_evt_t),
                    Err(RawError::NotFound) => break,
                    Err(RawError::BleNotEnabled) => break,
                    Err(RawError::NoMem) => panic!("BUG: BLE_EVT_MAX_SIZE is too low"),
                    Err(err) => panic!("sd_ble_evt_get err {:?}", err),
                }
            }
        }
    }
}

#[cfg(any(feature = "nrf52805", feature = "nrf52810", feature = "nrf52811"))]
#[interrupt]
unsafe fn SWI2() {
    SWI2_SIGNAL.signal(());
}

#[cfg(not(any(feature = "nrf52805", feature = "nrf52810", feature = "nrf52811")))]
#[interrupt]
unsafe fn SWI2_EGU2() {
    SWI2_SIGNAL.signal(());
}
