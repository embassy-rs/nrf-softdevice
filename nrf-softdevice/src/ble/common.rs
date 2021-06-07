use crate::raw;

pub(crate) unsafe fn on_evt(ble_evt: *const raw::ble_evt_t) {
    match (*ble_evt).header.evt_id as u32 {
        raw::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_REQUEST => on_user_mem_request(ble_evt),
        raw::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_RELEASE => on_user_mem_release(ble_evt),
        _ => {}
    }
}

fn on_user_mem_request(_ble_evt: *const raw::ble_evt_t) {
    trace!("on_user_mem_request");
}
fn on_user_mem_release(_ble_evt: *const raw::ble_evt_t) {
    trace!("on_user_mem_release");
}
