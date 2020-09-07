use core::convert::TryFrom;
use core::mem::MaybeUninit;
use core::ptr;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::error::Error;
use crate::util::*;
use crate::{interrupt, sd};

pub mod gap;
pub mod gatt_client;
pub mod gatt_server;
pub mod l2cap;
pub mod uuid;

enum Event {
    UserMemRequest,
    UserMemRelease,

    Gap(gap::Event),
    L2cap(l2cap::Event),
    GattClient(gatt_client::Event),
    GattServer(gatt_server::Event),
}

//conn_handle: evt.evt.gap_evt.conn_handle,
//params: gap::EventParams::Connected(evt.evt.gap_evt.params.connected),

#[rustfmt::skip]
pub(crate) unsafe fn on_ble_evt(evt: &sd::ble_evt_t) {
    let evt = match evt.header.evt_id as u32 {
        sd::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_REQUEST => Event::UserMemRequest,
        sd::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_RELEASE => Event::UserMemRelease,
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONNECTED => Event::Gap(gap::Event::Connected{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.connected}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => Event::Gap(gap::Event::Disconnected{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.disconnected}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE => Event::Gap(gap::Event::ConnParamUpdate{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.conn_param_update}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_PARAMS_REQUEST => Event::Gap(gap::Event::SecParamsRequest{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.sec_params_request}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_INFO_REQUEST => Event::Gap(gap::Event::SecInfoRequest{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.sec_info_request}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_PASSKEY_DISPLAY => Event::Gap(gap::Event::PasskeyDisplay{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.passkey_display}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_KEY_PRESSED => Event::Gap(gap::Event::KeyPressed{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.key_pressed}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_KEY_REQUEST => Event::Gap(gap::Event::AuthKeyRequest{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.auth_key_request}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_LESC_DHKEY_REQUEST => Event::Gap(gap::Event::LescDhkeyRequest{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.lesc_dhkey_request}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_STATUS => Event::Gap(gap::Event::AuthStatus{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.auth_status}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_SEC_UPDATE => Event::Gap(gap::Event::ConnSecUpdate{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.conn_sec_update}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => Event::Gap(gap::Event::Timeout{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.timeout}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_RSSI_CHANGED => Event::Gap(gap::Event::RssiChanged{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.rssi_changed}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_REPORT => Event::Gap(gap::Event::AdvReport{params: evt.evt.gap_evt.params.adv_report}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_REQUEST => Event::Gap(gap::Event::SecRequest{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.sec_request}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE_REQUEST => Event::Gap(gap::Event::ConnParamUpdateRequest{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.conn_param_update_request}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_SCAN_REQ_REPORT => Event::Gap(gap::Event::ScanReqReport{params: evt.evt.gap_evt.params.scan_req_report}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE_REQUEST => Event::Gap(gap::Event::PhyUpdateRequest{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.phy_update_request}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE => Event::Gap(gap::Event::PhyUpdate{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.phy_update}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE_REQUEST => Event::Gap(gap::Event::DataLengthUpdateRequest{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.data_length_update_request}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE => Event::Gap(gap::Event::DataLengthUpdate{conn_handle: evt.evt.gap_evt.conn_handle, params: evt.evt.gap_evt.params.data_length_update}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_QOS_CHANNEL_SURVEY_REPORT => Event::Gap(gap::Event::QosChannelSurveyReport{params: evt.evt.gap_evt.params.qos_channel_survey_report}),
        sd::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_SET_TERMINATED => Event::Gap(gap::Event::AdvSetTerminated{params: evt.evt.gap_evt.params.adv_set_terminated}),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REQUEST => Event::L2cap(l2cap::Event::ChSetupRequest),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REFUSED => Event::L2cap(l2cap::Event::ChSetupRefused),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP => Event::L2cap(l2cap::Event::ChSetup),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RELEASED => Event::L2cap(l2cap::Event::ChReleased),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SDU_BUF_RELEASED => Event::L2cap(l2cap::Event::ChSduBufReleased),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_CREDIT => Event::L2cap(l2cap::Event::ChCredit),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RX => Event::L2cap(l2cap::Event::ChRx),
        sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_TX => Event::L2cap(l2cap::Event::ChTx),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_PRIM_SRVC_DISC_RSP => Event::GattClient(gatt_client::Event::PrimSrvcDiscRsp),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_REL_DISC_RSP => Event::GattClient(gatt_client::Event::RelDiscRsp),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_DISC_RSP => Event::GattClient(gatt_client::Event::CharDiscRsp),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_DESC_DISC_RSP => Event::GattClient(gatt_client::Event::DescDiscRsp),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_ATTR_INFO_DISC_RSP => Event::GattClient(gatt_client::Event::AttrInfoDiscRsp),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VAL_BY_UUID_READ_RSP => Event::GattClient(gatt_client::Event::CharValByUuidReadRsp),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_READ_RSP => Event::GattClient(gatt_client::Event::ReadRsp),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VALS_READ_RSP => Event::GattClient(gatt_client::Event::CharValsReadRsp),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_RSP => Event::GattClient(gatt_client::Event::WriteRsp),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_HVX => Event::GattClient(gatt_client::Event::Hvx),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_EXCHANGE_MTU_RSP => Event::GattClient(gatt_client::Event::ExchangeMtuRsp),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_TIMEOUT => Event::GattClient(gatt_client::Event::Timeout),
        sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_CMD_TX_COMPLETE => Event::GattClient(gatt_client::Event::WriteCmdTxComplete),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_WRITE => Event::GattServer(gatt_server::Event::Write{conn_handle: evt.evt.gatts_evt.conn_handle, params: evt.evt.gatts_evt.params.write}),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_RW_AUTHORIZE_REQUEST => Event::GattServer(gatt_server::Event::RwAuthorizeRequest{conn_handle: evt.evt.gatts_evt.conn_handle, params: evt.evt.gatts_evt.params.authorize_request}),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_SYS_ATTR_MISSING => Event::GattServer(gatt_server::Event::SysAttrMissing{conn_handle: evt.evt.gatts_evt.conn_handle, params: evt.evt.gatts_evt.params.sys_attr_missing}),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVC => Event::GattServer(gatt_server::Event::Hvc{conn_handle: evt.evt.gatts_evt.conn_handle, params: evt.evt.gatts_evt.params.hvc}),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_SC_CONFIRM => Event::GattServer(gatt_server::Event::ScConfirm{conn_handle: evt.evt.gatts_evt.conn_handle}),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_EXCHANGE_MTU_REQUEST => Event::GattServer(gatt_server::Event::ExchangeMtuRequest{conn_handle: evt.evt.gatts_evt.conn_handle, params: evt.evt.gatts_evt.params.exchange_mtu_request}),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_TIMEOUT => Event::GattServer(gatt_server::Event::Timeout{conn_handle: evt.evt.gatts_evt.conn_handle, params: evt.evt.gatts_evt.params.timeout}),
        sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVN_TX_COMPLETE => Event::GattServer(gatt_server::Event::HvnTxComplete{conn_handle: evt.evt.gatts_evt.conn_handle, params: evt.evt.gatts_evt.params.hvn_tx_complete}),
        x => depanic!("Unknown ble evt {:u32}", x),
    };

    match evt {
        Event::Gap(e) => gap::on_evt(e),
        Event::L2cap(e) => l2cap::on_evt(e),
        Event::GattClient(e) => gatt_client::on_evt(e),
        Event::GattServer(e) => gatt_server::on_evt(e),
        _ => {}
    }
}
