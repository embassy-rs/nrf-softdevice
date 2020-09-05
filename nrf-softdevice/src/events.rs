use core::convert::TryFrom;
use core::mem::MaybeUninit;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::error::Error;
use crate::util::*;
use crate::{interrupt, sd};

static SWI2_SIGNAL: Signal<()> = Signal::new();

#[rustfmt::skip]
#[repr(u32)]
#[derive(defmt::Format, IntoPrimitive, TryFromPrimitive)]
enum SocEvent {
    Hfclkstarted = sd::NRF_SOC_EVTS_NRF_EVT_HFCLKSTARTED,
    PowerFailureWarning = sd::NRF_SOC_EVTS_NRF_EVT_POWER_FAILURE_WARNING,
    FlashOperationSuccess = sd::NRF_SOC_EVTS_NRF_EVT_FLASH_OPERATION_SUCCESS,
    FlashOperationError = sd::NRF_SOC_EVTS_NRF_EVT_FLASH_OPERATION_ERROR,
    RadioBlocked = sd::NRF_SOC_EVTS_NRF_EVT_RADIO_BLOCKED,
    RadioCanceled = sd::NRF_SOC_EVTS_NRF_EVT_RADIO_CANCELED,
    RadioSignalCallbackInvalidReturn = sd::NRF_SOC_EVTS_NRF_EVT_RADIO_SIGNAL_CALLBACK_INVALID_RETURN,
    RadioSessionIdle = sd::NRF_SOC_EVTS_NRF_EVT_RADIO_SESSION_IDLE,
    RadioSessionClosed = sd::NRF_SOC_EVTS_NRF_EVT_RADIO_SESSION_CLOSED,
    PowerUsbPowerReady = sd::NRF_SOC_EVTS_NRF_EVT_POWER_USB_POWER_READY,
    PowerUsbDetected = sd::NRF_SOC_EVTS_NRF_EVT_POWER_USB_DETECTED,
    PowerUsbRemoved = sd::NRF_SOC_EVTS_NRF_EVT_POWER_USB_REMOVED,
}

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

fn on_soc_evt(evt: u32) {
    let evt = match SocEvent::try_from(evt) {
        Ok(evt) => evt,
        Err(_) => depanic!("Unknown soc evt {:u32}", evt),
    };

    info!("soc evt {:?}", evt);
    match evt {
        SocEvent::FlashOperationError => crate::flash::on_flash_error(),
        SocEvent::FlashOperationSuccess => crate::flash::on_flash_success(),
        _ => {}
    }
}

fn on_ble_evt(evt: &sd::ble_evt_t) {
    let evt_id = evt.header.evt_id as u32;
    let evt = match BleEvent::try_from(evt_id) {
        Ok(evt) => evt,
        Err(_) => depanic!("Unknown ble evt {:u32}", evt_id),
    };

    info!("ble evt {:?}", evt);
}

// TODO actually derive this from the headers + the ATT_MTU
const BLE_EVT_MAX_SIZE: u16 = 128;

pub async fn run() {
    loop {
        SWI2_SIGNAL.wait().await;

        unsafe {
            let mut evt: u32 = 0;
            loop {
                match Error::convert(sd::sd_evt_get(&mut evt as _)) {
                    Ok(()) => on_soc_evt(evt),
                    Err(Error::NotFound) => break,
                    Err(err) => depanic!("sd_evt_get err {:?}", err),
                }
            }

            // Using u32 since the buffer has to be aligned to 4
            let mut evt: MaybeUninit<[u32; BLE_EVT_MAX_SIZE as usize / 4]> = MaybeUninit::uninit();

            loop {
                let mut len: u16 = BLE_EVT_MAX_SIZE;
                let ret = sd::sd_ble_evt_get(evt.as_mut_ptr() as *mut u8, &mut len as _);
                match Error::convert(ret) {
                    Ok(()) => on_ble_evt(&*(evt.as_ptr() as *const sd::ble_evt_t)),
                    Err(Error::NotFound) => break,
                    Err(Error::BleNotEnabled) => break,
                    Err(Error::NoMem) => depanic!("BUG: BLE_EVT_MAX_SIZE is too low"),
                    Err(err) => depanic!("sd_ble_evt_get err {:?}", err),
                }
            }
        }
    }
}

#[interrupt]
unsafe fn SWI2_EGU2() {
    SWI2_SIGNAL.signal(());
}
