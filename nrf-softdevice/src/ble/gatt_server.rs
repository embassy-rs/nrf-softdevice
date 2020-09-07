use core::ptr;

use crate::error::Error;
use crate::sd;
use crate::util::*;

pub(crate) unsafe fn on_write(evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_rw_authorize_request(evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_sys_attr_missing(evt: &sd::ble_gatts_evt_t) {
    sd::sd_ble_gatts_sys_attr_set(evt.conn_handle, ptr::null(), 0, 0);
}
pub(crate) unsafe fn on_hvc(evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_sc_confirm(evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_exchange_mtu_request(evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_timeout(evt: &sd::ble_gatts_evt_t) {}
pub(crate) unsafe fn on_hvn_tx_complete(evt: &sd::ble_gatts_evt_t) {}
