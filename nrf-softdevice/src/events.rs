use core::convert::TryFrom;
use core::mem::MaybeUninit;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::error::Error;
use crate::util::*;
use crate::{interrupt, sd};

static SWI2_SIGNAL: Signal<()> = Signal::new();

#[rustfmt::skip]
#[repr(u32)]
#[derive(defmt::Format, IntoPrimitive, TryFromPrimitive)]
enum SocEvent {
    Hfclkstarted = sd::NRF_SOC_EVTS_NRF_EVT_HFCLKSTARTED,
    PowerFailureWarning = sd::NRF_SOC_EVTS_NRF_EVT_POWER_FAILURE_WARNING,
    FlashOperationSuccess = sd::NRF_SOC_EVTS_NRF_EVT_FLASH_OPERATION_SUCCESS,
    FlashOperationError = sd::NRF_SOC_EVTS_NRF_EVT_FLASH_OPERATION_ERROR,
    RadioBlocked = sd::NRF_SOC_EVTS_NRF_EVT_RADIO_BLOCKED,
    RadioCanceled = sd::NRF_SOC_EVTS_NRF_EVT_RADIO_CANCELED,
    RadioSignalCallbackInvalidReturn = sd::NRF_SOC_EVTS_NRF_EVT_RADIO_SIGNAL_CALLBACK_INVALID_RETURN,
    RadioSessionIdle = sd::NRF_SOC_EVTS_NRF_EVT_RADIO_SESSION_IDLE,
    RadioSessionClosed = sd::NRF_SOC_EVTS_NRF_EVT_RADIO_SESSION_CLOSED,
    PowerUsbPowerReady = sd::NRF_SOC_EVTS_NRF_EVT_POWER_USB_POWER_READY,
    PowerUsbDetected = sd::NRF_SOC_EVTS_NRF_EVT_POWER_USB_DETECTED,
    PowerUsbRemoved = sd::NRF_SOC_EVTS_NRF_EVT_POWER_USB_REMOVED,
}

fn on_soc_evt(evt: u32) {
    let evt = match SocEvent::try_from(evt) {
        Ok(evt) => evt,
        Err(_) => depanic!("Unknown soc evt {:u32}", evt),
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

pub async fn run() {
    loop {
        SWI2_SIGNAL.wait().await;

        unsafe {
            let mut evt: u32 = 0;
            loop {
                match Error::convert(sd::sd_evt_get(&mut evt as _)) {
                    Ok(()) => on_soc_evt(evt),
                    Err(Error::NotFound) => break,
                    Err(err) => depanic!("sd_evt_get err {:?}", err),
                }
            }

            // Using u32 since the buffer has to be aligned to 4
            let mut evt: MaybeUninit<[u32; BLE_EVT_MAX_SIZE as usize / 4]> = MaybeUninit::uninit();

            loop {
                let mut len: u16 = BLE_EVT_MAX_SIZE;
                let ret = sd::sd_ble_evt_get(evt.as_mut_ptr() as *mut u8, &mut len as _);
                match Error::convert(ret) {
                    Ok(()) => crate::ble::on_ble_evt(&*(evt.as_ptr() as *const sd::ble_evt_t)),
                    Err(Error::NotFound) => break,
                    Err(Error::BleNotEnabled) => break,
                    Err(Error::NoMem) => depanic!("BUG: BLE_EVT_MAX_SIZE is too low"),
                    Err(err) => depanic!("sd_ble_evt_get err {:?}", err),
                }
            }
        }
    }
}

#[interrupt]
unsafe fn SWI2_EGU2() {
    SWI2_SIGNAL.signal(());
}
