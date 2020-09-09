use heapless::consts::*;
use heapless::Vec;
use num_enum::{FromPrimitive, IntoPrimitive};

use crate::error::Error;
use crate::sd;
use crate::util::*;
use crate::uuid::Uuid;

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
pub enum DiscoveryError {
    ServiceNotFound,
    Gatt(GattError),
    Raw(Error),
}

static DISCOVER_SERVICE_PORTAL: Portal<Result<sd::ble_gattc_service_t, DiscoveryError>> =
    Portal::new();

pub(crate) async fn discover_service(
    conn_handle: u16,
    uuid: Uuid,
) -> Result<sd::ble_gattc_service_t, DiscoveryError> {
    let ret =
        unsafe { sd::sd_ble_gattc_primary_services_discover(conn_handle, 1, uuid.as_raw_ptr()) };
    match Error::convert(ret) {
        Ok(_) => {}
        Err(err) => {
            warn!("sd_ble_gattc_primary_services_discover err {:?}", err);
            return Err(DiscoveryError::Raw(err));
        }
    };

    DISCOVER_SERVICE_PORTAL.wait().await
}

pub(crate) unsafe fn on_prim_srvc_disc_rsp(
    ble_evt: *const sd::ble_evt_t,
    gattc_evt: &sd::ble_gattc_evt_t,
) {
    let val = if gattc_evt.gatt_status as u32 == sd::BLE_GATT_STATUS_SUCCESS {
        let params = get_union_field(ble_evt, &gattc_evt.params.prim_srvc_disc_rsp);
        let v = get_flexarray(ble_evt, &params.services, params.count as usize);

        match v.len() {
            0 => Err(DiscoveryError::ServiceNotFound),
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
        Err(DiscoveryError::Gatt(GattError::from(
            gattc_evt.gatt_status as u32,
        )))
    };
    DISCOVER_SERVICE_PORTAL.signal(val);
}

// =============================

type DiscCharsMax = U6;

static DISCOVER_CHARS_PORTAL: Portal<
    Result<Vec<sd::ble_gattc_char_t, DiscCharsMax>, DiscoveryError>,
> = Portal::new();

pub(crate) async fn discover_chars(
    conn_handle: u16,
    start_handle: u16,
    end_handle: u16,
) -> Result<Vec<sd::ble_gattc_char_t, DiscCharsMax>, DiscoveryError> {
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
            return Err(DiscoveryError::Raw(err));
        }
    };

    DISCOVER_CHARS_PORTAL.wait().await
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
        Err(DiscoveryError::Gatt(GattError::from(
            gattc_evt.gatt_status as u32,
        )))
    };
    DISCOVER_CHARS_PORTAL.signal(val);
}

// =============================

type DiscDescsMax = U6;

static DISCOVER_DESCS_PORTAL: Portal<
    Result<Vec<sd::ble_gattc_desc_t, DiscDescsMax>, DiscoveryError>,
> = Portal::new();

pub(crate) async fn discover_descs(
    conn_handle: u16,
    start_handle: u16,
    end_handle: u16,
) -> Result<Vec<sd::ble_gattc_desc_t, DiscDescsMax>, DiscoveryError> {
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
            return Err(DiscoveryError::Raw(err));
        }
    };

    DISCOVER_DESCS_PORTAL.wait().await
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
        Err(DiscoveryError::Gatt(GattError::from(
            gattc_evt.gatt_status as u32,
        )))
    };
    DISCOVER_DESCS_PORTAL.signal(val);
}

pub(crate) async fn discover(conn_handle: u16, uuid: Uuid) -> Result<(), DiscoveryError> {
    // TODO this hangs forever if connection is disconnected during discovery.

    let svc = match discover_service(conn_handle, uuid).await {
        Err(DiscoveryError::Gatt(GattError::AtterrAttributeNotFound)) => {
            Err(DiscoveryError::ServiceNotFound)
        }
        x => x,
    }?;

    let discover_char = |curr: sd::ble_gattc_char_t, next: Option<sd::ble_gattc_char_t>| async move {
        info!(
            "char: handle_decl={:u16} handle_value={:u16} uuid={:u8}:{:u16}",
            curr.handle_decl, curr.handle_value, curr.uuid.type_, curr.uuid.uuid
        );

        // Calcuate range of possible descriptors
        let start_handle = curr.handle_value + 1;
        let end_handle = next
            .map(|c| c.handle_decl - 1)
            .unwrap_or(svc.handle_range.end_handle);

        // Only if range is non-empty, discover. (if it's empty there must be no descriptors)
        if start_handle <= end_handle {
            let descs = discover_descs(conn_handle, start_handle, end_handle).await?;
            for desc in descs.iter() {
                info!(
                    "    desc: handle={:u16} uuid={:u8}:{:u16}",
                    desc.handle, desc.uuid.type_, desc.uuid.uuid
                );
            }
        }

        Ok(())
    };

    let mut curr_handle = svc.handle_range.start_handle;
    let end_handle = svc.handle_range.end_handle;

    let mut prev_char: Option<sd::ble_gattc_char_t> = None;
    while curr_handle < end_handle {
        let chars = match discover_chars(conn_handle, curr_handle, end_handle).await {
            Err(DiscoveryError::Gatt(GattError::AtterrAttributeNotFound)) => break,
            x => x,
        }?;
        deassert!(chars.len() != 0);
        for curr in chars {
            if let Some(prev) = prev_char {
                discover_char(prev, Some(curr)).await?;
            }
            prev_char = Some(curr);
            curr_handle = curr.handle_value + 1;
        }
    }
    if let Some(prev) = prev_char {
        discover_char(prev, None).await?;
    }

    Ok(())
}
