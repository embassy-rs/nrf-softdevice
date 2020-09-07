use core::ptr;

use crate::error::Error;
use crate::sd;
use crate::util::*;

pub(crate) unsafe fn on_prim_srvc_disc_rsp(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_rel_disc_rsp(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_char_disc_rsp(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_desc_disc_rsp(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_attr_info_disc_rsp(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_char_val_by_uuid_read_rsp(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_read_rsp(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_char_vals_read_rsp(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_write_rsp(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_hvx(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_exchange_mtu_rsp(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_timeout(evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_write_cmd_tx_complete(evt: &sd::ble_gattc_evt_t) {}
