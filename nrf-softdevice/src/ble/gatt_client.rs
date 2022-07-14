//! Generic Attribute client. GATT clients consume functionality offered by GATT servers.

use heapless::Vec;
use num_enum::{FromPrimitive, IntoPrimitive};

use crate::ble::*;
use crate::util::{get_flexarray, get_union_field, Portal};
use crate::{raw, RawError};

/// Discovered characteristic
pub struct Characteristic {
    pub uuid: Option<Uuid>,
    pub handle_decl: u16,
    pub handle_value: u16,
    pub props: raw::ble_gatt_char_props_t,
    pub has_ext_props: bool,
}

/// Discovered descriptor
pub struct Descriptor {
    pub uuid: Option<Uuid>,
    pub handle: u16,
}

/// Trait for implementing GATT clients.
pub trait Client {
    /// Get the UUID of the GATT service. This is used by [`discover`] to search for the
    /// service in the GATT server.
    fn uuid() -> Uuid;

    /// Create a new instance in a "not-yet-discovered" state.
    fn new_undiscovered(conn: Connection) -> Self;

    /// Called by [`discover`] for every discovered characteristic. Implementations must
    /// check if they're interested in the UUID of the characteristic, and save their
    /// handles if needed.
    fn discovered_characteristic(&mut self, characteristic: &Characteristic, descriptors: &[Descriptor]);

    /// Called by [`discover`] at the end of the discovery procedure. Implementations must check
    /// that all required characteristics have been discovered, and return [`DiscoverError::ServiceIncomplete`]
    /// otherwise.
    ///
    /// If no error is returned, this instance is considered ready to use and is returned to
    /// the caller of [`discover`]
    fn discovery_complete(&mut self) -> Result<(), DiscoverError>;
}

#[rustfmt::skip]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, FromPrimitive)]
pub enum GattError {
    // This is not really an error, but IMO it's better to add it
    // anyway, just in case someone mistakenly converts BLE_GATT_STATUS_SUCCESS into GattError.
    // if they see "Success" they'll easily realize their mistake, if they see "Unknown" it'd be confusing.
    Success = raw::BLE_GATT_STATUS_SUCCESS,

    #[num_enum(default)]
    Unknown = raw::BLE_GATT_STATUS_UNKNOWN,

    AtterrInvalid = raw::BLE_GATT_STATUS_ATTERR_INVALID,
    AtterrInvalidHandle = raw::BLE_GATT_STATUS_ATTERR_INVALID_HANDLE,
    AtterrReadNotPermitted = raw::BLE_GATT_STATUS_ATTERR_READ_NOT_PERMITTED,
    AtterrWriteNotPermitted = raw::BLE_GATT_STATUS_ATTERR_WRITE_NOT_PERMITTED,
    AtterrInvalidPdu = raw::BLE_GATT_STATUS_ATTERR_INVALID_PDU,
    AtterrInsufAuthentication = raw::BLE_GATT_STATUS_ATTERR_INSUF_AUTHENTICATION,
    AtterrRequestNotSupported = raw::BLE_GATT_STATUS_ATTERR_REQUEST_NOT_SUPPORTED,
    AtterrInvalidOffset = raw::BLE_GATT_STATUS_ATTERR_INVALID_OFFSET,
    AtterrInsufAuthorization = raw::BLE_GATT_STATUS_ATTERR_INSUF_AUTHORIZATION,
    AtterrPrepareQueueFull = raw::BLE_GATT_STATUS_ATTERR_PREPARE_QUEUE_FULL,
    AtterrAttributeNotFound = raw::BLE_GATT_STATUS_ATTERR_ATTRIBUTE_NOT_FOUND,
    AtterrAttributeNotLong = raw::BLE_GATT_STATUS_ATTERR_ATTRIBUTE_NOT_LONG,
    AtterrInsufEncKeySize = raw::BLE_GATT_STATUS_ATTERR_INSUF_ENC_KEY_SIZE,
    AtterrInvalidAttValLength = raw::BLE_GATT_STATUS_ATTERR_INVALID_ATT_VAL_LENGTH,
    AtterrUnlikelyError = raw::BLE_GATT_STATUS_ATTERR_UNLIKELY_ERROR,
    AtterrInsufEncryption = raw::BLE_GATT_STATUS_ATTERR_INSUF_ENCRYPTION,
    AtterrUnsupportedGroupType = raw::BLE_GATT_STATUS_ATTERR_UNSUPPORTED_GROUP_TYPE,
    AtterrInsufResources = raw::BLE_GATT_STATUS_ATTERR_INSUF_RESOURCES,
    AtterrCpsWriteReqRejected = raw::BLE_GATT_STATUS_ATTERR_CPS_WRITE_REQ_REJECTED,
    AtterrCpsCccdConfigError = raw::BLE_GATT_STATUS_ATTERR_CPS_CCCD_CONFIG_ERROR,
    AtterrCpsProcAlrInProg = raw::BLE_GATT_STATUS_ATTERR_CPS_PROC_ALR_IN_PROG,
    AtterrCpsOutOfRange = raw::BLE_GATT_STATUS_ATTERR_CPS_OUT_OF_RANGE,
}

/// Error type for [`discover`]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DiscoverError {
    /// Connection is disconnected.
    Disconnected,
    /// No service with the given UUID found in the server.
    ServiceNotFound,
    /// Service with the given UUID found, but it's missing some required characteristics.
    ServiceIncomplete,
    Gatt(GattError),
    Raw(RawError),
}

impl From<DisconnectedError> for DiscoverError {
    fn from(_: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

impl From<GattError> for DiscoverError {
    fn from(err: GattError) -> Self {
        Self::Gatt(err)
    }
}

impl From<RawError> for DiscoverError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

const DISC_CHARS_MAX: usize = 6;
const DISC_DESCS_MAX: usize = 6;

pub(crate) async fn discover_service(conn: &Connection, uuid: Uuid) -> Result<raw::ble_gattc_service_t, DiscoverError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;
    let ret = unsafe { raw::sd_ble_gattc_primary_services_discover(conn_handle, 1, uuid.as_raw_ptr()) };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gattc_primary_services_discover err {:?}", err);
        err
    })?;

    portal(conn_handle)
        .wait_once(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => return Err(DiscoverError::Disconnected),
                raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_PRIM_SRVC_DISC_RSP => {
                    let gattc_evt = check_status(ble_evt)?;
                    let params = get_union_field(ble_evt, &gattc_evt.params.prim_srvc_disc_rsp);
                    let v = get_flexarray(ble_evt, &params.services, params.count as usize);

                    match v.len() {
                        0 => Err(DiscoverError::ServiceNotFound),
                        1 => Ok(v[0]),
                        _n => {
                            warn!(
                                "Found {:?} services with the same UUID, using the first one",
                                params.count
                            );
                            Ok(v[0])
                        }
                    }
                }
                e => panic!("unexpected event {}", e),
            }
        })
        .await
}

// =============================

async fn discover_characteristics(
    conn: &Connection,
    start_handle: u16,
    end_handle: u16,
) -> Result<Vec<raw::ble_gattc_char_t, DISC_CHARS_MAX>, DiscoverError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;

    let ret = unsafe {
        raw::sd_ble_gattc_characteristics_discover(
            conn_handle,
            &raw::ble_gattc_handle_range_t {
                start_handle,
                end_handle,
            },
        )
    };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gattc_characteristics_discover err {:?}", err);
        err
    })?;

    portal(conn_handle)
        .wait_once(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => return Err(DiscoverError::Disconnected),
                raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_DISC_RSP => {
                    let gattc_evt = check_status(ble_evt)?;
                    let params = get_union_field(ble_evt, &gattc_evt.params.char_disc_rsp);
                    let v = get_flexarray(ble_evt, &params.chars, params.count as usize);
                    let v = Vec::from_slice(v)
                        .unwrap_or_else(|_| panic!("too many gatt chars, increase DiscCharsMax: {:?}", v.len()));
                    Ok(v)
                }
                e => panic!("unexpected event {}", e),
            }
        })
        .await
}

// =============================

async fn discover_descriptors(
    conn: &Connection,
    start_handle: u16,
    end_handle: u16,
) -> Result<Vec<raw::ble_gattc_desc_t, DISC_DESCS_MAX>, DiscoverError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;

    let ret = unsafe {
        raw::sd_ble_gattc_descriptors_discover(
            conn_handle,
            &raw::ble_gattc_handle_range_t {
                start_handle,
                end_handle,
            },
        )
    };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gattc_descriptors_discover err {:?}", err);
        err
    })?;

    portal(conn_handle)
        .wait_once(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => return Err(DiscoverError::Disconnected),
                raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_DESC_DISC_RSP => {
                    let gattc_evt = check_status(ble_evt)?;
                    let params = get_union_field(ble_evt, &gattc_evt.params.desc_disc_rsp);
                    let v = get_flexarray(ble_evt, &params.descs, params.count as usize);
                    let v = Vec::from_slice(v)
                        .unwrap_or_else(|_| panic!("too many gatt descs, increase DiscDescsMax: {:?}", v.len()));
                    Ok(v)
                }
                e => panic!("unexpected event {}", e),
            }
        })
        .await
}

// =============================

async fn discover_inner<T: Client>(
    conn: &Connection,
    client: &mut T,
    svc: &raw::ble_gattc_service_t,
    curr: raw::ble_gattc_char_t,
    next: Option<raw::ble_gattc_char_t>,
) -> Result<(), DiscoverError> {
    // Calcuate range of possible descriptors
    let start_handle = curr.handle_value + 1;
    let end_handle = next.map(|c| c.handle_decl - 1).unwrap_or(svc.handle_range.end_handle);

    let characteristic = Characteristic {
        uuid: Uuid::from_raw(curr.uuid),
        handle_decl: curr.handle_decl,
        handle_value: curr.handle_value,
        has_ext_props: curr.char_ext_props() != 0,
        props: curr.char_props,
    };

    let mut descriptors: Vec<Descriptor, DISC_DESCS_MAX> = Vec::new();

    // Only if range is non-empty, discover. (if it's empty there must be no descriptors)
    if start_handle <= end_handle {
        let descs = {
            match discover_descriptors(conn, start_handle, end_handle).await {
                Ok(descs) => descs,
                Err(DiscoverError::Gatt(GattError::AtterrAttributeNotFound)) => Vec::new(),
                Err(err) => return Err(err),
            }
        };
        for desc in descs {
            descriptors
                .push(Descriptor {
                    uuid: Uuid::from_raw(desc.uuid),
                    handle: desc.handle,
                })
                .unwrap_or_else(|_| panic!("no size in descriptors"));
        }
    }

    client.discovered_characteristic(&characteristic, &descriptors[..]);

    Ok(())
}

/// Discover a service in the peer's GATT server and construct a Client instance
/// to use it.
pub async fn discover<T: Client>(conn: &Connection) -> Result<T, DiscoverError> {
    // TODO handle drop. Probably doable gracefully (no DropBomb)

    let svc = match discover_service(conn, T::uuid()).await {
        Err(DiscoverError::Gatt(GattError::AtterrAttributeNotFound)) => Err(DiscoverError::ServiceNotFound),
        x => x,
    }?;

    let mut client = T::new_undiscovered(conn.clone());

    let mut curr_handle = svc.handle_range.start_handle;
    let end_handle = svc.handle_range.end_handle;

    let mut prev_char: Option<raw::ble_gattc_char_t> = None;
    while curr_handle < end_handle {
        let chars = match discover_characteristics(conn, curr_handle, end_handle).await {
            Err(DiscoverError::Gatt(GattError::AtterrAttributeNotFound)) => break,
            x => x,
        }?;
        assert_ne!(chars.len(), 0);
        for curr in chars {
            if let Some(prev) = prev_char {
                discover_inner(conn, &mut client, &svc, prev, Some(curr)).await?;
            }
            prev_char = Some(curr);
            curr_handle = curr.handle_value + 1;
        }
    }
    if let Some(prev) = prev_char {
        discover_inner(conn, &mut client, &svc, prev, None).await?;
    }

    client.discovery_complete()?;

    Ok(client)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ReadError {
    Disconnected,
    Truncated,
    Gatt(GattError),
    Raw(RawError),
}

impl From<DisconnectedError> for ReadError {
    fn from(_: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

impl From<GattError> for ReadError {
    fn from(err: GattError) -> Self {
        Self::Gatt(err)
    }
}

impl From<RawError> for ReadError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

pub async fn read(conn: &Connection, handle: u16, buf: &mut [u8]) -> Result<usize, ReadError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;

    let ret = unsafe { raw::sd_ble_gattc_read(conn_handle, handle, 0) };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gattc_read err {:?}", err);
        err
    })?;

    portal(conn_handle)
        .wait_many(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => return Some(Err(ReadError::Disconnected)),
                raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_READ_RSP => {
                    let gattc_evt = match check_status(ble_evt) {
                        Ok(evt) => evt,
                        Err(e) => return Some(Err(e.into())),
                    };
                    let params = get_union_field(ble_evt, &gattc_evt.params.read_rsp);
                    let v = get_flexarray(ble_evt, &params.data, params.len as usize);
                    let len = core::cmp::min(v.len(), buf.len());
                    buf[..len].copy_from_slice(&v[..len]);

                    if v.len() > buf.len() {
                        return Some(Err(ReadError::Truncated));
                    }
                    Some(Ok(len))
                }
                _ => None,
            }
        })
        .await
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WriteError {
    Disconnected,
    Timeout,
    Gatt(GattError),
    Raw(RawError),
}

impl From<DisconnectedError> for WriteError {
    fn from(_: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

impl From<GattError> for WriteError {
    fn from(err: GattError) -> Self {
        Self::Gatt(err)
    }
}

impl From<RawError> for WriteError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

pub async fn write(conn: &Connection, handle: u16, buf: &[u8]) -> Result<(), WriteError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;

    assert!(buf.len() <= u16::MAX as usize);
    let params = raw::ble_gattc_write_params_t {
        write_op: raw::BLE_GATT_OP_WRITE_REQ as u8,
        flags: 0,
        handle,
        p_value: buf.as_ptr(),
        len: buf.len() as u16,
        offset: 0,
    };

    let ret = unsafe { raw::sd_ble_gattc_write(conn_handle, &params) };
    RawError::convert(ret).map_err(|err| {
        warn!("sd_ble_gattc_write err {:?}", err);
        err
    })?;

    portal(conn_handle)
        .wait_many(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => return Some(Err(WriteError::Disconnected)),
                raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_RSP => {
                    match check_status(ble_evt) {
                        Ok(_) => {}
                        Err(e) => return Some(Err(e.into())),
                    };
                    Some(Ok(()))
                }
                raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_TIMEOUT => {
                    return Some(Err(WriteError::Timeout));
                }
                _ => None,
            }
        })
        .await
}

pub async fn write_without_response(conn: &Connection, handle: u16, buf: &[u8]) -> Result<(), WriteError> {
    loop {
        let conn_handle = conn.with_state(|state| state.check_connected())?;

        assert!(buf.len() <= u16::MAX as usize);
        let params = raw::ble_gattc_write_params_t {
            write_op: raw::BLE_GATT_OP_WRITE_CMD as u8,
            flags: 0,
            handle,
            p_value: buf.as_ptr(),
            len: buf.len() as u16,
            offset: 0,
        };

        let ret = unsafe { raw::sd_ble_gattc_write(conn_handle, &params) };
        match RawError::convert(ret) {
            Err(RawError::Resources) => {}
            Err(e) => return Err(e.into()),
            Ok(()) => return Ok(()),
        }

        portal(conn_handle)
            .wait_many(|ble_evt| unsafe {
                match (*ble_evt).header.evt_id as u32 {
                    raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => return Some(Err(WriteError::Disconnected)),
                    raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_CMD_TX_COMPLETE => Some(Ok(())),
                    _ => None,
                }
            })
            .await?;
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TryWriteError {
    Disconnected,
    BufferFull,
    Gatt(GattError),
    Raw(RawError),
}

impl From<DisconnectedError> for TryWriteError {
    fn from(_: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

impl From<GattError> for TryWriteError {
    fn from(err: GattError) -> Self {
        Self::Gatt(err)
    }
}

impl From<RawError> for TryWriteError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

pub fn try_write_without_response(conn: &Connection, handle: u16, buf: &[u8]) -> Result<(), TryWriteError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;

    assert!(buf.len() <= u16::MAX as usize);
    let params = raw::ble_gattc_write_params_t {
        write_op: raw::BLE_GATT_OP_WRITE_CMD as u8,
        flags: 0,
        handle,
        p_value: buf.as_ptr(),
        len: buf.len() as u16,
        offset: 0,
    };

    let ret = unsafe { raw::sd_ble_gattc_write(conn_handle, &params) };
    match RawError::convert(ret) {
        Err(RawError::Resources) => Err(TryWriteError::BufferFull),
        Err(e) => Err(e.into()),
        Ok(()) => Ok(()),
    }
}

unsafe fn check_status(ble_evt: *const raw::ble_evt_t) -> Result<&'static raw::ble_gattc_evt_t, GattError> {
    let gattc_evt = get_union_field(ble_evt, &(*ble_evt).evt.gattc_evt);
    match gattc_evt.gatt_status as u32 {
        raw::BLE_GATT_STATUS_SUCCESS => Ok(gattc_evt),
        err => Err(GattError::from(err as u32)),
    }
}

pub(crate) unsafe fn on_evt(ble_evt: *const raw::ble_evt_t) {
    let gattc_evt = get_union_field(ble_evt, &(*ble_evt).evt.gattc_evt);
    portal(gattc_evt.conn_handle).call(ble_evt);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MtuExchangeError {
    /// Connection is disconnected.
    Disconnected,
    Gatt(GattError),
    Raw(RawError),
}

impl From<DisconnectedError> for MtuExchangeError {
    fn from(_: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

impl From<GattError> for MtuExchangeError {
    fn from(err: GattError) -> Self {
        Self::Gatt(err)
    }
}

impl From<RawError> for MtuExchangeError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

#[cfg(feature = "ble-central")]
pub(crate) async fn att_mtu_exchange(conn: &Connection, mtu: u16) -> Result<(), MtuExchangeError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;

    let current_mtu = conn.with_state(|state| state.att_mtu);

    if current_mtu >= mtu {
        debug!(
            "att mtu exchange: want mtu {:?}, already got {:?}. Doing nothing.",
            mtu, current_mtu
        );
        return Ok(());
    }

    debug!(
        "att mtu exchange: want mtu {:?}, got only {:?}, doing exchange...",
        mtu, current_mtu
    );

    let ret = unsafe { raw::sd_ble_gattc_exchange_mtu_request(conn_handle, mtu) };
    if let Err(err) = RawError::convert(ret) {
        warn!("sd_ble_gattc_exchange_mtu_request err {:?}", err);
        return Err(err.into());
    }

    portal(conn_handle)
        .wait_once(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => return Err(MtuExchangeError::Disconnected),
                raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_EXCHANGE_MTU_RSP => {
                    let gattc_evt = match check_status(ble_evt) {
                        Ok(evt) => evt,
                        Err(e) => return Err(e.into()),
                    };
                    let params = get_union_field(ble_evt, &gattc_evt.params.exchange_mtu_rsp);
                    let mtu = params.server_rx_mtu;
                    debug!("att mtu exchange: got mtu {:?}", mtu);
                    conn.with_state(|state| state.att_mtu = mtu);

                    Ok(())
                }
                e => panic!("unexpected event {}", e),
            }
        })
        .await
}

const PORTAL_NEW: Portal<*const raw::ble_evt_t> = Portal::new();
static PORTALS: [Portal<*const raw::ble_evt_t>; CONNS_MAX] = [PORTAL_NEW; CONNS_MAX];
pub(crate) fn portal(conn_handle: u16) -> &'static Portal<*const raw::ble_evt_t> {
    &PORTALS[conn_handle as usize]
}
