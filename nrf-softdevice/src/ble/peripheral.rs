//! Bluetooth Peripheral operations. Peripheral devices emit advertisements, and optionally accept connections from Central devices.

use core::mem;
use core::ptr;

use crate::ble::*;
use crate::raw;
use crate::util::get_union_field;
use crate::util::{OnDrop, Portal};
use crate::{RawError, Softdevice};

struct RawAdvertisement<'a> {
    kind: u8,
    adv_data: Option<&'a [u8]>,
    scan_data: Option<&'a [u8]>,
}

/// Connectable advertisement types, which can accept connections from interested Central devices.
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
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedNonscannableUndirected {
        adv_data: &'a [u8],
    },
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedNonscannableDirected {
        adv_data: &'a [u8],
    },
}

impl<'a> From<ConnectableAdvertisement<'a>> for RawAdvertisement<'a> {
    fn from(val: ConnectableAdvertisement<'a>) -> RawAdvertisement<'a> {
        match val {
            ConnectableAdvertisement::ScannableUndirected {
                adv_data,
                scan_data,
            } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_CONNECTABLE_SCANNABLE_UNDIRECTED as u8,
                adv_data: Some(adv_data),
                scan_data: Some(scan_data),
            },
            ConnectableAdvertisement::NonscannableDirected { scan_data } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_CONNECTABLE_NONSCANNABLE_DIRECTED as u8,
                adv_data: None,
                scan_data: Some(scan_data),
            },
            ConnectableAdvertisement::NonscannableDirectedHighDuty { scan_data } => {
                RawAdvertisement {
                    kind: raw::BLE_GAP_ADV_TYPE_CONNECTABLE_NONSCANNABLE_DIRECTED_HIGH_DUTY_CYCLE
                        as u8,
                    adv_data: None,
                    scan_data: Some(scan_data),
                }
            }
            #[cfg(any(feature = "s132", feature = "s140"))]
            ConnectableAdvertisement::ExtendedNonscannableUndirected { adv_data } => {
                RawAdvertisement {
                    kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_CONNECTABLE_NONSCANNABLE_UNDIRECTED as u8,
                    adv_data: Some(adv_data),
                    scan_data: None,
                }
            }
            #[cfg(any(feature = "s132", feature = "s140"))]
            ConnectableAdvertisement::ExtendedNonscannableDirected { adv_data } => {
                RawAdvertisement {
                    kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_CONNECTABLE_NONSCANNABLE_DIRECTED as u8,
                    adv_data: Some(adv_data),
                    scan_data: None,
                }
            }
        }
    }
}

/// Non-Connectable advertisement types. They cannot accept connections, they can be
/// only used to broadcast information in the air.
pub enum NonconnectableAdvertisement<'a> {
    ScannableUndirected {
        adv_data: &'a [u8],
        scan_data: &'a [u8],
    },
    NonscannableUndirected {
        adv_data: &'a [u8],
    },
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedScannableUndirected {
        adv_data: &'a [u8],
        scan_data: &'a [u8],
    },
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedScannableDirected {
        adv_data: &'a [u8],
        scan_data: &'a [u8],
    },
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedNonscannableUndirected {
        adv_data: &'a [u8],
    },
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedNonscannableDirected {
        adv_data: &'a [u8],
    },
}

impl<'a> From<NonconnectableAdvertisement<'a>> for RawAdvertisement<'a> {
    fn from(val: NonconnectableAdvertisement<'a>) -> RawAdvertisement<'a> {
        match val {
            NonconnectableAdvertisement::ScannableUndirected {
                adv_data,
                scan_data,
            } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_NONCONNECTABLE_SCANNABLE_UNDIRECTED as _,
                adv_data: Some(adv_data),
                scan_data: Some(scan_data),
            },
            NonconnectableAdvertisement::NonscannableUndirected { adv_data } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_NONCONNECTABLE_NONSCANNABLE_UNDIRECTED as _,
                adv_data: Some(adv_data),
                scan_data: None,
            },
            #[cfg(any(feature = "s132", feature = "s140"))]
            NonconnectableAdvertisement::ExtendedScannableUndirected {
                adv_data,
                scan_data,
            } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_NONCONNECTABLE_SCANNABLE_UNDIRECTED as _,
                adv_data: Some(adv_data),
                scan_data: Some(scan_data),
            },
            #[cfg(any(feature = "s132", feature = "s140"))]
            NonconnectableAdvertisement::ExtendedScannableDirected {
                adv_data,
                scan_data,
            } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_NONCONNECTABLE_SCANNABLE_DIRECTED as _,
                adv_data: Some(adv_data),
                scan_data: Some(scan_data),
            },
            #[cfg(any(feature = "s132", feature = "s140"))]
            NonconnectableAdvertisement::ExtendedNonscannableUndirected { adv_data } => {
                RawAdvertisement {
                    kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_NONCONNECTABLE_NONSCANNABLE_UNDIRECTED
                        as _,
                    adv_data: Some(adv_data),
                    scan_data: None,
                }
            }
            #[cfg(any(feature = "s132", feature = "s140"))]
            NonconnectableAdvertisement::ExtendedNonscannableDirected { adv_data } => {
                RawAdvertisement {
                    kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_NONCONNECTABLE_NONSCANNABLE_DIRECTED as _,
                    adv_data: Some(adv_data),
                    scan_data: None,
                }
            }
        }
    }
}

/// Error for [`advertise_start`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdvertiseError {
    Timeout,
    NoFreeConn,
    Raw(RawError),
}

impl From<RawError> for AdvertiseError {
    fn from(err: RawError) -> Self {
        AdvertiseError::Raw(err)
    }
}

static mut ADV_HANDLE: u8 = raw::BLE_GAP_ADV_SET_HANDLE_NOT_SET as u8;
pub(crate) static ADV_PORTAL: Portal<*const raw::ble_evt_t> = Portal::new();

fn start_adv(adv: RawAdvertisement<'_>, config: &Config) -> Result<(), AdvertiseError> {
    // TODO make these configurable, only the right params based on type?
    let mut adv_params: raw::ble_gap_adv_params_t = unsafe { mem::zeroed() };
    adv_params.properties.type_ = adv.kind;
    adv_params.primary_phy = config.primary_phy as u8;
    adv_params.secondary_phy = config.secondary_phy as u8;
    adv_params.duration = config.timeout.map(|t| t.max(1)).unwrap_or(0);
    adv_params.max_adv_evts = config.max_events.map(|t| t.max(1)).unwrap_or(0);
    adv_params.interval = config.interval;

    let map_data = |data: Option<&[u8]>| {
        if let Some(data) = data {
            assert!(data.len() < u16::MAX as usize);
            raw::ble_data_t {
                p_data: data.as_ptr() as _,
                len: data.len() as u16,
            }
        } else {
            raw::ble_data_t {
                p_data: ptr::null_mut(),
                len: 0,
            }
        }
    };

    let datas = raw::ble_gap_adv_data_t {
        adv_data: map_data(adv.adv_data),
        scan_rsp_data: map_data(adv.scan_data),
    };

    let ret = unsafe {
        raw::sd_ble_gap_adv_set_configure(&mut ADV_HANDLE as _, &datas as _, &adv_params as _)
    };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gap_adv_set_configure err {:?}", err);
        err
    })?;

    let ret = unsafe {
        raw::sd_ble_gap_tx_power_set(
            raw::BLE_GAP_TX_POWER_ROLES_BLE_GAP_TX_POWER_ROLE_ADV as _,
            ADV_HANDLE as _,
            config.tx_power as i8,
        )
    };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gap_tx_power_set err {:?}", err);
        err
    })?;

    let ret = unsafe { raw::sd_ble_gap_adv_start(ADV_HANDLE, 1 as u8) };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gap_adv_start err {:?}", err);
        err
    })?;

    Ok(())
}

/// Perform connectable advertising, returning the connection that's established as a result.
pub async fn advertise(
    _sd: &Softdevice,
    adv: NonconnectableAdvertisement<'_>,
    config: &Config,
) -> Result<(), AdvertiseError> {
    let d = OnDrop::new(|| {
        let ret = unsafe { raw::sd_ble_gap_adv_stop(ADV_HANDLE) };
        if let Err(_e) = RawError::convert(ret) {
            warn!("sd_ble_gap_adv_stop: {:?}", _e);
        }
    });

    start_adv(adv.into(), config)?;

    // The advertising data needs to be kept alive for the entire duration of the advertising procedure.
    let res = ADV_PORTAL
        .wait_once(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => Err(AdvertiseError::Timeout),
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_SET_TERMINATED => Err(AdvertiseError::Timeout),
                _ => unreachable!(),
            }
        })
        .await;

    d.defuse();
    res
}

/// Perform connectable advertising, returning the connection that's established as a result.
pub async fn advertise_connectable(
    _sd: &Softdevice,
    adv: ConnectableAdvertisement<'_>,
    config: &Config,
) -> Result<Connection, AdvertiseError> {
    let d = OnDrop::new(|| {
        let ret = unsafe { raw::sd_ble_gap_adv_stop(ADV_HANDLE) };
        if let Err(_e) = RawError::convert(ret) {
            warn!("sd_ble_gap_adv_stop: {:?}", _e);
        }
    });

    start_adv(adv.into(), config)?;

    // The advertising data needs to be kept alive for the entire duration of the advertising procedure.
    let res = ADV_PORTAL
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

                    match Connection::new(conn_handle, role, peer_address, conn_params) {
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
                            Err(AdvertiseError::NoFreeConn)
                        }
                    }
                }
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => Err(AdvertiseError::Timeout),
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_SET_TERMINATED => Err(AdvertiseError::Timeout),
                _ => unreachable!(),
            }
        })
        .await;

    d.defuse();
    res
}

#[derive(Copy, Clone)]
pub struct Config {
    pub primary_phy: Phy,
    pub secondary_phy: Phy,
    pub tx_power: TxPower,

    /// Timeout duration, in 10ms units
    pub timeout: Option<u16>,
    pub max_events: Option<u8>,

    /// Advertising interval, in 0.625us units
    pub interval: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            primary_phy: Phy::M1,
            secondary_phy: Phy::M1,
            tx_power: TxPower::ZerodBm,
            timeout: None,
            max_events: None,
            interval: 400, // 250ms
        }
    }
}
