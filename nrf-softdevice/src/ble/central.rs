//! Bluetooth Central operations. Central devices scan for advertisements from Peripheral devices and connect to them.
//!
//! Typically the Central device is the higher-powered device, such as a smartphone or laptop, since scanning is more
//! power-hungry than advertising.

use core::{mem, ptr};

use crate::ble::types::*;
use crate::ble::{Address, Connection, OutOfConnsError};
use crate::util::{get_union_field, OnDrop, Portal};
use crate::{raw, RawError, Softdevice};

#[cfg(feature = "ble-gatt-client")]
use crate::ble::gatt_client::MtuExchangeError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConnectError {
    Timeout,
    NoAddresses,
    NoFreeConn,
    #[cfg(feature = "ble-gatt-client")]
    MtuExchange(MtuExchangeError),
    Raw(RawError),
}

#[cfg(feature = "ble-gatt-client")]
impl From<MtuExchangeError> for ConnectError {
    fn from(err: MtuExchangeError) -> Self {
        ConnectError::MtuExchange(err)
    }
}

impl From<RawError> for ConnectError {
    fn from(err: RawError) -> Self {
        ConnectError::Raw(err)
    }
}

pub(crate) static CONNECT_PORTAL: Portal<*const raw::ble_evt_t> = Portal::new();

pub async fn connect(sd: &Softdevice, config: &ConnectConfig<'_>) -> Result<Connection, ConnectError> {
    connect_inner(sd, config, Connection::new).await
}

#[cfg(feature = "ble-sec")]
pub async fn connect_with_security(
    sd: &Softdevice,
    config: &ConnectConfig<'_>,
    security_handler: &'static dyn crate::ble::security::SecurityHandler,
) -> Result<Connection, ConnectError> {
    connect_inner(sd, config, |conn_handle, role, peer_address, conn_params| {
        Connection::with_security_handler(conn_handle, role, peer_address, conn_params, security_handler)
    })
    .await
}

// Begins an ATT MTU exchange procedure, followed by a data length update request as necessary.
async fn connect_inner<F>(_sd: &Softdevice, config: &ConnectConfig<'_>, new_conn: F) -> Result<Connection, ConnectError>
where
    F: Fn(u16, Role, Address, raw::ble_gap_conn_params_t) -> Result<Connection, OutOfConnsError>,
{
    if let Some(w) = config.scan_config.whitelist {
        if w.len() == 0 {
            return Err(ConnectError::NoAddresses);
        }
    } else {
        return Err(ConnectError::NoAddresses);
    }

    let scan_params = config.scan_config.to_raw()?;

    let d = OnDrop::new(|| {
        let ret = unsafe { raw::sd_ble_gap_connect_cancel() };
        if let Err(_e) = RawError::convert(ret) {
            warn!("sd_ble_gap_connect_cancel: {:?}", _e);
        }
    });

    let ret = unsafe { raw::sd_ble_gap_connect(ptr::null(), &scan_params, &config.conn_params, 1) };
    if let Err(err) = RawError::convert(ret) {
        warn!("sd_ble_gap_connect err {:?}", err);
        return Err(err.into());
    }

    debug!("connect started");

    let conn = CONNECT_PORTAL
        .wait_once(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONNECTED => {
                    let gap_evt = get_union_field(ble_evt, &(*ble_evt).evt.gap_evt);
                    let params = &gap_evt.params.connected;
                    let conn_handle = gap_evt.conn_handle;
                    let role = Role::from_raw(params.role);
                    let peer_address = Address::from_raw(params.peer_addr);
                    let conn_params = params.conn_params;
                    debug!("connected role={:?} peer_addr={:?}", role, peer_address);

                    match new_conn(conn_handle, role, peer_address, conn_params) {
                        Ok(conn) => Ok(conn),
                        Err(_) => {
                            raw::sd_ble_gap_disconnect(
                                conn_handle,
                                raw::BLE_HCI_REMOTE_DEV_TERMINATION_DUE_TO_LOW_RESOURCES as _,
                            );
                            Err(ConnectError::NoFreeConn)
                        }
                    }
                }
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => Err(ConnectError::Timeout),
                e => panic!("unexpected event {}", e),
            }
        })
        .await?;

    d.defuse();

    #[cfg(feature = "ble-gatt-client")]
    {
        let mtu = config.att_mtu.unwrap_or(_sd.att_mtu);
        crate::ble::gatt_client::att_mtu_exchange(&conn, mtu).await?;
    }

    Ok(conn)
}

#[derive(Copy, Clone)]
pub struct ConnectConfig<'a> {
    /// Requested ATT_MTU size for the next connection that is established.
    #[cfg(feature = "ble-gatt-client")]
    pub att_mtu: Option<u16>,

    pub scan_config: ScanConfig<'a>,
    pub conn_params: raw::ble_gap_conn_params_t,
}

impl<'a> Default for ConnectConfig<'a> {
    fn default() -> Self {
        Self {
            #[cfg(feature = "ble-gatt-client")]
            att_mtu: None,
            scan_config: ScanConfig::default(),
            conn_params: raw::ble_gap_conn_params_t {
                min_conn_interval: 40,
                max_conn_interval: 200,
                slave_latency: 0,
                conn_sup_timeout: 400, // 4s
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

pub async fn scan<'a, F, R>(_sd: &Softdevice, config: &ScanConfig<'a>, mut f: F) -> Result<R, ScanError>
where
    F: for<'b> FnMut(&'b raw::ble_gap_evt_adv_report_t) -> Option<R>,
{
    let scan_params = config.to_raw()?;

    // Buffer to store received advertisement data.
    const BUF_LEN: usize = 256;

    // Both of these are intentionally static because Softdevice will,
    // sometimes, write to the buffer after scan_stop() has been
    // called, somewhere around evt_get().
    //
    // This can result in UB as a use-after-free, given the buffer
    // has been dropped and the scanning has been stopped.
    static mut BUF: [u8; BUF_LEN] = [0u8; BUF_LEN];
    static mut BUF_DATA: raw::ble_data_t = raw::ble_data_t {
        p_data: unsafe { (&mut *(&raw mut BUF)).as_mut_ptr() },
        len: BUF_LEN as u16,
    };

    let ret = unsafe { raw::sd_ble_gap_scan_start(&scan_params, ptr::addr_of!(BUF_DATA)) };
    match RawError::convert(ret) {
        Ok(()) => {}
        Err(err) => {
            warn!("sd_ble_gap_scan_start err {:?}", err);
            return Err(ScanError::Raw(err));
        }
    }

    let _d = OnDrop::new(|| {
        let ret = unsafe { raw::sd_ble_gap_scan_stop() };
        match RawError::convert(ret) {
            Ok(_) => {}
            Err(RawError::InvalidState) => {} // scan stopped itself due to timeout, erroring is normal.
            Err(_e) => warn!("sd_ble_gap_scan_stop: {:?}", _e),
        }
    });

    debug!("Scan started");
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
                    let ret = raw::sd_ble_gap_scan_start(ptr::null(), ptr::addr_of!(BUF_DATA));
                    match RawError::convert(ret) {
                        Ok(()) => {}

                        // "The scanner has timed out when this function is called to continue scanning"
                        Err(RawError::InvalidState) => return Some(Err(ScanError::Timeout)),

                        Err(err) => {
                            warn!("sd_ble_gap_scan_start resume err {:?}", err);
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
    /// Whitelist of addresses to scan. If None, all advertisements
    /// will be processed when scanning.
    ///
    /// For connecting this must be Some, and have least 1 address.
    pub whitelist: Option<&'a [&'a Address]>,

    /// Support extended advertisements.
    ///
    /// If true, the scanner will accept extended advertising packets.
    /// If false, the scanner will not receive advertising packets
    /// on secondary advertising channels, and will not be able
    /// to receive long advertising PDUs.
    pub extended: bool,

    /// If true, scan actively by sending scan requests.
    /// Ignored when using for connecting.
    pub active: bool,

    /// Set of PHYs to scan
    pub phys: PhySet,

    /// Scan interval, in units of 625us
    pub interval: u32,

    /// Scan window, in units of 625us
    pub window: u32,

    /// Timeout in units of 10ms. If set to 0, scan forever.
    pub timeout: u16,

    /// Radio TX power. This is used for scanning, and is inherited
    /// as the connection TX power if this ScanConfig is used for connect().
    pub tx_power: TxPower,
}

impl<'a> Default for ScanConfig<'a> {
    fn default() -> Self {
        Self {
            extended: true,
            active: true,
            phys: PhySet::M1,
            interval: 2732,
            window: 500,
            timeout: raw::BLE_GAP_SCAN_TIMEOUT_UNLIMITED as _,
            whitelist: None,
            tx_power: TxPower::ZerodBm,
        }
    }
}

impl<'a> ScanConfig<'a> {
    fn to_raw(&self) -> Result<raw::ble_gap_scan_params_t, RawError> {
        let mut scan_params: raw::ble_gap_scan_params_t = unsafe { mem::zeroed() };
        if self.extended {
            scan_params.set_extended(1);
        }
        if self.active {
            scan_params.set_active(1);
        }
        scan_params.scan_phys = self.phys as u8;
        scan_params.timeout = self.timeout;

        // s122 has these in us instead of 625us :shrug:
        #[cfg(not(feature = "s122"))]
        {
            scan_params.interval = self.interval as u16;
            scan_params.window = self.window as u16;
        }
        #[cfg(feature = "s122")]
        {
            scan_params.interval_us = self.interval * 625;
            scan_params.window_us = self.window * 625;
        }

        // Set whitelist
        if let Some(w) = self.whitelist {
            assert!(w.len() <= u8::MAX as usize);
            let ret = unsafe { raw::sd_ble_gap_whitelist_set(w.as_ptr() as _, w.len() as u8) };
            if let Err(err) = RawError::convert(ret) {
                warn!("sd_ble_gap_whitelist_set err {:?}", err);
                return Err(err.into());
            }
            scan_params.set_filter_policy(raw::BLE_GAP_SCAN_FP_WHITELIST as _);
        } else {
            scan_params.set_filter_policy(raw::BLE_GAP_SCAN_FP_ACCEPT_ALL as _);
        }

        // Set tx power
        let ret = unsafe {
            raw::sd_ble_gap_tx_power_set(
                raw::BLE_GAP_TX_POWER_ROLES_BLE_GAP_TX_POWER_ROLE_SCAN_INIT as _,
                0,
                self.tx_power as i8,
            )
        };
        RawError::convert(ret).map_err(|err| {
            warn!("sd_ble_gap_tx_power_set err {:?}", err);
            err
        })?;

        Ok(scan_params)
    }
}
