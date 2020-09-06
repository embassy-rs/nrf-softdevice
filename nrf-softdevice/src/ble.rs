use core::convert::TryFrom;
use core::mem::MaybeUninit;
use core::ptr;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::error::Error;
use crate::util::*;
use crate::{interrupt, sd};

#[rustfmt::skip]
#[repr(u32)]
#[derive(defmt::Format, IntoPrimitive, TryFromPrimitive)]
enum BleEvent {
    CommonUserMemRequest = sd::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_REQUEST,
    CommonUserMemRelease = sd::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_RELEASE,
    GapConnected = sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONNECTED,
    GapDisconnected = sd::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED,
    GapConnParamUpdate = sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE,
    GapSecParamsRequest = sd::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_PARAMS_REQUEST,
    GapSecInfoRequest = sd::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_INFO_REQUEST,
    GapPasskeyDisplay = sd::BLE_GAP_EVTS_BLE_GAP_EVT_PASSKEY_DISPLAY,
    GapKeyPressed = sd::BLE_GAP_EVTS_BLE_GAP_EVT_KEY_PRESSED,
    GapAuthKeyRequest = sd::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_KEY_REQUEST,
    GapLescDhkeyRequest = sd::BLE_GAP_EVTS_BLE_GAP_EVT_LESC_DHKEY_REQUEST,
    GapAuthStatus = sd::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_STATUS,
    GapConnSecUpdate = sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_SEC_UPDATE,
    GapTimeout = sd::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT,
    GapRssiChanged = sd::BLE_GAP_EVTS_BLE_GAP_EVT_RSSI_CHANGED,
    GapAdvReport = sd::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_REPORT,
    GapSecRequest = sd::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_REQUEST,
    GapConnParamUpdateRequest = sd::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE_REQUEST,
    GapScanReqReport = sd::BLE_GAP_EVTS_BLE_GAP_EVT_SCAN_REQ_REPORT,
    GapPhyUpdateRequest = sd::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE_REQUEST,
    GapPhyUpdate = sd::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE,
    GapDataLengthUpdateRequest = sd::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE_REQUEST,
    GapDataLengthUpdate = sd::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE,
    GapQosChannelSurveyReport = sd::BLE_GAP_EVTS_BLE_GAP_EVT_QOS_CHANNEL_SURVEY_REPORT,
    GapAdvSetTerminated = sd::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_SET_TERMINATED,
    L2CapChSetupRequest = sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REQUEST,
    L2CapChSetupRefused = sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REFUSED,
    L2CapChSetup = sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP,
    L2CapChReleased = sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RELEASED,
    L2CapChSduBufReleased = sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SDU_BUF_RELEASED,
    L2CapChCredit = sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_CREDIT,
    L2CapChRx = sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RX,
    L2CapChTx = sd::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_TX,
    GattcPrimSrvcDiscRsp = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_PRIM_SRVC_DISC_RSP,
    GattcRelDiscRsp = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_REL_DISC_RSP,
    GattcCharDiscRsp = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_DISC_RSP,
    GattcDescDiscRsp = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_DESC_DISC_RSP,
    GattcAttrInfoDiscRsp = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_ATTR_INFO_DISC_RSP,
    GattcCharValByUuidReadRsp = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VAL_BY_UUID_READ_RSP,
    GattcReadRsp = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_READ_RSP,
    GattcCharValsReadRsp = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VALS_READ_RSP,
    GattcWriteRsp = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_RSP,
    GattcHvx = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_HVX,
    GattcExchangeMtuRsp = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_EXCHANGE_MTU_RSP,
    GattcTimeout = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_TIMEOUT,
    GattcWriteCmdTxComplete = sd::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_CMD_TX_COMPLETE,
    GattsEvtWrite = sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_WRITE,
    GattsEvtRwAuthorizeRequest = sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_RW_AUTHORIZE_REQUEST,
    GattsEvtSysAttrMissing = sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_SYS_ATTR_MISSING,
    GattsEvtHvc = sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVC,
    GattsEvtScConfirm = sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_SC_CONFIRM,
    GattsEvtExchangeMtuRequest = sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_EXCHANGE_MTU_REQUEST,
    GattsEvtTimeout = sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_TIMEOUT,
    GattsEvtHvnTxComplete = sd::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVN_TX_COMPLETE,
}

pub(crate) fn on_ble_evt(evt: &sd::ble_evt_t) {
    let evt_id = evt.header.evt_id as u32;
    let evt = match BleEvent::try_from(evt_id) {
        Ok(evt) => evt,
        Err(_) => depanic!("Unknown ble evt {:u32}", evt_id),
    };

    info!("ble evt {:?}", evt);

    match evt {
        BleEvent::GapConnected => ADV_SIGNAL.signal(()),
        BleEvent::GapAdvSetTerminated => ADV_SIGNAL.signal(()),
        _ => {}
    }
}

pub enum ConnectableAdvertisement<'a> {
    ScannableUndirected {
        adv_data: &'a [u8],
        scan_data: &'a [u8],
    },
    NonscannableDirected {
        scan_data: &'a [u8],
    },
    NonscannableDirectedHighDuty {
        scan_data: &'a [u8],
    },
    ExtendedNonscannableUndirected {
        adv_data: &'a [u8],
    },
    ExtendedNonscannableDirected {
        adv_data: &'a [u8],
    },
}

static mut ADV_HANDLE: u8 = sd::BLE_GAP_ADV_SET_HANDLE_NOT_SET as u8;

pub async fn advertise(adv: ConnectableAdvertisement<'_>) {
    // TODO make these configurable, only the right params based on type?
    let mut adv_params: sd::ble_gap_adv_params_t = unsafe { core::mem::zeroed() };
    adv_params.properties.type_ = sd::BLE_GAP_ADV_TYPE_CONNECTABLE_SCANNABLE_UNDIRECTED as u8;
    adv_params.primary_phy = sd::BLE_GAP_PHY_1MBPS as u8;
    adv_params.secondary_phy = sd::BLE_GAP_PHY_1MBPS as u8;
    adv_params.duration = sd::BLE_GAP_ADV_TIMEOUT_GENERAL_UNLIMITED as u16;
    adv_params.interval = 100;
    adv_params.duration = 100;

    let (adv_data, scan_data) = match adv {
        ConnectableAdvertisement::ScannableUndirected {
            adv_data,
            scan_data,
        } => (Some(adv_data), Some(scan_data)),
        ConnectableAdvertisement::NonscannableDirected { scan_data } => (None, Some(scan_data)),
        ConnectableAdvertisement::NonscannableDirectedHighDuty { scan_data } => {
            (None, Some(scan_data))
        }
        ConnectableAdvertisement::ExtendedNonscannableUndirected { adv_data } => {
            (Some(adv_data), None)
        }
        ConnectableAdvertisement::ExtendedNonscannableDirected { adv_data } => {
            (Some(adv_data), None)
        }
    };

    let map_data = |data: Option<&[u8]>| {
        if let Some(data) = data {
            assert!(data.len() < u16::MAX as usize);
            sd::ble_data_t {
                p_data: data.as_ptr() as _,
                len: data.len() as u16,
            }
        } else {
            sd::ble_data_t {
                p_data: ptr::null_mut(),
                len: 0,
            }
        }
    };

    let datas = sd::ble_gap_adv_data_t {
        adv_data: map_data(adv_data),
        scan_rsp_data: map_data(scan_data),
    };

    let ret = unsafe {
        sd::sd_ble_gap_adv_set_configure(&mut ADV_HANDLE as _, &datas as _, &adv_params as _)
    };

    match Error::convert(ret) {
        Ok(()) => info!("advertising configured!"),
        Err(err) => depanic!("sd_ble_gap_adv_set_configure err {:?}", err),
    }

    let ret = unsafe { sd::sd_ble_gap_adv_start(ADV_HANDLE, 1 as u8) };
    match Error::convert(ret) {
        Ok(()) => info!("advertising started!"),
        Err(err) => depanic!("sd_ble_gap_adv_start err {:?}", err),
    }

    // The structs above need to be kept alive for the entire duration of the advertising procedure.

    ADV_SIGNAL.wait().await;
}

static ADV_SIGNAL: Signal<()> = Signal::new();

enum NonconnectableAdvertisement {
    ScannableUndirected,
    NonscannableUndirected,
    ExtendedScannableUndirected,
    ExtendedScannableDirected,
    ExtendedNonscannableUndirected,
    ExtendedNonscannableDirected,
}
