use crate::sd;
use crate::util::*;

mod connection;
pub use connection::*;
pub mod gap;
pub mod gatt_client;
pub mod gatt_server;
pub mod l2cap;
pub mod uuid;

fn on_user_mem_request(_ble_evt: *const sd::ble_evt_t, _common_evt: &sd::ble_common_evt_t) {}
fn on_user_mem_release(_ble_evt: *const sd::ble_evt_t, _common_evt: &sd::ble_common_evt_t) {}

macro_rules! match_event {
    ($evt_ptr:ident, $($id:ident => $func:path[$field:ident]),* $(,)? ) => {
        let evt = &*$evt_ptr;
        defmt::trace!("ble evt {:istr}", evt_str(evt.header.evt_id as u32));
        match evt.header.evt_id as u32 {
            $(sd::$id => $func($evt_ptr, get_union_field($evt_ptr, &evt.evt.$field)) ),* ,
            x => depanic!("Unknown ble evt {:u32}", x),
        }
    };
}

#[rustfmt::skip]
pub(crate) unsafe fn on_evt(evt_ptr: *const sd::ble_evt_t) {
    match_event!(evt_ptr, 
        BLE_COMMON_EVTS_BLE_EVT_USER_MEM_REQUEST => on_user_mem_request[common_evt],
        BLE_COMMON_EVTS_BLE_EVT_USER_MEM_RELEASE => on_user_mem_release[common_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_CONNECTED => gap::on_connected[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => gap::on_disconnected[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE => gap::on_conn_param_update[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_SEC_PARAMS_REQUEST => gap::on_sec_params_request[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_SEC_INFO_REQUEST => gap::on_sec_info_request[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_PASSKEY_DISPLAY => gap::on_passkey_display[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_KEY_PRESSED => gap::on_key_pressed[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_KEY_REQUEST => gap::on_auth_key_request[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_LESC_DHKEY_REQUEST => gap::on_lesc_dhkey_request[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_STATUS => gap::on_auth_status[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_CONN_SEC_UPDATE => gap::on_conn_sec_update[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => gap::on_timeout[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_RSSI_CHANGED => gap::on_rssi_changed[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_ADV_REPORT => gap::on_adv_report[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_SEC_REQUEST => gap::on_sec_request[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE_REQUEST => gap::on_conn_param_update_request[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_SCAN_REQ_REPORT => gap::on_scan_req_report[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE_REQUEST => gap::on_phy_update_request[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE => gap::on_phy_update[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE_REQUEST => gap::on_data_length_update_request[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE => gap::on_data_length_update[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_QOS_CHANNEL_SURVEY_REPORT => gap::on_qos_channel_survey_report[gap_evt],
        BLE_GAP_EVTS_BLE_GAP_EVT_ADV_SET_TERMINATED => gap::on_adv_set_terminated[gap_evt],
        BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REQUEST => l2cap::on_ch_setup_request[l2cap_evt],
        BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REFUSED => l2cap::on_ch_setup_refused[l2cap_evt],
        BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP => l2cap::on_ch_setup[l2cap_evt],
        BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RELEASED => l2cap::on_ch_released[l2cap_evt],
        BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SDU_BUF_RELEASED => l2cap::on_ch_sdu_buf_released[l2cap_evt],
        BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_CREDIT => l2cap::on_ch_credit[l2cap_evt],
        BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RX => l2cap::on_ch_rx[l2cap_evt],
        BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_TX => l2cap::on_ch_tx[l2cap_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_PRIM_SRVC_DISC_RSP => gatt_client::on_prim_srvc_disc_rsp[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_REL_DISC_RSP => gatt_client::on_rel_disc_rsp[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_DISC_RSP => gatt_client::on_char_disc_rsp[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_DESC_DISC_RSP => gatt_client::on_desc_disc_rsp[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_ATTR_INFO_DISC_RSP => gatt_client::on_attr_info_disc_rsp[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VAL_BY_UUID_READ_RSP => gatt_client::on_char_val_by_uuid_read_rsp[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_READ_RSP => gatt_client::on_read_rsp[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VALS_READ_RSP => gatt_client::on_char_vals_read_rsp[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_RSP => gatt_client::on_write_rsp[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_HVX => gatt_client::on_hvx[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_EXCHANGE_MTU_RSP => gatt_client::on_exchange_mtu_rsp[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_TIMEOUT => gatt_client::on_timeout[gattc_evt],
        BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_CMD_TX_COMPLETE => gatt_client::on_write_cmd_tx_complete[gattc_evt],
        BLE_GATTS_EVTS_BLE_GATTS_EVT_WRITE => gatt_server::on_write[gatts_evt],
        BLE_GATTS_EVTS_BLE_GATTS_EVT_RW_AUTHORIZE_REQUEST => gatt_server::on_rw_authorize_request[gatts_evt],
        BLE_GATTS_EVTS_BLE_GATTS_EVT_SYS_ATTR_MISSING => gatt_server::on_sys_attr_missing[gatts_evt],
        BLE_GATTS_EVTS_BLE_GATTS_EVT_HVC => gatt_server::on_hvc[gatts_evt],
        BLE_GATTS_EVTS_BLE_GATTS_EVT_SC_CONFIRM => gatt_server::on_sc_confirm[gatts_evt],
        BLE_GATTS_EVTS_BLE_GATTS_EVT_EXCHANGE_MTU_REQUEST => gatt_server::on_exchange_mtu_request[gatts_evt],
        BLE_GATTS_EVTS_BLE_GATTS_EVT_TIMEOUT => gatt_server::on_timeout[gatts_evt],
        BLE_GATTS_EVTS_BLE_GATTS_EVT_HVN_TX_COMPLETE => gatt_server::on_hvn_tx_complete[gatts_evt],
    );
}

#[rustfmt::skip]
fn evt_str(evt: u32) -> defmt::Str {
    match evt {
        sd::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_REQUEST => defmt::intern!("USER_MEM_REQUEST"),
        sd::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_RELEASE => defmt::intern!("USER_MEM_RELEASE"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONNECTED => defmt::intern!("GAP CONNECTED"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => defmt::intern!("GAP DISCONNECTED"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE => defmt::intern!("GAP CONN_PARAM_UPDATE"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_PARAMS_REQUEST => defmt::intern!("GAP SEC_PARAMS_REQUEST"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_INFO_REQUEST => defmt::intern!("GAP SEC_INFO_REQUEST"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_PASSKEY_DISPLAY => defmt::intern!("GAP PASSKEY_DISPLAY"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_KEY_PRESSED => defmt::intern!("GAP KEY_PRESSED"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_KEY_REQUEST => defmt::intern!("GAP AUTH_KEY_REQUEST"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_LESC_DHKEY_REQUEST => defmt::intern!("GAP LESC_DHKEY_REQUEST"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_STATUS => defmt::intern!("GAP AUTH_STATUS"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_SEC_UPDATE => defmt::intern!("GAP CONN_SEC_UPDATE"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => defmt::intern!("GAP TIMEOUT"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_RSSI_CHANGED => defmt::intern!("GAP RSSI_CHANGED"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_REPORT => defmt::intern!("GAP ADV_REPORT"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_REQUEST => defmt::intern!("GAP SEC_REQUEST"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE_REQUEST => defmt::intern!("GAP CONN_PARAM_UPDATE_REQUEST"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_SCAN_REQ_REPORT => defmt::intern!("GAP SCAN_REQ_REPORT"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE_REQUEST => defmt::intern!("GAP PHY_UPDATE_REQUEST"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE => defmt::intern!("GAP PHY_UPDATE"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE_REQUEST => defmt::intern!("GAP DATA_LENGTH_UPDATE_REQUEST"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE => defmt::intern!("GAP DATA_LENGTH_UPDATE"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_QOS_CHANNEL_SURVEY_REPORT => defmt::intern!("GAP QOS_CHANNEL_SURVEY_REPORT"),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_SET_TERMINATED => defmt::intern!("GAP ADV_SET_TERMINATED"),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REQUEST => defmt::intern!("L2CAP CH_SETUP_REQUEST"),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REFUSED => defmt::intern!("L2CAP CH_SETUP_REFUSED"),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP => defmt::intern!("L2CAP CH_SETUP"),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RELEASED => defmt::intern!("L2CAP CH_RELEASED"),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SDU_BUF_RELEASED => defmt::intern!("L2CAP CH_SDU_BUF_RELEASED"),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_CREDIT => defmt::intern!("L2CAP CH_CREDIT"),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RX => defmt::intern!("L2CAP CH_RX"),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_TX => defmt::intern!("L2CAP CH_TX"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_PRIM_SRVC_DISC_RSP => defmt::intern!("GATTC PRIM_SRVC_DISC_RSP"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_REL_DISC_RSP => defmt::intern!("GATTC REL_DISC_RSP"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_DISC_RSP => defmt::intern!("GATTC CHAR_DISC_RSP"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_DESC_DISC_RSP => defmt::intern!("GATTC DESC_DISC_RSP"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_ATTR_INFO_DISC_RSP => defmt::intern!("GATTC ATTR_INFO_DISC_RSP"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VAL_BY_UUID_READ_RSP => defmt::intern!("GATTC CHAR_VAL_BY_UUID_READ_RSP"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_READ_RSP => defmt::intern!("GATTC READ_RSP"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VALS_READ_RSP => defmt::intern!("GATTC CHAR_VALS_READ_RSP"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_RSP => defmt::intern!("GATTC WRITE_RSP"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_HVX => defmt::intern!("GATTC HVX"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_EXCHANGE_MTU_RSP => defmt::intern!("GATTC EXCHANGE_MTU_RSP"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_TIMEOUT => defmt::intern!("GATTC TIMEOUT"),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_CMD_TX_COMPLETE => defmt::intern!("GATTC WRITE_CMD_TX_COMPLETE"),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_WRITE => defmt::intern!("GATTS WRITE"),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_RW_AUTHORIZE_REQUEST => defmt::intern!("GATTS RW_AUTHORIZE_REQUEST"),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_SYS_ATTR_MISSING => defmt::intern!("GATTS SYS_ATTR_MISSING"),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVC => defmt::intern!("GATTS HVC"),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_SC_CONFIRM => defmt::intern!("GATTS SC_CONFIRM"),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_EXCHANGE_MTU_REQUEST => defmt::intern!("GATTS EXCHANGE_MTU_REQUEST"),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_TIMEOUT => defmt::intern!("GATTS TIMEOUT"),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVN_TX_COMPLETE => defmt::intern!("GATTS HVN_TX_COMPLETE"),
        x => depanic!("Unknown ble evt {:u32}", x),
    }
}
