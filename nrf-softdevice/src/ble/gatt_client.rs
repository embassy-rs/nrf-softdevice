use heapless::consts::*;
use heapless::Vec;
use num_enum::{FromPrimitive, IntoPrimitive};

use crate::error::Error;
use crate::sd;
use crate::util::*;
use crate::uuid::Uuid;
use crate::DisconnectedError;
use crate::{Connection, ConnectionState, Role};

pub(crate) unsafe fn on_rel_disc_rsp(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gattc_evt_t,
) {
}

pub(crate) unsafe fn on_attr_info_disc_rsp(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gattc_evt_t,
) {
}

pub(crate) unsafe fn on_char_val_by_uuid_read_rsp(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gattc_evt_t,
) {
}

pub(crate) unsafe fn on_read_rsp(_ble_evt: *const sd::ble_evt_t, _gattc_evt: &sd::ble_gattc_evt_t) {
}

pub(crate) unsafe fn on_char_vals_read_rsp(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gattc_evt_t,
) {
}

pub(crate) unsafe fn on_write_rsp(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gattc_evt_t,
) {
}

pub(crate) unsafe fn on_hvx(_ble_evt: *const sd::ble_evt_t, _gattc_evt: &sd::ble_gattc_evt_t) {}

pub(crate) unsafe fn on_exchange_mtu_rsp(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gattc_evt_t,
) {
}

pub(crate) unsafe fn on_timeout(_ble_evt: *const sd::ble_evt_t, _gattc_evt: &sd::ble_gattc_evt_t) {}

pub(crate) unsafe fn on_write_cmd_tx_complete(
    _ble_evt: *const sd::ble_evt_t,
    _gattc_evt: &sd::ble_gattc_evt_t,
) {
}

#[rustfmt::skip]
#[repr(u32)]
#[derive(defmt::Format, IntoPrimitive, FromPrimitive)]
pub enum GattError {
    // This is not really an error, but IMO it's better to add it
    // anyway, just in case someone mistakenly converts BLE_GATT_STATUS_SUCCESS into GattError.
    // if they see "Success" they'll easily realize their mistake, if they see "Unknown" it'd be confusing.
    Success = sd::BLE_GATT_STATUS_SUCCESS,

    #[num_enum(default)]
    Unknown = sd::BLE_GATT_STATUS_UNKNOWN,

    AtterrInvalid = sd::BLE_GATT_STATUS_ATTERR_INVALID,
    AtterrInvalidHandle = sd::BLE_GATT_STATUS_ATTERR_INVALID_HANDLE,
    AtterrReadNotPermitted = sd::BLE_GATT_STATUS_ATTERR_READ_NOT_PERMITTED,
    AtterrWriteNotPermitted = sd::BLE_GATT_STATUS_ATTERR_WRITE_NOT_PERMITTED,
    AtterrInvalidPdu = sd::BLE_GATT_STATUS_ATTERR_INVALID_PDU,
    AtterrInsufAuthentication = sd::BLE_GATT_STATUS_ATTERR_INSUF_AUTHENTICATION,
    AtterrRequestNotSupported = sd::BLE_GATT_STATUS_ATTERR_REQUEST_NOT_SUPPORTED,
    AtterrInvalidOffset = sd::BLE_GATT_STATUS_ATTERR_INVALID_OFFSET,
    AtterrInsufAuthorization = sd::BLE_GATT_STATUS_ATTERR_INSUF_AUTHORIZATION,
    AtterrPrepareQueueFull = sd::BLE_GATT_STATUS_ATTERR_PREPARE_QUEUE_FULL,
    AtterrAttributeNotFound = sd::BLE_GATT_STATUS_ATTERR_ATTRIBUTE_NOT_FOUND,
    AtterrAttributeNotLong = sd::BLE_GATT_STATUS_ATTERR_ATTRIBUTE_NOT_LONG,
    AtterrInsufEncKeySize = sd::BLE_GATT_STATUS_ATTERR_INSUF_ENC_KEY_SIZE,
    AtterrInvalidAttValLength = sd::BLE_GATT_STATUS_ATTERR_INVALID_ATT_VAL_LENGTH,
    AtterrUnlikelyError = sd::BLE_GATT_STATUS_ATTERR_UNLIKELY_ERROR,
    AtterrInsufEncryption = sd::BLE_GATT_STATUS_ATTERR_INSUF_ENCRYPTION,
    AtterrUnsupportedGroupType = sd::BLE_GATT_STATUS_ATTERR_UNSUPPORTED_GROUP_TYPE,
    AtterrInsufResources = sd::BLE_GATT_STATUS_ATTERR_INSUF_RESOURCES,
    AtterrCpsWriteReqRejected = sd::BLE_GATT_STATUS_ATTERR_CPS_WRITE_REQ_REJECTED,
    AtterrCpsCccdConfigError = sd::BLE_GATT_STATUS_ATTERR_CPS_CCCD_CONFIG_ERROR,
    AtterrCpsProcAlrInProg = sd::BLE_GATT_STATUS_ATTERR_CPS_PROC_ALR_IN_PROG,
    AtterrCpsOutOfRange = sd::BLE_GATT_STATUS_ATTERR_CPS_OUT_OF_RANGE,
}

#[derive(defmt::Format)]
pub enum DiscoverError {
    Disconnected,
    ServiceNotFound,
    ServiceIncomplete,
    Gatt(GattError),
    Raw(Error),
}

impl From<DisconnectedError> for DiscoverError {
    fn from(_: DisconnectedError) -> Self {
        DiscoverError::Disconnected
    }
}

impl From<GattError> for DiscoverError {
    fn from(err: GattError) -> Self {
        DiscoverError::Gatt(err)
    }
}

impl From<Error> for DiscoverError {
    fn from(err: Error) -> Self {
        DiscoverError::Raw(err)
    }
}

type DiscCharsMax = U6;
type DiscDescsMax = U6;

pub(crate) enum PortalMessage {
    DiscoverService(Result<sd::ble_gattc_service_t, DiscoverError>),
    DiscoverCharacteristics(Result<Vec<sd::ble_gattc_char_t, DiscCharsMax>, DiscoverError>),
    DiscoverDescriptors(Result<Vec<sd::ble_gattc_desc_t, DiscDescsMax>, DiscoverError>),
    Disconnected,
}

pub(crate) async fn discover_service(
    conn: &ConnectionState,
    uuid: Uuid,
) -> Result<sd::ble_gattc_service_t, DiscoverError> {
    let conn_handle = conn.check_connected()?;
    let ret =
        unsafe { sd::sd_ble_gattc_primary_services_discover(conn_handle, 1, uuid.as_raw_ptr()) };
    match Error::convert(ret) {
        Ok(_) => {}
        Err(err) => {
            warn!("sd_ble_gattc_primary_services_discover err {:?}", err);
            return Err(DiscoverError::Raw(err));
        }
    };

    match conn.gattc_portal.wait().await {
        PortalMessage::DiscoverService(r) => r,
        PortalMessage::Disconnected => Err(DiscoverError::Disconnected),
        _ => unreachable!(),
    }
}

pub(crate) unsafe fn on_prim_srvc_disc_rsp(
    ble_evt: *const sd::ble_evt_t,
    gattc_evt: &sd::ble_gattc_evt_t,
) {
    let val = if gattc_evt.gatt_status as u32 == sd::BLE_GATT_STATUS_SUCCESS {
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
    } else {
        Err(DiscoverError::Gatt(GattError::from(
            gattc_evt.gatt_status as u32,
        )))
    };

    ConnectionState::by_conn_handle(gattc_evt.conn_handle)
        .gattc_portal
        .signal(PortalMessage::DiscoverService(val))
}

// =============================

pub(crate) async fn discover_chars(
    conn: &ConnectionState,
    start_handle: u16,
    end_handle: u16,
) -> Result<Vec<sd::ble_gattc_char_t, DiscCharsMax>, DiscoverError> {
    let conn_handle = conn.check_connected()?;
    let ret = unsafe {
        sd::sd_ble_gattc_characteristics_discover(
            conn_handle,
            &sd::ble_gattc_handle_range_t {
                start_handle,
                end_handle,
            },
        )
    };
    match Error::convert(ret) {
        Ok(_) => {}
        Err(err) => {
            warn!("sd_ble_gattc_characteristics_discover err {:?}", err);
            return Err(DiscoverError::Raw(err));
        }
    };

    match conn.gattc_portal.wait().await {
        PortalMessage::DiscoverCharacteristics(r) => r,
        PortalMessage::Disconnected => Err(DiscoverError::Disconnected),
        _ => unreachable!(),
    }
}

pub(crate) unsafe fn on_char_disc_rsp(
    ble_evt: *const sd::ble_evt_t,
    gattc_evt: &sd::ble_gattc_evt_t,
) {
    let val = if gattc_evt.gatt_status as u32 == sd::BLE_GATT_STATUS_SUCCESS {
        let params = get_union_field(ble_evt, &gattc_evt.params.char_disc_rsp);
        let v = get_flexarray(ble_evt, &params.chars, params.count as usize);
        let v = Vec::from_slice(v).unwrap_or_else(|_| {
            depanic!("too many gatt chars, increase DiscCharsMax: {:?}", v.len())
        });
        Ok(v)
    } else {
        Err(DiscoverError::Gatt(GattError::from(
            gattc_evt.gatt_status as u32,
        )))
    };
    ConnectionState::by_conn_handle(gattc_evt.conn_handle)
        .gattc_portal
        .signal(PortalMessage::DiscoverCharacteristics(val))
}

// =============================

pub(crate) async fn discover_descs(
    conn: &ConnectionState,
    start_handle: u16,
    end_handle: u16,
) -> Result<Vec<sd::ble_gattc_desc_t, DiscDescsMax>, DiscoverError> {
    let conn_handle = conn.check_connected()?;
    let ret = unsafe {
        sd::sd_ble_gattc_descriptors_discover(
            conn_handle,
            &sd::ble_gattc_handle_range_t {
                start_handle,
                end_handle,
            },
        )
    };
    match Error::convert(ret) {
        Ok(_) => {}
        Err(err) => {
            warn!("sd_ble_gattc_characteristics_discover err {:?}", err);
            return Err(DiscoverError::Raw(err));
        }
    };

    match conn.gattc_portal.wait().await {
        PortalMessage::DiscoverDescriptors(r) => r,
        PortalMessage::Disconnected => Err(DiscoverError::Disconnected),
        _ => unreachable!(),
    }
}

pub(crate) unsafe fn on_desc_disc_rsp(
    ble_evt: *const sd::ble_evt_t,
    gattc_evt: &sd::ble_gattc_evt_t,
) {
    let val = if gattc_evt.gatt_status as u32 == sd::BLE_GATT_STATUS_SUCCESS {
        let params = get_union_field(ble_evt, &gattc_evt.params.desc_disc_rsp);
        let v = get_flexarray(ble_evt, &params.descs, params.count as usize);
        let v = Vec::from_slice(v).unwrap_or_else(|_| {
            depanic!("too many gatt descs, increase DiscDescsMax: {:?}", v.len())
        });
        Ok(v)
    } else {
        Err(DiscoverError::Gatt(GattError::from(
            gattc_evt.gatt_status as u32,
        )))
    };

    ConnectionState::by_conn_handle(gattc_evt.conn_handle)
        .gattc_portal
        .signal(PortalMessage::DiscoverDescriptors(val))
}

async fn discover_char<T: Client>(
    client: &mut T,
    conn: &ConnectionState,
    svc: &sd::ble_gattc_service_t,
    curr: sd::ble_gattc_char_t,
    next: Option<sd::ble_gattc_char_t>,
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
        for desc in discover_descs(conn, start_handle, end_handle).await? {
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

pub async fn discover<T: Client>(conn: &Connection) -> Result<T, DiscoverError> {
    // TODO this hangs forever if connection is disconnected during discovery.
    // TODO handle drop. Probably doable gracefully (no DropBomb)

    let state = conn.state();

    let svc = match discover_service(state, T::uuid()).await {
        Err(DiscoverError::Gatt(GattError::AtterrAttributeNotFound)) => {
            Err(DiscoverError::ServiceNotFound)
        }
        x => x,
    }?;

    let mut client = T::new_undiscovered(conn.clone());

    let mut curr_handle = svc.handle_range.start_handle;
    let end_handle = svc.handle_range.end_handle;

    let mut prev_char: Option<sd::ble_gattc_char_t> = None;
    while curr_handle < end_handle {
        let chars = match discover_chars(state, curr_handle, end_handle).await {
            Err(DiscoverError::Gatt(GattError::AtterrAttributeNotFound)) => break,
            x => x,
        }?;
        deassert!(chars.len() != 0);
        for curr in chars {
            if let Some(prev) = prev_char {
                discover_char(&mut client, state, &svc, prev, Some(curr)).await?;
            }
            prev_char = Some(curr);
            curr_handle = curr.handle_value + 1;
        }
    }
    if let Some(prev) = prev_char {
        discover_char(&mut client, state, &svc, prev, None).await?;
    }

    client.discovery_complete()?;

    Ok(client)
}

pub struct Characteristic {
    pub uuid: Option<Uuid>,
    pub handle_decl: u16,
    pub handle_value: u16,
    pub props: sd::ble_gatt_char_props_t,
    pub has_ext_props: bool,
}

pub struct Descriptor {
    pub uuid: Option<Uuid>,
    pub handle: u16,
}

pub trait Client {
    fn uuid() -> Uuid;
    fn new_undiscovered(conn: Connection) -> Self;
    fn discovered_characteristic(
        &mut self,
        characteristic: &Characteristic,
        descriptors: &[Descriptor],
    );
    fn discovery_complete(&mut self) -> Result<(), DiscoverError>;
}
