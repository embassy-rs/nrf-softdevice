use core::marker::PhantomData;
use core::mem::MaybeUninit;
use defmt::info;
use nrf52840_pac::{interrupt, Interrupt};
use nrf_softdevice_s140 as sd;

use crate::util::Signal;

unsafe extern "C" fn fault_handler(id: u32, pc: u32, info: u32) {
    depanic!("fault_handler {:u32} {:u32} {:u32}", id, pc, info);
}

/// safety: call at most once
pub unsafe fn enable() {
    // TODO make this configurable via features or param
    let clock_cfg = sd::nrf_clock_lf_cfg_t {
        source: sd::NRF_CLOCK_LF_SRC_XTAL as u8,
        rc_ctiv: 0,
        rc_temp_ctiv: 0,
        accuracy: 7,
    };

    let ret = sd::sd_softdevice_enable(&clock_cfg as _, Some(fault_handler));
    if ret != sd::NRF_SUCCESS {
        depanic!("sd_softdevice_enable ret {:u32}", ret);
    }

    crate::interrupt::unmask(Interrupt::SWI2_EGU2);
}

static SWI2_SIGNAL: Signal<()> = Signal::new();

#[derive(defmt::Format)]
enum SocEvent {
    Hfclkstarted,
    PowerFailureWarning,
    FlashOperationSuccess,
    FlashOperationError,
    RadioBlocked,
    RadioCanceled,
    RadioSignalCallbackinvalidReturn,
    RadioSessionIdle,
    RadioSessionClosed,
    PowerUsbPowerReady,
    PowerUsbDetected,
    PowerUsbRemoved,
}

impl SocEvent {
    fn from_raw(raw: u32) -> Self {
        match raw {
            sd::NRF_SOC_EVTS_NRF_EVT_HFCLKSTARTED => SocEvent::Hfclkstarted,
            sd::NRF_SOC_EVTS_NRF_EVT_POWER_FAILURE_WARNING => SocEvent::PowerFailureWarning,
            sd::NRF_SOC_EVTS_NRF_EVT_FLASH_OPERATION_SUCCESS => SocEvent::FlashOperationSuccess,
            sd::NRF_SOC_EVTS_NRF_EVT_FLASH_OPERATION_ERROR => SocEvent::FlashOperationError,
            sd::NRF_SOC_EVTS_NRF_EVT_RADIO_BLOCKED => SocEvent::RadioBlocked,
            sd::NRF_SOC_EVTS_NRF_EVT_RADIO_CANCELED => SocEvent::RadioCanceled,
            sd::NRF_SOC_EVTS_NRF_EVT_RADIO_SIGNAL_CALLBACK_INVALID_RETURN => {
                SocEvent::RadioSignalCallbackinvalidReturn
            }
            sd::NRF_SOC_EVTS_NRF_EVT_RADIO_SESSION_IDLE => SocEvent::RadioSessionIdle,
            sd::NRF_SOC_EVTS_NRF_EVT_RADIO_SESSION_CLOSED => SocEvent::RadioSessionClosed,
            sd::NRF_SOC_EVTS_NRF_EVT_POWER_USB_POWER_READY => SocEvent::PowerUsbPowerReady,
            sd::NRF_SOC_EVTS_NRF_EVT_POWER_USB_DETECTED => SocEvent::PowerUsbDetected,
            sd::NRF_SOC_EVTS_NRF_EVT_POWER_USB_REMOVED => SocEvent::PowerUsbRemoved,
            x => depanic!("unknown soc evt {:u32}", x),
        }
    }
}

#[derive(defmt::Format)]
enum BleEvent<'a> {
    ToDo(PhantomData<&'a ()>),
}

impl<'a> BleEvent<'a> {
    fn from_raw(ble_evt: &'a sd::ble_evt_t, len: usize) -> Self {
        Self::ToDo(PhantomData)
    }
}

fn on_soc_evt(evt: SocEvent) {
    info!("soc evt {:?}", evt);
    match evt {
        SocEvent::FlashOperationError => crate::flash::on_flash_error(),
        SocEvent::FlashOperationSuccess => crate::flash::on_flash_success(),
        _ => {}
    }
}

fn on_ble_evt(evt: BleEvent<'_>) {
    info!("got ble evt");
}

// TODO actually derive this from the headers + the ATT_MTU
const BLE_EVT_MAX_SIZE: u16 = 128;

pub async fn run() {
    loop {
        SWI2_SIGNAL.wait().await;

        unsafe {
            let mut evt: u32 = 0;
            loop {
                match sd::sd_evt_get(&mut evt as _) {
                    sd::NRF_SUCCESS => on_soc_evt(SocEvent::from_raw(evt)),
                    sd::NRF_ERROR_NOT_FOUND => break,
                    err => depanic!("sd_evt_get returned {:u32}", err),
                }
            }

            // Using u32 since the buffer has to be aligned to 4
            let mut evt: MaybeUninit<[u32; BLE_EVT_MAX_SIZE as usize / 4]> = MaybeUninit::uninit();

            loop {
                let mut len: u16 = BLE_EVT_MAX_SIZE;
                match sd::sd_ble_evt_get(evt.as_mut_ptr() as *mut u8, &mut len as _) {
                    sd::NRF_SUCCESS => {
                        let evt_ref = &*(evt.as_ptr() as *const sd::ble_evt_t);
                        on_ble_evt(BleEvent::from_raw(evt_ref, len as usize));
                    }
                    sd::NRF_ERROR_NO_MEM => depanic!("BUG: BLE_EVT_MAX_SIZE is too low"),
                    sd::NRF_ERROR_NOT_FOUND => break,
                    sd::BLE_ERROR_NOT_ENABLED => break,
                    err => depanic!("sd_ble_evt_get returned {:u32}", err),
                }
            }
        }
    }
}

#[cortex_m_rt::interrupt]
unsafe fn SWI2_EGU2() {
    SWI2_SIGNAL.signal(());
}
