use core::cell::{Cell, UnsafeCell};
use core::iter::FusedIterator;

use raw::ble_gap_conn_params_t;

use super::{HciStatus, PhySet};
#[cfg(feature = "ble-central")]
use crate::ble::gap::default_security_params;
#[cfg(feature = "ble-sec")]
use crate::ble::security::SecurityHandler;
use crate::ble::types::{Address, AddressType, Role, SecurityMode};
use crate::util::get_union_field;
use crate::{raw, RawError};

#[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
const BLE_GAP_DATA_LENGTH_DEFAULT: u8 = 27; //  The stack's default data length. <27-251>

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct OutOfConnsError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DisconnectedError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) enum ConnHandleState {
    Disconnected(HciStatus),
    Connected(u16),
}

impl ConnHandleState {
    const fn to_result(self) -> Result<u16, DisconnectedError> {
        match self {
            ConnHandleState::Disconnected(_) => Err(DisconnectedError),
            ConnHandleState::Connected(handle) => Ok(handle),
        }
    }

    const fn handle(self) -> Option<u16> {
        match self {
            ConnHandleState::Disconnected(_) => None,
            ConnHandleState::Connected(handle) => Some(handle),
        }
    }

    const fn is_connected(&self) -> bool {
        matches!(self, ConnHandleState::Connected(_))
    }

    const fn disconnect_reason(self) -> Option<HciStatus> {
        match self {
            ConnHandleState::Disconnected(reason) => Some(reason),
            ConnHandleState::Connected(_) => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SetConnParamsError {
    Disconnected,
    Raw(RawError),
}

impl From<DisconnectedError> for SetConnParamsError {
    fn from(_err: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

impl From<RawError> for SetConnParamsError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(feature = "ble-peripheral")]
pub enum IgnoreSlaveLatencyError {
    Disconnected,
    Raw(RawError),
}

#[cfg(feature = "ble-peripheral")]
impl From<DisconnectedError> for IgnoreSlaveLatencyError {
    fn from(_err: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

#[cfg(feature = "ble-peripheral")]
impl From<RawError> for IgnoreSlaveLatencyError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

#[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
#[derive(Debug, Clone, Copy)]
pub enum DataLengthUpdateError {
    Disconnected,
    NotSupported(raw::ble_gap_data_length_limitation_t),
    Resources(raw::ble_gap_data_length_limitation_t),
    Raw(RawError),
}

#[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
impl From<DisconnectedError> for DataLengthUpdateError {
    fn from(_err: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

pub enum PhyUpdateError {
    Disconnected,
    Raw(RawError),
}

impl From<DisconnectedError> for PhyUpdateError {
    fn from(_err: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

impl From<RawError> for PhyUpdateError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(any(feature = "ble-central", feature = "ble-peripheral"))]
pub enum AuthenticateError {
    Disconnected,
    Raw(RawError),
}

#[cfg(any(feature = "ble-central", feature = "ble-peripheral"))]
impl From<DisconnectedError> for AuthenticateError {
    fn from(_err: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

#[cfg(any(feature = "ble-central", feature = "ble-peripheral"))]
impl From<RawError> for AuthenticateError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg(all(feature = "ble-central", feature = "ble-sec"))]
pub enum EncryptError {
    Disconnected,
    NoSecurityHandler,
    PeerKeysNotFound,
    Raw(RawError),
}

#[cfg(all(feature = "ble-central", feature = "ble-sec"))]
impl From<DisconnectedError> for EncryptError {
    fn from(_err: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

#[cfg(all(feature = "ble-central", feature = "ble-sec"))]
impl From<RawError> for EncryptError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

// Highest ever the softdevice can support.
pub(crate) const CONNS_MAX: usize = 20;

#[cfg(feature = "ble-sec")]
#[derive(Clone, Copy)]
pub(crate) struct EncryptionState {
    pub handler: Option<&'static dyn SecurityHandler>,

    pub own_enc_key: raw::ble_gap_enc_key_t,
    pub peer_enc_key: raw::ble_gap_enc_key_t,
    pub peer_id: raw::ble_gap_id_key_t,
}

#[cfg(feature = "ble-sec")]
const NEW_GAP_ENC_KEY: raw::ble_gap_enc_key_t = raw::ble_gap_enc_key_t {
    enc_info: raw::ble_gap_enc_info_t {
        ltk: [0; 16],
        _bitfield_1: raw::__BindgenBitfieldUnit::new([0; 1]),
    },
    master_id: raw::ble_gap_master_id_t { ediv: 0, rand: [0; 8] },
};

#[cfg(feature = "ble-sec")]
const NEW_GAP_ID_KEY: raw::ble_gap_id_key_t = raw::ble_gap_id_key_t {
    id_info: raw::ble_gap_irk_t { irk: [0; 16] },
    id_addr_info: raw::ble_gap_addr_t {
        _bitfield_1: raw::__BindgenBitfieldUnit::new([0; 1]),
        addr: [0; 6],
    },
};

#[cfg(feature = "ble-sec")]
const NEW_ENCRYPTION_STATE: EncryptionState = EncryptionState {
    handler: None,
    own_enc_key: NEW_GAP_ENC_KEY,
    peer_enc_key: NEW_GAP_ENC_KEY,
    peer_id: NEW_GAP_ID_KEY,
};

// We could make the public Connection type simply hold the softdevice's conn_handle.
// However, that would allow for bugs like:
// - Connection is established with conn_handle=5
// - Client code stores a Connection instance with conn_handle=5
// - Connection gets disconnected
// - A new, unrelated connection is established with same conn_handle=5
// - Client code uses the Connection instance from before, mistakenly doing an operation in an unrelated connection.
//
// To avoid this, the public Connection struct has an "index" into a private ConnectionState array.
// It is refcounted, so an index will never be reused until client code has dropped all Connection instances.

pub(crate) struct ConnectionState {
    // Every Connection instance counts as one ref.
    //
    // When client code drops all instances, refcount becomes 0 and disconnection is initiated.
    // However, disconnection is not complete until the event GAP_DISCONNECTED.
    // so there's a small gap of time where the ConnectionState is not "free" even if refcount=0.
    pub refcount: u8,
    pub conn_handle: ConnHandleState,

    pub disconnecting: bool,
    pub role: Role,
    pub peer_address: Address,
    pub security_mode: SecurityMode,

    pub conn_params: ble_gap_conn_params_t,

    #[cfg(feature = "ble-rssi")]
    pub rssi: Option<i8>,

    #[cfg(feature = "ble-gatt")]
    pub att_mtu: u16, // Effective ATT_MTU size (in bytes).
    #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
    pub data_length_effective: u8, // Effective data length (in bytes).

    #[cfg(feature = "ble-sec")]
    pub security: EncryptionState,
}

impl ConnectionState {
    const fn dummy() -> Self {
        // The returned value should have bit pattern 0, so that STATES
        // can go into .bss instead of .data, which saves flash space.
        Self {
            refcount: 0,
            conn_handle: ConnHandleState::Disconnected(HciStatus::SUCCESS),
            #[cfg(feature = "ble-central")]
            role: Role::Central,
            #[cfg(not(feature = "ble-central"))]
            role: Role::Peripheral,
            peer_address: Address::new(AddressType::Public, [0; 6]),
            security_mode: SecurityMode::NoAccess,
            disconnecting: false,
            conn_params: ble_gap_conn_params_t {
                conn_sup_timeout: 0,
                max_conn_interval: 0,
                min_conn_interval: 0,
                slave_latency: 0,
            },
            #[cfg(feature = "ble-rssi")]
            rssi: None,
            #[cfg(feature = "ble-gatt")]
            att_mtu: 0,
            #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
            data_length_effective: 0,
            #[cfg(feature = "ble-sec")]
            security: NEW_ENCRYPTION_STATE,
        }
    }
    pub(crate) fn check_connected(&mut self) -> Result<u16, DisconnectedError> {
        self.conn_handle.to_result()
    }

    pub(crate) fn disconnect(&mut self) -> Result<(), DisconnectedError> {
        self.disconnect_with_reason(HciStatus::REMOTE_USER_TERMINATED_CONNECTION)
    }

    pub(crate) fn disconnect_with_reason(&mut self, reason: HciStatus) -> Result<(), DisconnectedError> {
        let conn_handle = self.check_connected()?;

        if self.disconnecting {
            return Ok(());
        }

        let ret = unsafe { raw::sd_ble_gap_disconnect(conn_handle, reason.into()) };
        // This case can occur in normal operation if e.g. the connection was
        // already being disconnected when disconnect() is called, so
        // return as error instead of a panic.
        if ret == raw::NRF_ERROR_INVALID_STATE {
            return Err(DisconnectedError);
        }
        unwrap!(RawError::convert(ret), "sd_ble_gap_disconnect");

        self.disconnecting = true;
        Ok(())
    }

    pub(crate) fn on_disconnected(&mut self, ble_evt: *const raw::ble_evt_t) {
        let conn_handle = unwrap!(
            self.conn_handle.handle(),
            "bug: on_disconnected when already disconnected"
        );

        let ibh = index_by_handle(conn_handle);
        let _index = unwrap!(ibh.get(), "bug: conn_handle has no index");

        #[cfg(all(feature = "ble-gatt-server", feature = "ble-sec"))]
        if let Some(handler) = self.security.handler {
            let conn = unwrap!(Connection::from_handle(conn_handle), "bug: conn_handle has no index");
            handler.save_sys_attrs(&conn);
        }

        ibh.set(None);

        let reason = unsafe {
            assert_eq!(
                u32::from((*ble_evt).header.evt_id),
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED,
                "bug: on_disconnected called with non-disconnect event"
            );
            let gap_evt = get_union_field(ble_evt, &(*ble_evt).evt.gap_evt);
            HciStatus::new(gap_evt.params.disconnected.reason)
        };
        self.conn_handle = ConnHandleState::Disconnected(reason);

        // Signal possible in-progess operations that the connection has disconnected.
        #[cfg(feature = "ble-gatt-client")]
        crate::ble::gatt_client::portal(conn_handle).call(ble_evt);
        #[cfg(feature = "ble-gatt-client")]
        crate::ble::gatt_client::hvx_portal(conn_handle).call(ble_evt);
        #[cfg(feature = "ble-gatt-server")]
        crate::ble::gatt_server::portal(conn_handle).call(ble_evt);
        #[cfg(feature = "ble-l2cap")]
        crate::ble::l2cap::portal(conn_handle).call(ble_evt);

        trace!("conn {:?}: disconnected", _index);
    }

    pub(crate) fn keyset(&mut self) -> raw::ble_gap_sec_keyset_t {
        #[cfg(feature = "ble-sec")]
        return raw::ble_gap_sec_keyset_t {
            keys_own: raw::ble_gap_sec_keys_t {
                p_enc_key: &mut self.security.own_enc_key,
                p_id_key: core::ptr::null_mut(),
                p_sign_key: core::ptr::null_mut(),
                p_pk: core::ptr::null_mut(),
            },
            keys_peer: raw::ble_gap_sec_keys_t {
                p_enc_key: &mut self.security.peer_enc_key,
                p_id_key: &mut self.security.peer_id,
                p_sign_key: core::ptr::null_mut(),
                p_pk: core::ptr::null_mut(),
            },
        };
        #[cfg(not(feature = "ble-sec"))]
        return raw::ble_gap_sec_keyset_t {
            keys_own: raw::ble_gap_sec_keys_t {
                p_enc_key: core::ptr::null_mut(),
                p_id_key: core::ptr::null_mut(),
                p_sign_key: core::ptr::null_mut(),
                p_pk: core::ptr::null_mut(),
            },
            keys_peer: raw::ble_gap_sec_keys_t {
                p_enc_key: core::ptr::null_mut(),
                p_id_key: core::ptr::null_mut(),
                p_sign_key: core::ptr::null_mut(),
                p_pk: core::ptr::null_mut(),
            },
        };
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Connection {
    index: u8,
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.with_state(|state| {
            state.refcount = unwrap!(
                state.refcount.checked_sub(1),
                "bug: dropping a conn which is already at refcount 0"
            );

            if state.refcount == 0 {
                if state.conn_handle.is_connected() {
                    trace!("conn {:?}: dropped, disconnecting", self.index);
                    // We still leave conn_handle set, because the connection is
                    // not really disconnected until we get GAP_DISCONNECTED event.
                    unwrap!(state.disconnect());
                } else {
                    trace!("conn {:?}: dropped, already disconnected", self.index);
                }
            }
        });
    }
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        self.with_state(|state| {
            state.refcount = unwrap!(state.refcount.checked_add(1), "Too many references to same connection");
        });

        Self { index: self.index }
    }
}

impl Connection {
    pub fn role(&self) -> Role {
        self.with_state(|state| state.role)
    }

    pub fn peer_address(&self) -> Address {
        self.with_state(|state| state.peer_address)
    }

    pub fn disconnect(&self) -> Result<(), DisconnectedError> {
        self.with_state(|state| state.disconnect())
    }

    pub fn disconnect_with_reason(&self, reason: HciStatus) -> Result<(), DisconnectedError> {
        self.with_state(|state| state.disconnect_with_reason(reason))
    }

    pub fn disconnect_reason(&self) -> Option<HciStatus> {
        self.with_state(|state| state.conn_handle.disconnect_reason())
    }

    pub fn handle(&self) -> Option<u16> {
        self.with_state(|state| state.conn_handle.handle())
    }

    pub fn from_handle(conn_handle: u16) -> Option<Connection> {
        index_by_handle(conn_handle).get().map(|index| {
            with_state(index, |state| {
                state.refcount = unwrap!(state.refcount.checked_add(1), "Too many references to same connection");
                Connection { index }
            })
        })
    }

    pub(crate) fn new(
        conn_handle: u16,
        role: Role,
        peer_address: Address,
        conn_params: ble_gap_conn_params_t,
    ) -> Result<Self, OutOfConnsError> {
        allocate_index(|index, state| {
            // Initialize
            *state = ConnectionState {
                refcount: 1,
                conn_handle: ConnHandleState::Connected(conn_handle),
                role,
                peer_address,
                security_mode: SecurityMode::Open,

                disconnecting: false,

                conn_params,

                #[cfg(feature = "ble-rssi")]
                rssi: None,

                #[cfg(feature = "ble-gatt")]
                att_mtu: raw::BLE_GATT_ATT_MTU_DEFAULT as _,

                #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
                data_length_effective: BLE_GAP_DATA_LENGTH_DEFAULT,

                #[cfg(feature = "ble-sec")]
                security: NEW_ENCRYPTION_STATE,
            };

            // Update index_by_handle
            let ibh = index_by_handle(conn_handle);
            assert!(ibh.get().is_none(), "bug: conn_handle already has index");
            ibh.set(Some(index));

            trace!("conn {:?}: connected", index);
            Self { index }
        })
    }

    #[cfg(feature = "ble-sec")]
    pub(crate) fn with_security_handler(
        conn_handle: u16,
        role: Role,
        peer_address: Address,
        conn_params: ble_gap_conn_params_t,
        handler: &'static dyn SecurityHandler,
    ) -> Result<Self, OutOfConnsError> {
        let conn = Self::new(conn_handle, role, peer_address, conn_params)?;
        conn.with_state(|state| state.security.handler = Some(handler));
        Ok(conn)
    }

    /// Start measuring RSSI on this connection.
    #[cfg(feature = "ble-rssi")]
    pub fn start_rssi(&self) {
        if let Ok(conn_handle) = self.with_state(|state| state.check_connected()) {
            let ret = unsafe { raw::sd_ble_gap_rssi_start(conn_handle, 0, 0) };
            if let Err(err) = RawError::convert(ret) {
                warn!("sd_ble_gap_rssi_start err {:?}", err);
            }
        }
    }

    /// Get the connection's RSSI.
    ///
    /// This will return None if `start_rssi` has not been called yet, or if
    /// no measurement has been done yet.
    #[cfg(feature = "ble-rssi")]
    pub fn rssi(&self) -> Option<i8> {
        self.with_state(|state| state.rssi)
    }

    /// Get the currently active connection params.
    pub fn conn_params(&self) -> ble_gap_conn_params_t {
        with_state(self.index, |s| s.conn_params)
    }

    /// Get the currently active ATT MTU.
    #[cfg(feature = "ble-gatt")]
    pub fn att_mtu(&self) -> u16 {
        with_state(self.index, |s| s.att_mtu)
    }

    pub fn security_mode(&self) -> SecurityMode {
        with_state(self.index, |s| s.security_mode)
    }

    #[cfg(feature = "ble-sec")]
    pub fn security_handler(&self) -> Option<&dyn SecurityHandler> {
        with_state(self.index, |s| s.security.handler)
    }

    /// Set the connection params.
    ///
    /// Note that this just initiates the connection param change, it does not wait for completion.
    /// Immediately after return, the active params will still be the old ones, and after some time they
    /// should change to the new ones.
    ///
    /// For central connections, this will initiate a Link Layer connection parameter update procedure.
    /// For peripheral connections, this will send the corresponding L2CAP request to the central. It is then
    /// up to the central to accept or deny the request.
    pub fn set_conn_params(&self, conn_params: ble_gap_conn_params_t) -> Result<(), SetConnParamsError> {
        let conn_handle = self.with_state(|state| state.check_connected())?;
        let ret = unsafe { raw::sd_ble_gap_conn_param_update(conn_handle, &conn_params) };
        if let Err(err) = RawError::convert(ret) {
            warn!("sd_ble_gap_conn_param_update err {:?}", err);
            return Err(err.into());
        }

        Ok(())
    }

    /// Temporarily ignore slave latency for peripehral connections.
    ///
    /// "Slave latency" is a setting in the conn params that allows the peripheral
    /// to intentionally sleep through and miss up to N connection events if it doesn't
    /// have any data to send to the central.
    ///
    /// Slave latency is useful because it can yield the same power savings on the peripheral
    /// as increasing the conn interval, but it only impacts latency in the central->peripheral
    /// direction, not both.
    ///
    /// However, in some cases, if the peripheral knows the central will send it some data soon
    /// it might be useful to temporarily force ignoring the slave latency setting, ie waking up
    /// at every single conn interval, to lower the latency.
    ///
    /// This only works on peripheral connections.
    #[cfg(feature = "ble-peripheral")]
    pub fn ignore_slave_latency(&mut self, ignore: bool) -> Result<(), IgnoreSlaveLatencyError> {
        let conn_handle = self.with_state(|state| state.check_connected())?;

        let mut disable: raw::ble_gap_opt_slave_latency_disable_t = unsafe { core::mem::zeroed() };
        disable.conn_handle = conn_handle;
        disable.set_disable(ignore as u8); // 0 or 1

        let ret = unsafe {
            raw::sd_ble_opt_set(
                raw::BLE_GAP_OPTS_BLE_GAP_OPT_SLAVE_LATENCY_DISABLE,
                &raw::ble_opt_t {
                    gap_opt: raw::ble_gap_opt_t {
                        slave_latency_disable: disable,
                    },
                },
            )
        };
        if let Err(err) = RawError::convert(ret) {
            warn!("ignore_slave_latency sd_ble_opt_set err {:?}", err);
            return Err(err.into());
        }

        Ok(())
    }

    pub(crate) fn with_state<T>(&self, f: impl FnOnce(&mut ConnectionState) -> T) -> T {
        with_state(self.index, f)
    }

    pub fn iter() -> ConnectionIter {
        ConnectionIter(0)
    }

    /// Initiate a Data Length Update procedure.
    ///
    /// Note that this just initiates the data length update, it does not wait for completion.
    /// Immediately after return, the active data length will still be the old one, and after some time they
    /// should change to the new ones.
    #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
    pub fn data_length_update(
        &mut self,
        params: Option<&raw::ble_gap_data_length_params_t>,
    ) -> Result<(), DataLengthUpdateError> {
        let conn_handle = self.with_state(|state| state.check_connected())?;

        let params = params.map(core::ptr::from_ref).unwrap_or(core::ptr::null());
        let mut dl_limitation = unsafe { core::mem::zeroed() };
        let ret = unsafe { raw::sd_ble_gap_data_length_update(conn_handle, params, &mut dl_limitation) };

        if let Err(err) = RawError::convert(ret) {
            warn!("sd_ble_gap_data_length_update err {:?}", err);

            if dl_limitation.tx_payload_limited_octets != 0 || dl_limitation.rx_payload_limited_octets != 0 {
                warn!(
                    "The requested TX/RX packet length is too long by {:?}/{:?} octets.",
                    dl_limitation.tx_payload_limited_octets, dl_limitation.rx_payload_limited_octets
                );
            }

            if dl_limitation.tx_rx_time_limited_us != 0 {
                warn!(
                    "The requested combination of TX and RX packet lengths is too long by {:?} us",
                    dl_limitation.tx_rx_time_limited_us
                );
            }

            let err = match err {
                RawError::NotSupported => DataLengthUpdateError::NotSupported(dl_limitation),
                RawError::Resources => DataLengthUpdateError::Resources(dl_limitation),
                err => DataLengthUpdateError::Raw(err),
            };

            return Err(err);
        }

        Ok(())
    }

    /// Send a request to the connected device to change the PHY.
    ///
    /// Note that this just initiates the PHY change, it does not wait for completion.
    /// Immediately after return, the active PHYs will still be the old ones, and after some time
    /// they should change to the new ones.
    pub fn phy_update(&mut self, tx_phys: PhySet, rx_phys: PhySet) -> Result<(), PhyUpdateError> {
        let conn_handle = self.with_state(|state| state.check_connected())?;
        let p_gap_phys = raw::ble_gap_phys_t {
            tx_phys: tx_phys as u8,
            rx_phys: rx_phys as u8,
        };
        let ret = unsafe { raw::sd_ble_gap_phy_update(conn_handle, &p_gap_phys) };
        if let Err(err) = RawError::convert(ret) {
            warn!("sd_ble_gap_phy_update err {:?}", err);
            return Err(err.into());
        }

        Ok(())
    }

    #[cfg(feature = "ble-central")]
    /// Send a pairing request to the peripheral.
    pub fn request_pairing(&self) -> Result<(), AuthenticateError> {
        let (conn_handle, sec_params) = self.with_state(|state| {
            assert!(
                state.role == Role::Central,
                "Connection::request_pairing may only be called for connections in the central role"
            );

            let conn_handle = state.check_connected()?;

            #[cfg(not(feature = "ble-sec"))]
            let sec_params = default_security_params();
            #[cfg(feature = "ble-sec")]
            let sec_params = state
                .security
                .handler
                .map(|h| h.security_params(self))
                .unwrap_or_else(default_security_params);

            Ok::<_, AuthenticateError>((conn_handle, sec_params))
        })?;

        let ret = unsafe { raw::sd_ble_gap_authenticate(conn_handle, &sec_params) };
        if let Err(err) = RawError::convert(ret) {
            warn!("sd_ble_gap_authenticate err {:?}", err);
            return Err(err.into());
        }

        Ok(())
    }

    #[cfg(feature = "ble-peripheral")]
    /// Send a security request to the central.
    pub fn request_security(&self) -> Result<(), AuthenticateError> {
        let (conn_handle, sec_params) = self.with_state(|state| {
            assert!(
                state.role == Role::Peripheral,
                "Connection::request_security may only be called for connections in the peripheral role"
            );

            let conn_handle = state.check_connected()?;

            #[cfg(not(feature = "ble-sec"))]
            let sec_params = unsafe { core::mem::zeroed() };
            #[cfg(feature = "ble-sec")]
            let sec_params = state
                .security
                .handler
                .map(|h| {
                    let mut p: raw::ble_gap_sec_params_t = unsafe { core::mem::zeroed() };
                    p.set_bond(h.can_bond(self) as u8);
                    p.set_mitm(h.request_mitm_protection(self) as u8);
                    p
                })
                .unwrap_or_else(|| unsafe { core::mem::zeroed() });

            Ok::<_, AuthenticateError>((conn_handle, sec_params))
        })?;

        let ret = unsafe { raw::sd_ble_gap_authenticate(conn_handle, &sec_params) };
        if let Err(err) = RawError::convert(ret) {
            warn!("sd_ble_gap_authenticate err {:?}", err);
            return Err(err.into());
        }

        Ok(())
    }

    #[cfg(all(feature = "ble-central", feature = "ble-sec"))]
    /// Initiate GAP encryption with the peripheral using stored keys
    pub fn encrypt(&self) -> Result<(), EncryptError> {
        let (conn_handle, (master_id, ltk)) = self.with_state(|state| {
            let conn_handle = state.check_connected()?;
            state
                .security
                .handler
                .ok_or(EncryptError::NoSecurityHandler)
                .and_then(|handler| handler.get_peripheral_key(self).ok_or(EncryptError::PeerKeysNotFound))
                .map(|keys| (conn_handle, keys))
        })?;

        let ret = unsafe { raw::sd_ble_gap_encrypt(conn_handle, master_id.as_raw(), ltk.as_raw()) };
        if let Err(err) = RawError::convert(ret) {
            warn!("sd_ble_gap_authenticate err {:?}", err);
            return Err(err.into());
        }

        Ok(())
    }
}

pub struct ConnectionIter(u8);

impl Iterator for ConnectionIter {
    type Item = Connection;

    fn next(&mut self) -> Option<Self::Item> {
        let n = usize::from(self.0);
        if n < CONNS_MAX {
            unsafe {
                for (i, s) in STATES[n..].iter().enumerate() {
                    let state = &mut *s.get();
                    if state.conn_handle.is_connected() {
                        let index = (n + i) as u8;
                        state.refcount =
                            unwrap!(state.refcount.checked_add(1), "Too many references to same connection");
                        self.0 = index + 1;
                        return Some(Connection { index });
                    }
                }
            }
            self.0 = CONNS_MAX as u8;
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(CONNS_MAX - usize::from(self.0)))
    }
}

impl FusedIterator for ConnectionIter {}

// ConnectionStates by index.
const DUMMY_STATE: UnsafeCell<ConnectionState> = UnsafeCell::new(ConnectionState::dummy());
static mut STATES: [UnsafeCell<ConnectionState>; CONNS_MAX] = [DUMMY_STATE; CONNS_MAX];

pub(crate) fn with_state_by_conn_handle<T>(conn_handle: u16, f: impl FnOnce(&mut ConnectionState) -> T) -> T {
    let index = unwrap!(
        index_by_handle(conn_handle).get(),
        "bug: with_state_by_conn_handle on conn_handle that has no state"
    );
    with_state(index, f)
}

pub(crate) fn with_state<T>(index: u8, f: impl FnOnce(&mut ConnectionState) -> T) -> T {
    let state = unsafe { &mut *STATES[index as usize].get() };
    f(state)
}

fn allocate_index<T>(f: impl FnOnce(u8, &mut ConnectionState) -> T) -> Result<T, OutOfConnsError> {
    unsafe {
        for (i, s) in (&*(&raw const STATES)).iter().enumerate() {
            let state = &mut *s.get();
            if state.refcount == 0 && !state.conn_handle.is_connected() {
                return Ok(f(i as u8, state));
            }
        }
        Err(OutOfConnsError)
    }
}

// conn_handle -> index mapping. Used to make stuff go faster
const INDEX_NONE: Cell<Option<u8>> = Cell::new(None);
static mut INDEX_BY_HANDLE: [Cell<Option<u8>>; CONNS_MAX] = [INDEX_NONE; CONNS_MAX];

fn index_by_handle(conn_handle: u16) -> &'static Cell<Option<u8>> {
    unsafe { &INDEX_BY_HANDLE[conn_handle as usize] }
}
