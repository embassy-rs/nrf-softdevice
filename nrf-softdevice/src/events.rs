use core::mem::MaybeUninit;
use core::task::Poll;

use embassy_sync::waitqueue::AtomicWaker;
use futures::future::poll_fn;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{raw, RawError};

static SWI2_SOC_EVT_WAKER: AtomicWaker = AtomicWaker::new();
static SWI2_BLE_EVT_WAKER: AtomicWaker = AtomicWaker::new();

/// SoC events reported by the softdevice.
#[rustfmt::skip]
#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SocEvent {
    Hfclkstarted = raw::NRF_SOC_EVTS_NRF_EVT_HFCLKSTARTED,
    PowerFailureWarning = raw::NRF_SOC_EVTS_NRF_EVT_POWER_FAILURE_WARNING,
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

fn on_soc_evt<F: FnMut(SocEvent)>(evt: u32, evt_handler: &mut F) {
    trace!("soc evt {:?}", evt);

    match evt {
        raw::NRF_SOC_EVTS_NRF_EVT_FLASH_OPERATION_ERROR => crate::flash::on_flash_error(),
        raw::NRF_SOC_EVTS_NRF_EVT_FLASH_OPERATION_SUCCESS => crate::flash::on_flash_success(),
        _ => {
            let evt = match SocEvent::try_from(evt) {
                Ok(evt) => evt,
                Err(_) => panic!("Unknown soc evt {:?}", evt),
            };

            evt_handler(evt)
        }
    }
}

// Doing this without features would require Softdevice to have its configuration available as
// consts (through associated constants), then we'd have a const generic run function that
// allocates a precalculated size.
#[cfg(feature = "evt-max-size-512")]
const BLE_EVT_MAX_SIZE: u16 = 512;
#[cfg(all(feature = "evt-max-size-256", not(feature = "evt-max-size-512")))]
const BLE_EVT_MAX_SIZE: u16 = 256;
#[cfg(not(any(feature = "evt-max-size-256", feature = "evt-max-size-512")))]
const BLE_EVT_MAX_SIZE: u16 = 128;

pub(crate) async fn run_soc<F: FnMut(SocEvent)>(mut soc_evt_handler: F) -> ! {
    poll_fn(|cx| unsafe {
        SWI2_SOC_EVT_WAKER.register(cx.waker());

        let mut evt: u32 = 0;
        loop {
            match RawError::convert(raw::sd_evt_get(&mut evt as _)) {
                Ok(()) => on_soc_evt(evt, &mut soc_evt_handler),
                Err(RawError::NotFound) => break,
                Err(err) => panic!("sd_evt_get err {:?}", err),
            }
        }

        Poll::Pending
    })
    .await
}

pub(crate) async fn run_ble() -> ! {
    poll_fn(|cx| unsafe {
        SWI2_BLE_EVT_WAKER.register(cx.waker());
        // Using u32 since the buffer has to be aligned to 4
        let mut evt: MaybeUninit<[u32; BLE_EVT_MAX_SIZE as usize / 4]> = MaybeUninit::uninit();

        loop {
            let mut len: u16 = BLE_EVT_MAX_SIZE;
            let ret = raw::sd_ble_evt_get(evt.as_mut_ptr() as *mut u8, &mut len as _);
            match RawError::convert(ret) {
                Ok(()) => crate::ble::on_evt(evt.as_ptr() as *const raw::ble_evt_t),
                Err(RawError::NotFound) => break,
                Err(RawError::BleNotEnabled) => break,
                Err(RawError::DataSize) => panic!("BLE_EVT_MAX_SIZE is too low, use larger evt-max-size feature"),
                Err(err) => panic!("sd_ble_evt_get err {:?}", err),
            }
        }

        Poll::Pending
    })
    .await
}

#[cfg_attr(
    any(feature = "nrf52805", feature = "nrf52810", feature = "nrf52811"),
    export_name = "SWI2"
)]
#[cfg_attr(
    not(any(feature = "nrf52805", feature = "nrf52810", feature = "nrf52811")),
    export_name = "EGU2_SWI2"
)]
unsafe extern "C" fn swi2_irq_handler() {
    SWI2_SOC_EVT_WAKER.wake();
    SWI2_BLE_EVT_WAKER.wake();
}

/// `nrf528xx_pac` and early versions of `nrf_pac` name the SWI2 interrupt `SWI2_EGU2` instead of `EGU2_SWI2`
#[cfg(not(any(feature = "nrf52805", feature = "nrf52810", feature = "nrf52811")))]
#[allow(dead_code)]
#[export_name = "SWI2_EGU2"]
unsafe extern "C" fn old_swi2_irq_handler() {
    SWI2_SOC_EVT_WAKER.wake();
    SWI2_BLE_EVT_WAKER.wake();
}
