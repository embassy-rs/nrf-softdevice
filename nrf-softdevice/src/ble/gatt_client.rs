//! Generic Attribute client. GATT clients consume functionality offered by GATT servers.

use heapless::consts::*;
use heapless::Vec;
use num_enum::{FromPrimitive, IntoPrimitive};

use crate::ble::*;
use crate::raw;
use crate::util::*;
use crate::RawError;

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
    fn discovered_characteristic(
        &mut self,
        characteristic: &Characteristic,
        descriptors: &[Descriptor],
    );

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
#[derive(defmt::Format, IntoPrimitive, FromPrimitive)]
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
#[derive(defmt::Format)]
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

type DiscCharsMax = U6;
type DiscDescsMax = U6;

pub(crate) enum PortalMessage {
    DiscoverService(*const raw::ble_evt_t),
    DiscoverCharacteristics(*const raw::ble_evt_t),
    DiscoverDescriptors(*const raw::ble_evt_t),
    Read(*const raw::ble_evt_t),
    Write(*const raw::ble_evt_t),
    WriteTxComplete(*const raw::ble_evt_t),
    Disconnected,
}

pub(crate) async fn discover_service(
    conn: &Connection,
    uuid: Uuid,
) -> Result<raw::ble_gattc_service_t, DiscoverError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;
    let ret =
        unsafe { raw::sd_ble_gattc_primary_services_discover(conn_handle, 1, uuid.as_raw_ptr()) };
    RawError::convert(ret).dewarn(intern!("sd_ble_gattc_primary_services_discover"))?;

    portal(conn_handle)
        .wait_once(|e| match e {
            PortalMessage::DiscoverService(ble_evt) => unsafe {
                let gattc_evt = check_status(ble_evt)?;
                let params = get_union_field(ble_evt, &gattc_evt.params.prim_srvc_disc_rsp);
                let v = get_flexarray(ble_evt, &params.services, params.count as usize);

                match v.len() {
                    0 => Err(DiscoverError::ServiceNotFound),
                    1 => Ok(v[0]),
                    n => {
                        warn!(
                            "Found {:u16} services with the same UUID, using the first one",
                            params.count
                        );
                        Ok(v[0])
                    }
                }
            },
            PortalMessage::Disconnected => Err(DiscoverError::Disconnected),
            _ => unreachable!(),
        })
        .await
}

pub(crate) unsafe fn on_prim_srvc_disc_rsp(
    ble_evt: *const raw::ble_evt_t,
    gattc_evt: &raw::ble_gattc_evt_t,
) {
    trace!(
        "gattc on_prim_srvc_disc_rsp conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );
    portal(gattc_evt.conn_handle).call(PortalMessage::DiscoverService(ble_evt))
}

// =============================

async fn discover_characteristics(
    conn: &Connection,
    start_handle: u16,
    end_handle: u16,
) -> Result<Vec<raw::ble_gattc_char_t, DiscCharsMax>, DiscoverError> {
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
    RawError::convert(ret).dewarn(intern!("sd_ble_gattc_characteristics_discover"))?;

    portal(conn_handle)
        .wait_once(|e| match e {
            PortalMessage::DiscoverCharacteristics(ble_evt) => unsafe {
                let gattc_evt = check_status(ble_evt)?;
                let params = get_union_field(ble_evt, &gattc_evt.params.char_disc_rsp);
                let v = get_flexarray(ble_evt, &params.chars, params.count as usize);
                let v = Vec::from_slice(v).unwrap_or_else(|_| {
                    depanic!("too many gatt chars, increase DiscCharsMax: {:?}", v.len())
                });
                Ok(v)
            },
            PortalMessage::Disconnected => Err(DiscoverError::Disconnected),
            _ => unreachable!(),
        })
        .await
}

pub(crate) unsafe fn on_char_disc_rsp(
    ble_evt: *const raw::ble_evt_t,
    gattc_evt: &raw::ble_gattc_evt_t,
) {
    trace!(
        "gattc on_char_disc_rsp conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );

    portal(gattc_evt.conn_handle).call(PortalMessage::DiscoverCharacteristics(ble_evt))
}

// =============================

async fn discover_descriptors(
    conn: &Connection,
    start_handle: u16,
    end_handle: u16,
) -> Result<Vec<raw::ble_gattc_desc_t, DiscDescsMax>, DiscoverError> {
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
    RawError::convert(ret).dewarn(intern!("sd_ble_gattc_descriptors_discover"))?;

    portal(conn_handle)
        .wait_once(|e| match e {
            PortalMessage::DiscoverDescriptors(ble_evt) => unsafe {
                let gattc_evt = check_status(ble_evt)?;
                let params = get_union_field(ble_evt, &gattc_evt.params.desc_disc_rsp);
                let v = get_flexarray(ble_evt, &params.descs, params.count as usize);
                let v = Vec::from_slice(v).unwrap_or_else(|_| {
                    depanic!("too many gatt descs, increase DiscDescsMax: {:?}", v.len())
                });
                Ok(v)
            },
            PortalMessage::Disconnected => Err(DiscoverError::Disconnected),
            _ => unreachable!(),
        })
        .await
}

pub(crate) unsafe fn on_desc_disc_rsp(
    ble_evt: *const raw::ble_evt_t,
    gattc_evt: &raw::ble_gattc_evt_t,
) {
    trace!(
        "gattc on_desc_disc_rsp conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );

    portal(gattc_evt.conn_handle).call(PortalMessage::DiscoverDescriptors(ble_evt))
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
    let end_handle = next
        .map(|c| c.handle_decl - 1)
        .unwrap_or(svc.handle_range.end_handle);

    let characteristic = Characteristic {
        uuid: Uuid::from_raw(curr.uuid),
        handle_decl: curr.handle_decl,
        handle_value: curr.handle_value,
        has_ext_props: curr.char_ext_props() != 0,
        props: curr.char_props,
    };

    let mut descriptors: Vec<Descriptor, DiscDescsMax> = Vec::new();

    // Only if range is non-empty, discover. (if it's empty there must be no descriptors)
    if start_handle <= end_handle {
        for desc in discover_descriptors(conn, start_handle, end_handle).await? {
            descriptors
                .push(Descriptor {
                    uuid: Uuid::from_raw(desc.uuid),
                    handle: desc.handle,
                })
                .unwrap_or_else(|_| depanic!("no size in descriptors"));
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
        Err(DiscoverError::Gatt(GattError::AtterrAttributeNotFound)) => {
            Err(DiscoverError::ServiceNotFound)
        }
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
        deassert!(chars.len() != 0);
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

#[derive(defmt::Format)]
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
    RawError::convert(ret).dewarn(intern!("sd_ble_gattc_read"))?;

    portal(conn_handle)
        .wait_many(|e| match e {
            PortalMessage::Read(ble_evt) => unsafe {
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
            },
            PortalMessage::Disconnected => Some(Err(ReadError::Disconnected)),
            _ => None,
        })
        .await
}

pub(crate) unsafe fn on_read_rsp(ble_evt: *const raw::ble_evt_t, gattc_evt: &raw::ble_gattc_evt_t) {
    trace!(
        "gattc on_read_rsp conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );

    portal(gattc_evt.conn_handle).call(PortalMessage::Read(ble_evt))
}

#[derive(defmt::Format)]
pub enum WriteError {
    Disconnected,
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

    deassert!(buf.len() <= u16::MAX as usize);
    let params = raw::ble_gattc_write_params_t {
        write_op: raw::BLE_GATT_OP_WRITE_REQ as u8,
        flags: 0,
        handle,
        p_value: buf.as_ptr(),
        len: buf.len() as u16,
        offset: 0,
    };

    let ret = unsafe { raw::sd_ble_gattc_write(conn_handle, &params) };
    RawError::convert(ret).dewarn(intern!("sd_ble_gattc_write"))?;

    portal(conn_handle)
        .wait_many(|e| match e {
            PortalMessage::Write(ble_evt) => unsafe {
                match check_status(ble_evt) {
                    Ok(_) => {}
                    Err(e) => return Some(Err(e.into())),
                };
                Some(Ok(()))
            },
            PortalMessage::Disconnected => Some(Err(WriteError::Disconnected)),
            _ => None,
        })
        .await
}

pub async fn write_without_response(
    conn: &Connection,
    handle: u16,
    buf: &[u8],
) -> Result<(), WriteError> {
    loop {
        let conn_handle = conn.with_state(|state| state.check_connected())?;

        deassert!(buf.len() <= u16::MAX as usize);
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
            .wait_many(|e| match e {
                PortalMessage::WriteTxComplete(_) => Some(Ok(())),
                PortalMessage::Disconnected => Some(Err(WriteError::Disconnected)),
                _ => None,
            })
            .await?;
    }
}

#[derive(defmt::Format)]
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

pub fn try_write_without_response(
    conn: &Connection,
    handle: u16,
    buf: &[u8],
) -> Result<(), TryWriteError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;

    deassert!(buf.len() <= u16::MAX as usize);
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

pub(crate) unsafe fn on_write_rsp(
    ble_evt: *const raw::ble_evt_t,
    gattc_evt: &raw::ble_gattc_evt_t,
) {
    trace!(
        "gattc on_write_rsp conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );

    portal(gattc_evt.conn_handle).call(PortalMessage::Write(ble_evt))
}

unsafe fn check_status(
    ble_evt: *const raw::ble_evt_t,
) -> Result<&'static raw::ble_gattc_evt_t, GattError> {
    let gattc_evt = get_union_field(ble_evt, &(*ble_evt).evt.gattc_evt);
    match gattc_evt.gatt_status as u32 {
        raw::BLE_GATT_STATUS_SUCCESS => Ok(gattc_evt),
        err => Err(GattError::from(err as u32)),
    }
}

pub(crate) unsafe fn on_rel_disc_rsp(
    _ble_evt: *const raw::ble_evt_t,
    gattc_evt: &raw::ble_gattc_evt_t,
) {
    trace!(
        "gattc on_rel_disc_rsp conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );
}

pub(crate) unsafe fn on_attr_info_disc_rsp(
    _ble_evt: *const raw::ble_evt_t,
    gattc_evt: &raw::ble_gattc_evt_t,
) {
    trace!(
        "gattc on_attr_info_disc_rsp conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );
}

pub(crate) unsafe fn on_char_val_by_uuid_read_rsp(
    _ble_evt: *const raw::ble_evt_t,
    gattc_evt: &raw::ble_gattc_evt_t,
) {
    trace!(
        "gattc on_char_val_by_uuid_read_rsp conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );
}

pub(crate) unsafe fn on_char_vals_read_rsp(
    _ble_evt: *const raw::ble_evt_t,
    gattc_evt: &raw::ble_gattc_evt_t,
) {
    trace!(
        "gattc on_char_vals_read_rsp conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );
}

pub(crate) unsafe fn on_hvx(_ble_evt: *const raw::ble_evt_t, gattc_evt: &raw::ble_gattc_evt_t) {
    trace!(
        "gattc on_hvx conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );
}

pub(crate) unsafe fn on_exchange_mtu_rsp(
    ble_evt: *const raw::ble_evt_t,
    gattc_evt: &raw::ble_gattc_evt_t,
) {
    let conn_handle = gattc_evt.conn_handle;
    connection::with_state_by_conn_handle(conn_handle, |state| {
        // TODO can probably get it from gattc_evt directly?
        let exchange_mtu_rsp = get_union_field(ble_evt, &gattc_evt.params.exchange_mtu_rsp);
        let server_rx_mtu = exchange_mtu_rsp.server_rx_mtu;

        // Determine the lowest MTU between our own desired MTU and the peer's.
        // The MTU may not be less than BLE_GATT_ATT_MTU_DEFAULT.
        let att_mtu_effective = core::cmp::min(server_rx_mtu, state.att_mtu_desired);
        let att_mtu_effective =
            core::cmp::max(att_mtu_effective, raw::BLE_GATT_ATT_MTU_DEFAULT as u16);

        state.att_mtu_effective = att_mtu_effective;

        trace!(
            "gattc on_exchange_mtu_rsp conn_handle={:u16} gatt_status={:u16} server_rx_mtu={:u16} att_mtu_effective=={:u16}",
            gattc_evt.conn_handle,
            gattc_evt.gatt_status,
            server_rx_mtu,
            state.att_mtu_effective
        );
    })
}

pub(crate) unsafe fn on_timeout(_ble_evt: *const raw::ble_evt_t, gattc_evt: &raw::ble_gattc_evt_t) {
    trace!(
        "gattc on_timeout conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );
}

pub(crate) unsafe fn on_write_cmd_tx_complete(
    ble_evt: *const raw::ble_evt_t,
    gattc_evt: &raw::ble_gattc_evt_t,
) {
    trace!(
        "gattc on_write_cmd_tx_complete conn_handle={:u16} gatt_status={:u16}",
        gattc_evt.conn_handle,
        gattc_evt.gatt_status,
    );

    portal(gattc_evt.conn_handle).call(PortalMessage::WriteTxComplete(ble_evt))
}

static PORTALS: [Portal<PortalMessage>; CONNS_MAX] = [Portal::new(); CONNS_MAX];
pub(crate) fn portal(conn_handle: u16) -> &'static Portal<PortalMessage> {
    unsafe { &PORTALS[conn_handle as usize] }
}
