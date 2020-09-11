use crate::raw;
use crate::util::*;

mod connection;
pub use connection::*;
mod types;
pub use types::*;

pub mod gap;
#[cfg(feature = "ble-central")]
pub mod gap_central;
#[cfg(feature = "ble-peripheral")]
pub mod gap_peripheral;
pub mod gatt_client;
pub mod gatt_server;

#[cfg(feature = "ble-l2cap")]
pub mod l2cap;

fn on_user_mem_request(_ble_evt: *const raw::ble_evt_t, _common_evt: &raw::ble_common_evt_t) {}
fn on_user_mem_release(_ble_evt: *const raw::ble_evt_t, _common_evt: &raw::ble_common_evt_t) {}

#[rustfmt::skip]
pub(crate) unsafe fn on_evt(ble_evt: *const raw::ble_evt_t) {
    let evt = &*ble_evt;
    //defmt::trace!("ble evt {:istr}", evt_str(evt.header.evt_id as u32));
    match evt.header.evt_id as u32 {
        raw::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_REQUEST => on_user_mem_request(ble_evt, get_union_field(ble_evt, &evt.evt.common_evt)),
        raw::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_RELEASE => on_user_mem_release(ble_evt, get_union_field(ble_evt, &evt.evt.common_evt)),

        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONNECTED => gap::on_connected(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => gap::on_disconnected(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE => gap::on_conn_param_update(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_PARAMS_REQUEST => gap::on_sec_params_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-peripheral")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_INFO_REQUEST => gap_peripheral::on_sec_info_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_PASSKEY_DISPLAY => gap::on_passkey_display(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_KEY_PRESSED => gap::on_key_pressed(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_KEY_REQUEST => gap::on_auth_key_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_LESC_DHKEY_REQUEST => gap::on_lesc_dhkey_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_STATUS => gap::on_auth_status(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_SEC_UPDATE => gap::on_conn_sec_update(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => gap::on_timeout(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_RSSI_CHANGED => gap::on_rssi_changed(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-central")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_REPORT => gap_central::on_adv_report(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_REQUEST => gap::on_sec_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-central")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE_REQUEST => gap_central::on_conn_param_update_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-peripheral")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_SCAN_REQ_REPORT => gap_peripheral::on_scan_req_report(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE_REQUEST => gap::on_phy_update_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE => gap::on_phy_update(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(any(feature="s113", feature="s132", feature="s140"))]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE_REQUEST => gap::on_data_length_update_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(any(feature="s113", feature="s132", feature="s140"))]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE => gap::on_data_length_update(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-central")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_QOS_CHANNEL_SURVEY_REPORT => gap_central::on_qos_channel_survey_report(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-peripheral")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_SET_TERMINATED => gap_peripheral::on_adv_set_terminated(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),

        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REQUEST => l2cap::on_ch_setup_request(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REFUSED => l2cap::on_ch_setup_refused(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP => l2cap::on_ch_setup(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RELEASED => l2cap::on_ch_released(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SDU_BUF_RELEASED => l2cap::on_ch_sdu_buf_released(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_CREDIT => l2cap::on_ch_credit(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RX => l2cap::on_ch_rx(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_TX => l2cap::on_ch_tx(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),

        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_PRIM_SRVC_DISC_RSP => gatt_client::on_prim_srvc_disc_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_REL_DISC_RSP => gatt_client::on_rel_disc_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_DISC_RSP => gatt_client::on_char_disc_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_DESC_DISC_RSP => gatt_client::on_desc_disc_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_ATTR_INFO_DISC_RSP => gatt_client::on_attr_info_disc_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VAL_BY_UUID_READ_RSP => gatt_client::on_char_val_by_uuid_read_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_READ_RSP => gatt_client::on_read_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VALS_READ_RSP => gatt_client::on_char_vals_read_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_RSP => gatt_client::on_write_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_HVX => gatt_client::on_hvx(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_EXCHANGE_MTU_RSP => gatt_client::on_exchange_mtu_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_TIMEOUT => gatt_client::on_timeout(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_CMD_TX_COMPLETE => gatt_client::on_write_cmd_tx_complete(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),

        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_WRITE => gatt_server::on_write(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_RW_AUTHORIZE_REQUEST => gatt_server::on_rw_authorize_request(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_SYS_ATTR_MISSING => gatt_server::on_sys_attr_missing(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVC => gatt_server::on_hvc(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_SC_CONFIRM => gatt_server::on_sc_confirm(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_EXCHANGE_MTU_REQUEST => gatt_server::on_exchange_mtu_request(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_TIMEOUT => gatt_server::on_timeout(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVN_TX_COMPLETE => gatt_server::on_hvn_tx_complete(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),

        x => depanic!("Unknown ble evt {:u32}", x),
    }
}
