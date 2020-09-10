use crate::raw;

pub(crate) fn on_ch_setup_request(
    _ble_evt: *const raw::ble_evt_t,
    _l2cap_evt: &raw::ble_l2cap_evt_t,
) {
}

pub(crate) fn on_ch_setup_refused(
    _ble_evt: *const raw::ble_evt_t,
    _l2cap_evt: &raw::ble_l2cap_evt_t,
) {
}

pub(crate) fn on_ch_setup(_ble_evt: *const raw::ble_evt_t, _l2cap_evt: &raw::ble_l2cap_evt_t) {}

pub(crate) fn on_ch_released(_ble_evt: *const raw::ble_evt_t, _l2cap_evt: &raw::ble_l2cap_evt_t) {}

pub(crate) fn on_ch_sdu_buf_released(
    _ble_evt: *const raw::ble_evt_t,
    _l2cap_evt: &raw::ble_l2cap_evt_t,
) {
}

pub(crate) fn on_ch_credit(_ble_evt: *const raw::ble_evt_t, _l2cap_evt: &raw::ble_l2cap_evt_t) {}

pub(crate) fn on_ch_rx(_ble_evt: *const raw::ble_evt_t, _l2cap_evt: &raw::ble_l2cap_evt_t) {}

pub(crate) fn on_ch_tx(_ble_evt: *const raw::ble_evt_t, _l2cap_evt: &raw::ble_l2cap_evt_t) {}
