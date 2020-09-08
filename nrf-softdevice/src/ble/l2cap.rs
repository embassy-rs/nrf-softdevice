use crate::sd;

pub(crate) fn on_ch_setup_request(_evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_setup_refused(_evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_setup(_evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_released(_evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_sdu_buf_released(_evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_credit(_evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_rx(_evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_tx(_evt: &sd::ble_l2cap_evt_t) {}
