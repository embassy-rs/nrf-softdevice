use core::convert::TryFrom;
use core::mem::MaybeUninit;
use core::ptr;

use crate::error::Error;
use crate::util::*;
use crate::{interrupt, sd};

pub(crate) enum Event {
    Connected {
        conn_handle: u16,
        params: sd::ble_gap_evt_connected_t,
    },
    Disconnected {
        conn_handle: u16,
        params: sd::ble_gap_evt_disconnected_t,
    },
    ConnParamUpdate {
        conn_handle: u16,
        params: sd::ble_gap_evt_conn_param_update_t,
    },
    SecParamsRequest {
        conn_handle: u16,
        params: sd::ble_gap_evt_sec_params_request_t,
    },
    SecInfoRequest {
        conn_handle: u16,
        params: sd::ble_gap_evt_sec_info_request_t,
    },
    PasskeyDisplay {
        conn_handle: u16,
        params: sd::ble_gap_evt_passkey_display_t,
    },
    KeyPressed {
        conn_handle: u16,
        params: sd::ble_gap_evt_key_pressed_t,
    },
    AuthKeyRequest {
        conn_handle: u16,
        params: sd::ble_gap_evt_auth_key_request_t,
    },
    LescDhkeyRequest {
        conn_handle: u16,
        params: sd::ble_gap_evt_lesc_dhkey_request_t,
    },
    AuthStatus {
        conn_handle: u16,
        params: sd::ble_gap_evt_auth_status_t,
    },
    ConnSecUpdate {
        conn_handle: u16,
        params: sd::ble_gap_evt_conn_sec_update_t,
    },
    Timeout {
        conn_handle: u16,
        params: sd::ble_gap_evt_timeout_t,
    },
    RssiChanged {
        conn_handle: u16,
        params: sd::ble_gap_evt_rssi_changed_t,
    },
    AdvReport {
        params: sd::ble_gap_evt_adv_report_t,
    },
    SecRequest {
        conn_handle: u16,
        params: sd::ble_gap_evt_sec_request_t,
    },
    ConnParamUpdateRequest {
        conn_handle: u16,
        params: sd::ble_gap_evt_conn_param_update_request_t,
    },
    ScanReqReport {
        params: sd::ble_gap_evt_scan_req_report_t,
    },
    PhyUpdateRequest {
        conn_handle: u16,
        params: sd::ble_gap_evt_phy_update_request_t,
    },
    PhyUpdate {
        conn_handle: u16,
        params: sd::ble_gap_evt_phy_update_t,
    },
    DataLengthUpdateRequest {
        conn_handle: u16,
        params: sd::ble_gap_evt_data_length_update_request_t,
    },
    DataLengthUpdate {
        conn_handle: u16,
        params: sd::ble_gap_evt_data_length_update_t,
    },
    QosChannelSurveyReport {
        params: sd::ble_gap_evt_qos_channel_survey_report_t,
    },
    AdvSetTerminated {
        params: sd::ble_gap_evt_adv_set_terminated_t,
    },
}

impl Event {
    fn str(&self) -> defmt::Str {
        match self {
            Self::Connected { .. } => defmt::intern!("Connected"),
            Self::Disconnected { .. } => defmt::intern!("Disconnected"),
            Self::ConnParamUpdate { .. } => defmt::intern!("ConnParamUpdate"),
            Self::SecParamsRequest { .. } => defmt::intern!("SecParamsRequest"),
            Self::SecInfoRequest { .. } => defmt::intern!("SecInfoRequest"),
            Self::PasskeyDisplay { .. } => defmt::intern!("PasskeyDisplay"),
            Self::KeyPressed { .. } => defmt::intern!("KeyPressed"),
            Self::AuthKeyRequest { .. } => defmt::intern!("AuthKeyRequest"),
            Self::LescDhkeyRequest { .. } => defmt::intern!("LescDhkeyRequest"),
            Self::AuthStatus { .. } => defmt::intern!("AuthStatus"),
            Self::ConnSecUpdate { .. } => defmt::intern!("ConnSecUpdate"),
            Self::Timeout { .. } => defmt::intern!("Timeout"),
            Self::RssiChanged { .. } => defmt::intern!("RssiChanged"),
            Self::AdvReport { .. } => defmt::intern!("AdvReport"),
            Self::SecRequest { .. } => defmt::intern!("SecRequest"),
            Self::ConnParamUpdateRequest { .. } => defmt::intern!("ConnParamUpdateRequest"),
            Self::ScanReqReport { .. } => defmt::intern!("ScanReqReport"),
            Self::PhyUpdateRequest { .. } => defmt::intern!("PhyUpdateRequest"),
            Self::PhyUpdate { .. } => defmt::intern!("PhyUpdate"),
            Self::DataLengthUpdateRequest { .. } => defmt::intern!("DataLengthUpdateRequest"),
            Self::DataLengthUpdate { .. } => defmt::intern!("DataLengthUpdate"),
            Self::QosChannelSurveyReport { .. } => defmt::intern!("QosChannelSurveyReport"),
            Self::AdvSetTerminated { .. } => defmt::intern!("AdvSetTerminated"),
        }
    }
}

pub(crate) fn on_evt(evt: Event) {
    info!("gap evt {:istr}", evt.str());
    match evt {
        Event::Connected { .. } => ADV_SIGNAL.signal(()),
        Event::AdvSetTerminated { .. } => ADV_SIGNAL.signal(()),
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
        Ok(()) => {}
        Err(err) => depanic!("sd_ble_gap_adv_set_configure err {:?}", err),
    }

    let ret = unsafe { sd::sd_ble_gap_adv_start(ADV_HANDLE, 1 as u8) };
    match Error::convert(ret) {
        Ok(()) => {}
        Err(err) => depanic!("sd_ble_gap_adv_start err {:?}", err),
    }

    info!("Advertising started!");

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
