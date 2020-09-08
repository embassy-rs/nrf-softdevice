use crate::sd;

pub(crate) unsafe fn on_prim_srvc_disc_rsp(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_rel_disc_rsp(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_char_disc_rsp(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_desc_disc_rsp(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_attr_info_disc_rsp(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_char_val_by_uuid_read_rsp(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_read_rsp(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_char_vals_read_rsp(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_write_rsp(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_hvx(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_exchange_mtu_rsp(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_timeout(_evt: &sd::ble_gattc_evt_t) {}
pub(crate) unsafe fn on_write_cmd_tx_complete(_evt: &sd::ble_gattc_evt_t) {}
