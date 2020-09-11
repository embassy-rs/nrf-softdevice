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
        #[cfg(feature = "ble-central")]
        Role::Central => crate::gap_central::CONNECT_SIGNAL.signal(Ok(conn)),
        #[cfg(feature = "ble-peripheral")]
        Role::Peripheral => crate::gap_peripheral::ADV_SIGNAL.signal(Ok(conn)),
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
        #[cfg(feature = "ble-central")]
        raw::BLE_GAP_TIMEOUT_SRC_CONN => crate::gap_central::CONNECT_SIGNAL
            .signal(Err(crate::gap_central::ConnectError::Stopped)),
        x => depanic!("unknown timeout src {:u32}", x),
    }
}

pub(crate) unsafe fn on_rssi_changed(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

pub(crate) unsafe fn on_sec_request(
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

#[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
pub(crate) unsafe fn on_data_length_update_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}

#[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
pub(crate) unsafe fn on_data_length_update(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
}
