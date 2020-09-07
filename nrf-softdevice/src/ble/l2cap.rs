use crate::sd;

pub(crate) fn on_ch_setup_request(evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_setup_refused(evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_setup(evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_released(evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_sdu_buf_released(evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_credit(evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_rx(evt: &sd::ble_l2cap_evt_t) {}
pub(crate) fn on_ch_tx(evt: &sd::ble_l2cap_evt_t) {}
