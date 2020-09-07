use core::convert::TryFrom;
use core::mem::MaybeUninit;
use core::ptr;

use crate::error::Error;
use crate::util::*;
use crate::{interrupt, sd};

pub(crate) enum Event {
    Write {
        conn_handle: u16,
        params: sd::ble_gatts_evt_write_t,
    },
    RwAuthorizeRequest {
        conn_handle: u16,
        params: sd::ble_gatts_evt_rw_authorize_request_t,
    },
    SysAttrMissing {
        conn_handle: u16,
        params: sd::ble_gatts_evt_sys_attr_missing_t,
    },
    Hvc {
        conn_handle: u16,
        params: sd::ble_gatts_evt_hvc_t,
    },
    ScConfirm {
        conn_handle: u16,
    },
    ExchangeMtuRequest {
        conn_handle: u16,
        params: sd::ble_gatts_evt_exchange_mtu_request_t,
    },
    Timeout {
        conn_handle: u16,
        params: sd::ble_gatts_evt_timeout_t,
    },
    HvnTxComplete {
        conn_handle: u16,
        params: sd::ble_gatts_evt_hvn_tx_complete_t,
    },
}

impl Event {
    fn str(&self) -> defmt::Str {
        match self {
            Self::Write { .. } => defmt::intern!("Write"),
            Self::RwAuthorizeRequest { .. } => defmt::intern!("RwAuthorizeRequest"),
            Self::SysAttrMissing { .. } => defmt::intern!("SysAttrMissing"),
            Self::Hvc { .. } => defmt::intern!("Hvc"),
            Self::ScConfirm { .. } => defmt::intern!("ScConfirm"),
            Self::ExchangeMtuRequest { .. } => defmt::intern!("ExchangeMtuRequest"),
            Self::Timeout { .. } => defmt::intern!("Timeout"),
            Self::HvnTxComplete { .. } => defmt::intern!("HvnTxComplete"),
        }
    }
}

pub(crate) unsafe fn on_evt(evt: Event) {
    info!("gatts evt {:istr}", evt.str());

    match evt {
        Event::SysAttrMissing { conn_handle, .. } => {
            sd::sd_ble_gatts_sys_attr_set(conn_handle, ptr::null(), 0, 0);
        }
        _ => {}
    }
}
