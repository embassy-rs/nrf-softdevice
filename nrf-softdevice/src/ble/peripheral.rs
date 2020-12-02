//! Bluetooth Peripheral operations. Peripheral devices emit advertisements, and optionally accept connections from Central devices.

use core::mem;
use core::ptr;

use crate::ble::*;
use crate::raw;
use crate::util::{assert, *};
use crate::{RawError, Softdevice};

pub(crate) unsafe fn on_adv_set_terminated(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    trace!(
        "peripheral on_adv_set_terminated conn_handle={:u16}",
        gap_evt.conn_handle
    );
    ADV_PORTAL.call(Err(AdvertiseError::Stopped))
}

pub(crate) unsafe fn on_scan_req_report(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    trace!(
        "peripheral on_scan_req_report conn_handle={:u16}",
        gap_evt.conn_handle
    );
}

pub(crate) unsafe fn on_sec_info_request(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    trace!(
        "peripheral on_sec_info_request conn_handle={:u16}",
        gap_evt.conn_handle
    );
}

/// Connectable advertisement types, which can accept connections from interested Central devices.
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

/// Non-Connectable advertisement types. They cannot accept connections, they can be
/// only used to broadcast information in the air.
pub enum NonconnectableAdvertisement {
    ScannableUndirected,
    NonscannableUndirected,
    ExtendedScannableUndirected,
    ExtendedScannableDirected,
    ExtendedNonscannableUndirected,
    ExtendedNonscannableDirected,
}

/// Error for [`advertise_start`]
#[derive(defmt::Format)]
pub enum AdvertiseError {
    Stopped,
    Raw(RawError),
}

impl From<RawError> for AdvertiseError {
    fn from(err: RawError) -> Self {
        AdvertiseError::Raw(err)
    }
}

static mut ADV_HANDLE: u8 = raw::BLE_GAP_ADV_SET_HANDLE_NOT_SET as u8;
pub(crate) static ADV_PORTAL: Portal<Result<Connection, AdvertiseError>> = Portal::new();

// Begins an ATT MTU exchange procedure, followed by a data length update request as necessary.
pub async fn advertise(
    sd: &Softdevice,
    adv: ConnectableAdvertisement<'_>,
    config: Config,
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

    let d = OnDrop::new(|| {
        let ret = unsafe { raw::sd_ble_gap_adv_stop(ADV_HANDLE) };
        if let Err(e) = RawError::convert(ret) {
            warn!("sd_ble_gap_adv_stop: {:?}", e);
        }
    });

    let ret = unsafe {
        raw::sd_ble_gap_adv_set_configure(&mut ADV_HANDLE as _, &datas as _, &adv_params as _)
    };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gap_adv_set_configure err {:?}", err);
        err
    })?;

    let ret = unsafe { raw::sd_ble_gap_adv_start(ADV_HANDLE, 1 as u8) };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gap_adv_start err {:?}", err);
        err
    })?;

    info!("Advertising started!");

    // The advertising data needs to be kept alive for the entire duration of the advertising procedure.

    let conn = ADV_PORTAL.wait_once(|res| res).await?;

    d.defuse();

    Ok(conn)
}

#[derive(Copy, Clone)]
pub struct Config {
    // bits of BLE_GAP_PHY_
    tx_phys: u8,
    // bits of BLE_GAP_PHY_
    rx_phys: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tx_phys: raw::BLE_GAP_PHY_AUTO as _,
            rx_phys: raw::BLE_GAP_PHY_AUTO as _,
        }
    }
}
