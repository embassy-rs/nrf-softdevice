use core::mem;
use core::ptr;

use crate::ble::*;
use crate::raw;
use crate::util::*;
use crate::{Error, Softdevice};

pub(crate) unsafe fn on_adv_set_terminated(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
    ADV_SIGNAL.signal(Err(AdvertiseError::Stopped))
}

pub(crate) unsafe fn on_scan_req_report(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_sec_info_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub enum ConnectableAdvertisement<'a> {
    ScannableUndirected {
        adv_data: &'a [u8],
        scan_data: &'a [u8],
    },
    NonscannableDirected {
        scan_data: &'a [u8],
    },
    NonscannableDirectedHighDuty {
        scan_data: &'a [u8],
    },
    ExtendedNonscannableUndirected {
        adv_data: &'a [u8],
    },
    ExtendedNonscannableDirected {
        adv_data: &'a [u8],
    },
}

enum NonconnectableAdvertisement {
    ScannableUndirected,
    NonscannableUndirected,
    ExtendedScannableUndirected,
    ExtendedScannableDirected,
    ExtendedNonscannableUndirected,
    ExtendedNonscannableDirected,
}

#[derive(defmt::Format)]
pub enum AdvertiseError {
    Stopped,
    Raw(Error),
}

impl From<Error> for AdvertiseError {
    fn from(err: Error) -> Self {
        AdvertiseError::Raw(err)
    }
}

#[derive(defmt::Format)]
pub enum AdvertiseStopError {
    NotRunning,
    Raw(Error),
}

impl From<Error> for AdvertiseStopError {
    fn from(err: Error) -> Self {
        AdvertiseStopError::Raw(err)
    }
}

static mut ADV_HANDLE: u8 = raw::BLE_GAP_ADV_SET_HANDLE_NOT_SET as u8;
pub(crate) static ADV_SIGNAL: Signal<Result<Connection, AdvertiseError>> = Signal::new();

pub async fn advertise(
    sd: &Softdevice,
    adv: ConnectableAdvertisement<'_>,
) -> Result<Connection, AdvertiseError> {
    // TODO make these configurable, only the right params based on type?
    let mut adv_params: raw::ble_gap_adv_params_t = unsafe { mem::zeroed() };
    adv_params.properties.type_ = raw::BLE_GAP_ADV_TYPE_CONNECTABLE_SCANNABLE_UNDIRECTED as u8;
    adv_params.primary_phy = raw::BLE_GAP_PHY_1MBPS as u8;
    adv_params.secondary_phy = raw::BLE_GAP_PHY_1MBPS as u8;
    adv_params.duration = raw::BLE_GAP_ADV_TIMEOUT_GENERAL_UNLIMITED as u16;
    adv_params.interval = 100;

    let (adv_data, scan_data) = match adv {
        ConnectableAdvertisement::ScannableUndirected {
            adv_data,
            scan_data,
        } => (Some(adv_data), Some(scan_data)),
        ConnectableAdvertisement::NonscannableDirected { scan_data } => (None, Some(scan_data)),
        ConnectableAdvertisement::NonscannableDirectedHighDuty { scan_data } => {
            (None, Some(scan_data))
        }
        ConnectableAdvertisement::ExtendedNonscannableUndirected { adv_data } => {
            (Some(adv_data), None)
        }
        ConnectableAdvertisement::ExtendedNonscannableDirected { adv_data } => {
            (Some(adv_data), None)
        }
    };

    let map_data = |data: Option<&[u8]>| {
        if let Some(data) = data {
            assert!(data.len() < u16::MAX as usize);
            raw::ble_data_t {
                p_data: data.as_ptr() as _,
                len: data.len() as u16,
            }
        } else {
            raw::ble_data_t {
                p_data: ptr::null_mut(),
                len: 0,
            }
        }
    };

    let datas = raw::ble_gap_adv_data_t {
        adv_data: map_data(adv_data),
        scan_rsp_data: map_data(scan_data),
    };

    let ret = unsafe {
        raw::sd_ble_gap_adv_set_configure(&mut ADV_HANDLE as _, &datas as _, &adv_params as _)
    };
    Error::convert(ret).dewarn(intern!("sd_ble_gap_adv_set_configure"))?;

    let ret = unsafe { raw::sd_ble_gap_adv_start(ADV_HANDLE, 1 as u8) };
    Error::convert(ret).dewarn(intern!("sd_ble_gap_adv_start"))?;

    // TODO handle future drop

    info!("Advertising started!");

    // The advertising data needs to be kept alive for the entire duration of the advertising procedure.

    ADV_SIGNAL.wait().await
}

pub fn advertise_stop(sd: &Softdevice) -> Result<(), AdvertiseStopError> {
    let ret = unsafe { raw::sd_ble_gap_adv_stop(ADV_HANDLE) };
    match Error::convert(ret).dewarn(intern!("sd_ble_gap_adv_stop")) {
        Ok(()) => Ok(()),
        Err(Error::InvalidState) => Err(AdvertiseStopError::NotRunning),
        Err(e) => Err(e.into()),
    }
}
