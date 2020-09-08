use core::ptr;

use crate::sd;

pub(crate) unsafe fn on_write(_ble_evt: *const sd::ble_evt_t, _gattc_evt: &sd::ble_gatts_evt_t) {}

pub(crate) unsafe fn on_rw_authorize_request(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gatts_evt_t,
) {
}

pub(crate) unsafe fn on_sys_attr_missing(
    _ble_evt: *const sd::ble_evt_t,
    gattc_evt: &sd::ble_gatts_evt_t,
) {
    sd::sd_ble_gatts_sys_attr_set(gattc_evt.conn_handle, ptr::null(), 0, 0);
}

pub(crate) unsafe fn on_hvc(_ble_evt: *const sd::ble_evt_t, _gattc_evt: &sd::ble_gatts_evt_t) {}

pub(crate) unsafe fn on_sc_confirm(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gatts_evt_t,
) {
}

pub(crate) unsafe fn on_exchange_mtu_request(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gatts_evt_t,
) {
}

pub(crate) unsafe fn on_timeout(_ble_evt: *const sd::ble_evt_t, _gattc_evt: &sd::ble_gatts_evt_t) {}

pub(crate) unsafe fn on_hvn_tx_complete(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gatts_evt_t,
) {
}
