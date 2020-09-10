use core::mem;
use core::ptr;

use crate::ble::types::*;
use crate::error::Error;
use crate::raw;
use crate::util::*;
use crate::{Connection, ConnectionState, Role};

pub(crate) unsafe fn on_connected(_ble_evt: *const raw::ble_evt_t, gap_evt: &raw::ble_gap_evt_t) {
    let params = &gap_evt.params.connected;
    let conn_handle = gap_evt.conn_handle;
    let role = Role::from_raw(params.role);

    // TODO what to do if new fails because no free con indexes?
    let conn = Connection::new(conn_handle).dewrap();
    let state = conn.state();

    state.role.set(role);

    match role {
        Role::Central => CONNECT_SIGNAL.signal(Ok(conn)),
        Role::Peripheral => ADV_SIGNAL.signal(Ok(conn)),
    }
}

pub(crate) unsafe fn on_disconnected(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    let conn_handle = gap_evt.conn_handle;
    let state = ConnectionState::by_conn_handle(conn_handle);
    state.on_disconnected()
}

pub(crate) unsafe fn on_conn_param_update(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_sec_params_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_sec_info_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_passkey_display(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_key_pressed(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}
pub(crate) unsafe fn on_auth_key_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_lesc_dhkey_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_auth_status(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_conn_sec_update(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_timeout(_ble_evt: *const raw::ble_evt_t, gap_evt: &raw::ble_gap_evt_t) {
    let params = &gap_evt.params.timeout;
    match params.src as u32 {
        raw::BLE_GAP_TIMEOUT_SRC_CONN => CONNECT_SIGNAL.signal(Err(ConnectError::Stopped)),
        x => warn!("unknown timeout src {:u32}", x),
    }
}

pub(crate) unsafe fn on_rssi_changed(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_adv_report(_ble_evt: *const raw::ble_evt_t, _gap_evt: &raw::ble_gap_evt_t) {
}

pub(crate) unsafe fn on_sec_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_conn_param_update_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_scan_req_report(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_phy_update_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_phy_update(_ble_evt: *const raw::ble_evt_t, _gap_evt: &raw::ble_gap_evt_t) {
}

pub(crate) unsafe fn on_data_length_update_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_data_length_update(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_qos_channel_survey_report(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_adv_set_terminated(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
    ADV_SIGNAL.signal(Err(AdvertiseError::Stopped))
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

static mut ADV_HANDLE: u8 = raw::BLE_GAP_ADV_SET_HANDLE_NOT_SET as u8;

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

pub async fn advertise(adv: ConnectableAdvertisement<'_>) -> Result<Connection, AdvertiseError> {
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

static ADV_SIGNAL: Signal<Result<Connection, AdvertiseError>> = Signal::new();

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

pub fn advertise_stop() -> Result<(), AdvertiseStopError> {
    let ret = unsafe { raw::sd_ble_gap_adv_stop(ADV_HANDLE) };
    match Error::convert(ret).dewarn(intern!("sd_ble_gap_adv_stop")) {
        Ok(()) => Ok(()),
        Err(Error::InvalidState) => Err(AdvertiseStopError::NotRunning),
        Err(e) => Err(e.into()),
    }
}

#[derive(defmt::Format)]
pub enum ConnectError {
    Stopped,
    Raw(Error),
}

impl From<Error> for ConnectError {
    fn from(err: Error) -> Self {
        ConnectError::Raw(err)
    }
}

static CONNECT_SIGNAL: Signal<Result<Connection, ConnectError>> = Signal::new();

pub async fn connect(whitelist: &[Address]) -> Result<Connection, ConnectError> {
    let (addr, fp) = match whitelist.len() {
        0 => depanic!("zero-length whitelist"),
        1 => (
            &whitelist[0] as *const Address as *const raw::ble_gap_addr_t,
            raw::BLE_GAP_SCAN_FP_ACCEPT_ALL as u8,
        ),
        _ => depanic!("todo"),
    };

    // TODO make configurable
    let mut scan_params: raw::ble_gap_scan_params_t = unsafe { mem::zeroed() };
    scan_params.set_extended(1);
    scan_params.set_active(1);
    scan_params.scan_phys = raw::BLE_GAP_PHY_1MBPS as u8;
    scan_params.interval = 2732;
    scan_params.window = 500;
    scan_params.set_filter_policy(fp);
    scan_params.timeout = 123;

    // TODO make configurable
    let mut conn_params: raw::ble_gap_conn_params_t = unsafe { mem::zeroed() };
    conn_params.min_conn_interval = 50;
    conn_params.max_conn_interval = 200;
    conn_params.slave_latency = 4;
    conn_params.conn_sup_timeout = 400; // 4 s

    let ret = unsafe { raw::sd_ble_gap_connect(addr, &mut scan_params, &mut conn_params, 1) };
    match Error::convert(ret) {
        Ok(()) => {}
        Err(err) => {
            warn!("sd_ble_gap_connect err {:?}", err);
            return Err(ConnectError::Raw(err));
        }
    }

    info!("connect started");

    // TODO handle future drop

    CONNECT_SIGNAL.wait().await
}

#[derive(defmt::Format)]
pub enum ConnectStopError {
    NotRunning,
    Raw(Error),
}

impl From<Error> for ConnectStopError {
    fn from(err: Error) -> Self {
        ConnectStopError::Raw(err)
    }
}

pub fn connect_stop() -> Result<(), ConnectStopError> {
    let ret = unsafe { raw::sd_ble_gap_connect_cancel() };
    match Error::convert(ret).dewarn(intern!("sd_ble_gap_connect_cancel")) {
        Ok(()) => Ok(()),
        Err(Error::InvalidState) => Err(ConnectStopError::NotRunning),
        Err(e) => Err(e.into()),
    }
}
