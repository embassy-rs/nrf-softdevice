//! Bluetooth Peripheral operations. Peripheral devices emit advertisements, and optionally accept connections from Central devices.

use core::ptr;

use crate::ble::*;
use crate::util::{get_union_field, OnDrop, Portal};
use crate::{raw, RawError, Softdevice};

struct RawAdvertisement<'a> {
    kind: u8,
    adv_data: Option<&'a [u8]>,
    scan_data: Option<&'a [u8]>,
    peer: Option<Address>,
    anonymous: bool,
    set_id: u8,
}

/// Connectable advertisement types, which can accept connections from interested Central devices.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConnectableAdvertisement<'a> {
    ScannableUndirected {
        adv_data: &'a [u8],
        scan_data: &'a [u8],
    },
    NonscannableDirected {
        peer: Address,
    },
    NonscannableDirectedHighDuty {
        peer: Address,
    },
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedNonscannableUndirected {
        set_id: u8,
        adv_data: &'a [u8],
    },
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedNonscannableDirected {
        set_id: u8,
        peer: Address,
        adv_data: &'a [u8],
    },
}

impl<'a> From<ConnectableAdvertisement<'a>> for RawAdvertisement<'a> {
    fn from(val: ConnectableAdvertisement<'a>) -> RawAdvertisement<'a> {
        match val {
            ConnectableAdvertisement::ScannableUndirected { adv_data, scan_data } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_CONNECTABLE_SCANNABLE_UNDIRECTED as u8,
                adv_data: Some(adv_data),
                scan_data: Some(scan_data),
                peer: None,
                anonymous: false,
                set_id: 0,
            },
            ConnectableAdvertisement::NonscannableDirected { peer } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_CONNECTABLE_NONSCANNABLE_DIRECTED as u8,
                adv_data: None,
                scan_data: None,
                peer: Some(peer),
                anonymous: false,
                set_id: 0,
            },
            ConnectableAdvertisement::NonscannableDirectedHighDuty { peer } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_CONNECTABLE_NONSCANNABLE_DIRECTED_HIGH_DUTY_CYCLE as u8,
                adv_data: None,
                scan_data: None,
                peer: Some(peer),
                anonymous: false,
                set_id: 0,
            },
            #[cfg(any(feature = "s132", feature = "s140"))]
            ConnectableAdvertisement::ExtendedNonscannableUndirected { adv_data, set_id } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_CONNECTABLE_NONSCANNABLE_UNDIRECTED as u8,
                adv_data: Some(adv_data),
                scan_data: None,
                peer: None,
                anonymous: false,
                set_id,
            },
            #[cfg(any(feature = "s132", feature = "s140"))]
            ConnectableAdvertisement::ExtendedNonscannableDirected { adv_data, peer, set_id } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_CONNECTABLE_NONSCANNABLE_DIRECTED as u8,
                adv_data: Some(adv_data),
                scan_data: None,
                peer: Some(peer),
                anonymous: false,
                set_id,
            },
        }
    }
}

/// Non-Connectable advertisement types. They cannot accept connections, they can be
/// only used to broadcast information in the air.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
        set_id: u8,
        scan_data: &'a [u8],
    },
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedScannableDirected {
        set_id: u8,
        peer: Address,
        scan_data: &'a [u8],
    },
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedNonscannableUndirected {
        set_id: u8,
        anonymous: bool,
        adv_data: &'a [u8],
    },
    #[cfg(any(feature = "s132", feature = "s140"))]
    ExtendedNonscannableDirected {
        set_id: u8,
        anonymous: bool,
        peer: Address,
        adv_data: &'a [u8],
    },
}

impl<'a> From<NonconnectableAdvertisement<'a>> for RawAdvertisement<'a> {
    fn from(val: NonconnectableAdvertisement<'a>) -> RawAdvertisement<'a> {
        match val {
            NonconnectableAdvertisement::ScannableUndirected { adv_data, scan_data } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_NONCONNECTABLE_SCANNABLE_UNDIRECTED as _,
                adv_data: Some(adv_data),
                scan_data: Some(scan_data),
                peer: None,
                anonymous: false,
                set_id: 0,
            },
            NonconnectableAdvertisement::NonscannableUndirected { adv_data } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_NONCONNECTABLE_NONSCANNABLE_UNDIRECTED as _,
                adv_data: Some(adv_data),
                scan_data: None,
                peer: None,
                anonymous: false,
                set_id: 0,
            },
            #[cfg(any(feature = "s132", feature = "s140"))]
            NonconnectableAdvertisement::ExtendedScannableUndirected { scan_data, set_id } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_NONCONNECTABLE_SCANNABLE_UNDIRECTED as _,
                adv_data: None,
                scan_data: Some(scan_data),
                peer: None,
                anonymous: false,
                set_id,
            },
            #[cfg(any(feature = "s132", feature = "s140"))]
            NonconnectableAdvertisement::ExtendedScannableDirected {
                scan_data,
                peer,
                set_id,
            } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_NONCONNECTABLE_SCANNABLE_DIRECTED as _,
                adv_data: None,
                scan_data: Some(scan_data),
                peer: Some(peer),
                anonymous: false,
                set_id,
            },
            #[cfg(any(feature = "s132", feature = "s140"))]
            NonconnectableAdvertisement::ExtendedNonscannableUndirected {
                adv_data,
                anonymous,
                set_id,
            } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_NONCONNECTABLE_NONSCANNABLE_UNDIRECTED as _,
                adv_data: Some(adv_data),
                scan_data: None,
                peer: None,
                anonymous,
                set_id,
            },
            #[cfg(any(feature = "s132", feature = "s140"))]
            NonconnectableAdvertisement::ExtendedNonscannableDirected {
                adv_data,
                peer,
                anonymous,
                set_id,
            } => RawAdvertisement {
                kind: raw::BLE_GAP_ADV_TYPE_EXTENDED_NONCONNECTABLE_NONSCANNABLE_DIRECTED as _,
                adv_data: Some(adv_data),
                scan_data: None,
                peer: Some(peer),
                anonymous,
                set_id,
            },
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
    let mut adv_params: raw::ble_gap_adv_params_t = unsafe { core::mem::zeroed() };

    adv_params.properties.type_ = adv.kind;
    adv_params.properties.set_anonymous(u8::from(adv.anonymous));

    adv_params.p_peer_addr = adv.peer.as_ref().map(|x| x.as_raw() as *const _).unwrap_or(ptr::null());
    adv_params.primary_phy = config.primary_phy as u8;
    adv_params.secondary_phy = config.secondary_phy as u8;
    adv_params.duration = config.timeout.map(|t| t.max(1)).unwrap_or(0);
    adv_params.max_adv_evts = config.max_events.map(|t| t.max(1)).unwrap_or(0);
    adv_params.interval = config.interval;
    adv_params.filter_policy = config.filter_policy as u8;
    adv_params.set_set_id(adv.set_id);
    // Unsupported: channel_mask and scan_req_notification

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

    let ret =
        unsafe { raw::sd_ble_gap_adv_set_configure(ptr::addr_of!(ADV_HANDLE) as _, &datas as _, &adv_params as _) };
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

    let ret = unsafe { raw::sd_ble_gap_adv_start(ADV_HANDLE, 1u8) };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gap_adv_start err {:?}", err);
        err
    })?;

    Ok(())
}

/// Perform non-connectable advertising.
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
                e => panic!("unexpected event {}", e),
            }
        })
        .await;

    d.defuse();
    res
}

/// Perform connectable advertising, returning the connection that's established as a result.
pub async fn advertise_connectable(
    sd: &Softdevice,
    adv: ConnectableAdvertisement<'_>,
    config: &Config,
) -> Result<Connection, AdvertiseError> {
    advertise_inner(sd, adv, config, Connection::new).await
}

#[cfg(feature = "ble-sec")]
pub async fn advertise_pairable<'a>(
    sd: &'a Softdevice,
    adv: ConnectableAdvertisement<'a>,
    config: &'a Config,
    security_handler: &'static dyn crate::ble::security::SecurityHandler,
) -> Result<Connection, AdvertiseError> {
    advertise_inner(sd, adv, config, |conn_handle, role, peer_address, conn_params| {
        Connection::with_security_handler(conn_handle, role, peer_address, conn_params, security_handler)
    })
    .await
}

async fn advertise_inner<'a, F>(
    _sd: &'a Softdevice,
    adv: ConnectableAdvertisement<'a>,
    config: &'a Config,
    mut f: F,
) -> Result<Connection, AdvertiseError>
where
    F: FnMut(u16, Role, Address, raw::ble_gap_conn_params_t) -> Result<Connection, OutOfConnsError>,
{
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

                    match f(conn_handle, role, peer_address, conn_params) {
                        Ok(conn) => Ok(conn),
                        Err(_) => {
                            raw::sd_ble_gap_disconnect(
                                conn_handle,
                                raw::BLE_HCI_REMOTE_DEV_TERMINATION_DUE_TO_LOW_RESOURCES as _,
                            );
                            Err(AdvertiseError::NoFreeConn)
                        }
                    }
                }
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => Err(AdvertiseError::Timeout),
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_SET_TERMINATED => Err(AdvertiseError::Timeout),
                e => panic!("unexpected event {}", e),
            }
        })
        .await;

    d.defuse();
    res
}

#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FilterPolicy {
    #[default]
    Any = raw::BLE_GAP_ADV_FP_ANY as u8,
    ScanRequests = raw::BLE_GAP_ADV_FP_FILTER_SCANREQ as u8,
    ConnectionRequests = raw::BLE_GAP_ADV_FP_FILTER_CONNREQ as u8,
    Both = raw::BLE_GAP_ADV_FP_FILTER_BOTH as u8,
}

#[derive(Copy, Clone)]
pub struct Config {
    pub primary_phy: Phy,
    pub secondary_phy: Phy,
    pub tx_power: TxPower,

    /// Timeout duration, in 10ms units
    pub timeout: Option<u16>,
    pub max_events: Option<u8>,

    /// Advertising interval, in 625us units
    pub interval: u32,

    pub filter_policy: FilterPolicy,
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
            filter_policy: FilterPolicy::default(),
        }
    }
}
