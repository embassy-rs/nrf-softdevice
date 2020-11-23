//! Bluetooth Central operations. Central devices scan for advertisements from Peripheral devices and connect to them.
//!
//! Typically the Central device is the higher-powered device, such as a smartphone or laptop, since scanning is more
//! power-hungry than advertising.

use core::mem;
use core::ptr;

use crate::ble::{Address, Connection, ConnectionState};
use crate::raw;
use crate::util::*;
use crate::{RawError, Softdevice};

pub(crate) unsafe fn on_adv_report(_ble_evt: *const raw::ble_evt_t, _gap_evt: &raw::ble_gap_evt_t) {
    trace!("central on_adv_report");
}

pub(crate) unsafe fn on_qos_channel_survey_report(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
    trace!("central on_qos_channel_survey_report");
}

pub(crate) unsafe fn on_conn_param_update_request(
    _ble_evt: *const raw::ble_evt_t,
    _gap_evt: &raw::ble_gap_evt_t,
) {
    trace!("central on_conn_param_update_request");
}

#[derive(defmt::Format)]
pub enum ConnectError {
    Stopped,
    Raw(RawError),
}

impl From<RawError> for ConnectError {
    fn from(err: RawError) -> Self {
        ConnectError::Raw(err)
    }
}

pub(crate) static CONNECT_PORTAL: Portal<Result<Connection, ConnectError>> = Portal::new();

// Begins an ATT MTU exchange procedure, followed by a data length update request as necessary.
pub async fn connect(
    sd: &Softdevice,
    whitelist: &[Address],
    config: Config,
) -> Result<Connection, ConnectError> {
    let (addr, fp) = match whitelist.len() {
        0 => depanic!("zero-length whitelist"),
        1 => (
            &whitelist[0] as *const Address as *const raw::ble_gap_addr_t,
            raw::BLE_GAP_SCAN_FP_ACCEPT_ALL as u8,
        ),
        _ => depanic!("todo"),
    };

    // in units of 625us
    let scan_interval: u32 = 2732;
    let scan_window: u32 = 500;

    // TODO make configurable
    let mut scan_params: raw::ble_gap_scan_params_t = unsafe { mem::zeroed() };
    scan_params.set_extended(1);
    scan_params.set_active(1);
    scan_params.scan_phys = raw::BLE_GAP_PHY_1MBPS as u8;
    scan_params.set_filter_policy(fp);
    scan_params.timeout = 123;

    // s122 has these in us instead of 625us :shrug:
    #[cfg(not(feature = "s122"))]
    {
        scan_params.interval = scan_interval as u16;
        scan_params.window = scan_window as u16;
    }
    #[cfg(feature = "s122")]
    {
        scan_params.interval_us = scan_interval * 625;
        scan_params.window_us = scan_window * 625;
    }

    let d = OnDrop::new(|| {
        let ret = unsafe { raw::sd_ble_gap_connect_cancel() };
        let _ = RawError::convert(ret).dewarn(intern!("sd_ble_gap_connect_cancel"));
    });

    // TODO make configurable
    let mut conn_params: raw::ble_gap_conn_params_t = unsafe { mem::zeroed() };
    conn_params.min_conn_interval = 50;
    conn_params.max_conn_interval = 200;
    conn_params.slave_latency = 4;
    conn_params.conn_sup_timeout = 400; // 4 s

    let ret = unsafe { raw::sd_ble_gap_connect(addr, &mut scan_params, &mut conn_params, 1) };
    match RawError::convert(ret) {
        Ok(()) => {}
        Err(err) => {
            warn!("sd_ble_gap_connect err {:?}", err);
            return Err(ConnectError::Raw(err));
        }
    }

    info!("connect started");

    let conn = CONNECT_PORTAL.wait_once(|res| res).await?;

    conn.with_state(|state| {
        state.rx_phys = config.tx_phys;
        state.tx_phys = config.rx_phys;
        state.set_att_mtu_desired(config.att_mtu_desired);
    });

    d.defuse();

    Ok(conn)
}

#[derive(Copy, Clone)]
pub struct Config {
    /// Requested ATT_MTU size for the next connection that is established.
    att_mtu_desired: u16,
    // bits of BLE_GAP_PHY_
    tx_phys: u8,
    // bits of BLE_GAP_PHY_
    rx_phys: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            att_mtu_desired: raw::BLE_GATT_ATT_MTU_DEFAULT as _,
            tx_phys: raw::BLE_GAP_PHY_AUTO as _,
            rx_phys: raw::BLE_GAP_PHY_AUTO as _,
        }
    }
}
