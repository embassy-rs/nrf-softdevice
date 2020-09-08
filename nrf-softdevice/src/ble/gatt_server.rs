use core::ptr;

use crate::sd;

pub(crate) unsafe fn on_write(_evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_rw_authorize_request(_evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_sys_attr_missing(evt: &sd::ble_gatts_evt_t) {
    sd::sd_ble_gatts_sys_attr_set(evt.conn_handle, ptr::null(), 0, 0);
}
pub(crate) unsafe fn on_hvc(_evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_sc_confirm(_evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_exchange_mtu_request(_evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_timeout(_evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_hvn_tx_complete(_evt: &sd::ble_gatts_evt_t) {}
