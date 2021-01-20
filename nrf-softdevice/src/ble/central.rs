//! Bluetooth Central operations. Central devices scan for advertisements from Peripheral devices and connect to them.
//!
//! Typically the Central device is the higher-powered device, such as a smartphone or laptop, since scanning is more
//! power-hungry than advertising.

use core::mem;
use core::ptr;

use crate::ble::gap;
use crate::ble::types::*;
use crate::ble::{Address, Connection};
use crate::fmt::{assert, unreachable, *};
use crate::raw;
use crate::util::{get_union_field, OnDrop, Portal};
use crate::{RawError, Softdevice};

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConnectError {
    Timeout,
    NoAddresses,
    NoFreeConn,
    Raw(RawError),
}

impl From<RawError> for ConnectError {
    fn from(err: RawError) -> Self {
        ConnectError::Raw(err)
    }
}

pub(crate) static CONNECT_PORTAL: Portal<*const raw::ble_evt_t> = Portal::new();

// Begins an ATT MTU exchange procedure, followed by a data length update request as necessary.
pub async fn connect(
    sd: &Softdevice,
    addresses: &[&Address],
    config: &Config,
) -> Result<Connection, ConnectError> {
    if addresses.len() == 0 {
        return Err(ConnectError::NoAddresses);
    }

    // Set tx power
    let ret = unsafe {
        raw::sd_ble_gap_tx_power_set(
            raw::BLE_GAP_TX_POWER_ROLES_BLE_GAP_TX_POWER_ROLE_SCAN_INIT as _,
            0,
            config.tx_power as i8,
        )
    };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gap_tx_power_set err {:?}", err);
        err
    })?;

    let mut scan_params = raw::ble_gap_scan_params_t::from(&config.scan_params);
    scan_params.set_filter_policy(raw::BLE_GAP_SCAN_FP_WHITELIST as _);

    let d = OnDrop::new(|| {
        let ret = unsafe { raw::sd_ble_gap_connect_cancel() };
        if let Err(e) = RawError::convert(ret) {
            warn!("sd_ble_gap_connect_cancel: {:?}", e);
        }
    });

    assert!(addresses.len() <= u8::MAX as usize);
    let ret =
        unsafe { raw::sd_ble_gap_whitelist_set(addresses.as_ptr() as _, addresses.len() as u8) };
    if let Err(err) = RawError::convert(ret) {
        warn!("sd_ble_gap_connect err {:?}", err);
        return Err(err.into());
    }

    let ret =
        unsafe { raw::sd_ble_gap_connect(ptr::null(), &mut scan_params, &config.conn_params, 1) };
    if let Err(err) = RawError::convert(ret) {
        warn!("sd_ble_gap_connect err {:?}", err);
        return Err(err.into());
    }

    info!("connect started");

    let conn = CONNECT_PORTAL
        .wait_once(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONNECTED => {
                    let gap_evt = get_union_field(ble_evt, &(*ble_evt).evt.gap_evt);
                    let params = &gap_evt.params.connected;
                    let conn_handle = gap_evt.conn_handle;
                    let role = Role::from_raw(params.role);
                    let peer_address = Address::from_raw(params.peer_addr);
                    debug!("connected role={:?} peer_addr={:?}", role, peer_address);

                    match Connection::new(conn_handle, role, peer_address) {
                        Ok(conn) => {
                            #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
                            gap::do_data_length_update(conn_handle, ptr::null());

                            Ok(conn)
                        }
                        Err(_) => {
                            raw::sd_ble_gap_disconnect(
                                conn_handle,
                                raw::BLE_HCI_REMOTE_USER_TERMINATED_CONNECTION as _,
                            );
                            Err(ConnectError::NoFreeConn)
                        }
                    }
                }
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => Err(ConnectError::Timeout),
                _ => unreachable!(),
            }
        })
        .await?;

    conn.with_state(|state| {
        state.rx_phys = config.tx_phys;
        state.tx_phys = config.rx_phys;
    });

    d.defuse();

    #[cfg(feature = "ble-gatt-client")]
    {
        let mtu = config.att_mtu.unwrap_or(sd.att_mtu);
        unwrap!(crate::ble::gatt_client::att_mtu_exchange(&conn, mtu).await);
    }

    Ok(conn)
}

#[derive(Copy, Clone)]
pub struct Config {
    pub tx_power: TxPower,

    /// Requested ATT_MTU size for the next connection that is established.
    #[cfg(feature = "ble-gatt-client")]
    pub att_mtu: Option<u16>,
    // bits of BLE_GAP_PHY_
    pub tx_phys: u8,
    // bits of BLE_GAP_PHY_
    pub rx_phys: u8,

    pub scan_params: ScanParams,
    pub conn_params: raw::ble_gap_conn_params_t,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tx_power: TxPower::ZerodBm,
            #[cfg(feature = "ble-gatt-client")]
            att_mtu: None,
            tx_phys: raw::BLE_GAP_PHY_AUTO as _,
            rx_phys: raw::BLE_GAP_PHY_AUTO as _,
            scan_params: ScanParams::default(),
            conn_params: raw::ble_gap_conn_params_t {
                min_conn_interval: 40,
                max_conn_interval: 200,
                slave_latency: 0,
                conn_sup_timeout: 400, // 4s
            },
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ScanError {
    Timeout,
    Raw(RawError),
}

impl From<RawError> for ScanError {
    fn from(err: RawError) -> Self {
        ScanError::Raw(err)
    }
}

pub(crate) static SCAN_PORTAL: Portal<*const raw::ble_evt_t> = Portal::new();

pub async fn scan<'a, F, R>(
    _sd: &Softdevice,
    config: &ScanConfig<'a>,
    mut f: F,
) -> Result<R, ScanError>
where
    F: for<'b> FnMut(&'b raw::ble_gap_evt_adv_report_t) -> Option<R>,
{
    // Set tx power
    let ret = unsafe {
        raw::sd_ble_gap_tx_power_set(
            raw::BLE_GAP_TX_POWER_ROLES_BLE_GAP_TX_POWER_ROLE_SCAN_INIT as _,
            0,
            config.tx_power as i8,
        )
    };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gap_tx_power_set err {:?}", err);
        err
    })?;

    let mut scan_params = raw::ble_gap_scan_params_t::from(&config.scan_params);

    // Buffer to store received advertisement data.
    const BUF_LEN: usize = 256;
    let mut buf = [0u8; BUF_LEN];
    let buf_data = raw::ble_data_t {
        p_data: buf.as_mut_ptr(),
        len: BUF_LEN as u16,
    };

    let ret = unsafe { raw::sd_ble_gap_scan_start(&scan_params, &buf_data) };
    match RawError::convert(ret) {
        Ok(()) => {}
        Err(err) => {
            warn!("sd_ble_gap_scan_start err {:?}", err);
            return Err(ScanError::Raw(err));
        }
    }

    let _d = OnDrop::new(|| {
        let ret = unsafe { raw::sd_ble_gap_scan_stop() };
        if let Err(e) = RawError::convert(ret) {
            warn!("sd_ble_gap_scan_stop: {:?}", e);
        }
    });

    info!("Scan started");
    let res = SCAN_PORTAL
        .wait_many(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => return Some(Err(ScanError::Timeout)),
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_REPORT => {
                    let gap_evt = get_union_field(ble_evt, &(*ble_evt).evt.gap_evt);
                    let params = &gap_evt.params.adv_report;
                    if let Some(r) = f(params) {
                        return Some(Ok(r));
                    }

                    // Resume scan
                    let ret = raw::sd_ble_gap_scan_start(ptr::null(), &buf_data);
                    match RawError::convert(ret) {
                        Ok(()) => {}
                        Err(err) => {
                            warn!("sd_ble_gap_scan_start err {:?}", err);
                            return Some(Err(ScanError::Raw(err)));
                        }
                    };
                    None
                }
                _ => None,
            }
        })
        .await?;

    Ok(res)
}

#[derive(Copy, Clone)]
pub struct ScanConfig<'a> {
    pub whitelist: Option<&'a [Address]>,
    pub tx_power: TxPower,
    pub scan_params: ScanParams,
}

impl<'a> Default for ScanConfig<'a> {
    fn default() -> Self {
        Self {
            whitelist: None,
            tx_power: TxPower::ZerodBm,
            scan_params: ScanParams::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct ScanParams {
    pub extended: bool,
    pub active: bool,
    pub filter_policy: u8,
    pub scan_phys: u8,
    pub interval: u32,
    pub window: u32,
    pub timeout: u16,
}
impl From<&ScanParams> for raw::ble_gap_scan_params_t {
    fn from(res: &ScanParams) -> raw::ble_gap_scan_params_t {
        let mut scan_params: raw::ble_gap_scan_params_t = unsafe { mem::zeroed() };
        if res.extended {
            scan_params.set_extended(1);
        }
        if res.active {
            scan_params.set_active(1);
        }
        scan_params.set_filter_policy(res.filter_policy);
        scan_params.scan_phys = res.scan_phys;
        scan_params.timeout = res.timeout;

        // s122 has these in us instead of 625us :shrug:
        #[cfg(not(feature = "s122"))]
        {
            scan_params.interval = res.interval as u16;
            scan_params.window = res.window as u16;
        }
        #[cfg(feature = "s122")]
        {
            scan_params.interval_us = res.interval * 625;
            scan_params.window_us = res.window * 625;
        }
        return scan_params;
    }
}
impl Default for ScanParams {
    fn default() -> Self {
        Self {
            extended: true,
            active: true,
            filter_policy: raw::BLE_GAP_SCAN_FP_ACCEPT_ALL as _,
            scan_phys: raw::BLE_GAP_PHY_1MBPS as _,
            interval: 2732,
            window: 500,
            timeout: raw::BLE_GAP_SCAN_TIMEOUT_UNLIMITED as _,
        }
    }
}
