//! Generic Attribute server. GATT servers offer functionality to clients.
//!
//! Typically the peripheral device is the GATT server, but it is not necessary.
//! In a connection any device can be server and client, and even both can be both at the same time.

use crate::ble::*;
use crate::util::{get_flexarray, get_union_field, Portal};
use crate::{raw, RawError, Softdevice};

pub mod builder;
pub mod characteristic;

pub struct Characteristic {
    pub uuid: Uuid,
    pub can_read: bool,
    pub can_write: bool,
    pub can_write_without_response: bool,
    pub can_notify: bool,
    pub can_indicate: bool,
    pub max_len: usize,
    pub vlen: bool,
}

pub struct CharacteristicHandles {
    pub value_handle: u16,
    pub user_desc_handle: u16,
    pub cccd_handle: u16,
    pub sccd_handle: u16,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ServiceHandle(u16);

impl ServiceHandle {
    pub fn handle(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IncludedServiceHandle(u16);

impl IncludedServiceHandle {
    pub fn handle(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DescriptorHandle(u16);

impl DescriptorHandle {
    pub fn handle(&self) -> u16 {
        self.0
    }
}

pub trait Server: Sized {
    type Event;
    fn on_write(&self, handle: u16, data: &[u8]) -> Option<Self::Event>;
}

pub trait Service: Sized {
    type Event;

    fn on_write(&self, handle: u16, data: &[u8]) -> Option<Self::Event>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RegisterError {
    Raw(RawError),
}

impl From<RawError> for RegisterError {
    fn from(err: RawError) -> Self {
        RegisterError::Raw(err)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RunError {
    Disconnected,
    Raw(RawError),
}

impl From<RawError> for RunError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

impl From<DisconnectedError> for RunError {
    fn from(_: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

pub async fn run<'m, F, S>(conn: &Connection, server: &S, mut f: F) -> Result<(), RunError>
where
    F: FnMut(S::Event),
    S: Server,
{
    let conn_handle = conn.with_state(|state| state.check_connected())?;
    portal(conn_handle)
        .wait_many(|ble_evt| unsafe {
            match (*ble_evt).header.evt_id as u32 {
                raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => return Some(Err(RunError::Disconnected)),
                raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_WRITE => {
                    let evt = &*ble_evt;
                    let gatts_evt = get_union_field(ble_evt, &evt.evt.gatts_evt);
                    let params = get_union_field(ble_evt, &gatts_evt.params.write);
                    let v = get_flexarray(ble_evt, &params.data, params.len as usize);
                    trace!("gatts write handle={:?} data={:?}", params.handle, v);
                    if params.offset != 0 {
                        panic!("gatt_server writes with nonzero offset are not yet supported");
                    }
                    if params.auth_required != 0 {
                        panic!("gatt_server auth_required not yet supported");
                    }

                    server.on_write(params.handle, &v).map(|e| f(e));
                }
                raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_SYS_ATTR_MISSING => {
                    debug!("initializing gatt sys att");
                    let ret = raw::sd_ble_gatts_sys_attr_set(conn_handle, ::core::ptr::null(), 0, 0);
                    RawError::convert(ret).err();
                }
                _ => {}
            }
            None
        })
        .await
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GetValueError {
    Truncated,
    Raw(RawError),
}

impl From<RawError> for GetValueError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

pub fn get_value(_sd: &Softdevice, handle: u16, buf: &mut [u8]) -> Result<usize, GetValueError> {
    let mut value = raw::ble_gatts_value_t {
        p_value: buf.as_mut_ptr(),
        len: buf.len() as _,
        offset: 0,
    };
    let ret = unsafe { raw::sd_ble_gatts_value_get(raw::BLE_CONN_HANDLE_INVALID as u16, handle, &mut value) };
    RawError::convert(ret)?;

    if value.len as usize > buf.len() {
        return Err(GetValueError::Truncated);
    }

    Ok(value.len as _)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SetValueError {
    Truncated,
    Raw(RawError),
}

impl From<RawError> for SetValueError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

pub fn set_value(_sd: &Softdevice, handle: u16, val: &[u8]) -> Result<(), SetValueError> {
    let mut value = raw::ble_gatts_value_t {
        p_value: val.as_ptr() as _,
        len: val.len() as _,
        offset: 0,
    };
    let ret = unsafe { raw::sd_ble_gatts_value_set(raw::BLE_CONN_HANDLE_INVALID as u16, handle, &mut value) };
    RawError::convert(ret)?;

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum NotifyValueError {
    Disconnected,
    Raw(RawError),
}

impl From<RawError> for NotifyValueError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

impl From<DisconnectedError> for NotifyValueError {
    fn from(_: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

/// Multiple notifications can be queued. Will fail when the queue is full.
pub fn notify_value(conn: &Connection, handle: u16, val: &[u8]) -> Result<(), NotifyValueError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;

    let mut len: u16 = val.len() as _;
    let params = raw::ble_gatts_hvx_params_t {
        handle,
        type_: raw::BLE_GATT_HVX_NOTIFICATION as u8,
        offset: 0,
        p_data: val.as_ptr() as _,
        p_len: &mut len,
    };
    let ret = unsafe { raw::sd_ble_gatts_hvx(conn_handle, &params) };
    RawError::convert(ret)?;

    Ok(())
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IndicateValueError {
    Disconnected,
    Raw(RawError),
}

impl From<RawError> for IndicateValueError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

impl From<DisconnectedError> for IndicateValueError {
    fn from(_: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

/// This will fail if an indication is already in progress
pub fn indicate_value(conn: &Connection, handle: u16, val: &[u8]) -> Result<(), IndicateValueError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;

    let mut len: u16 = val.len() as _;
    let params = raw::ble_gatts_hvx_params_t {
        handle,
        type_: raw::BLE_GATT_HVX_INDICATION as u8,
        offset: 0,
        p_data: val.as_ptr() as _,
        p_len: &mut len,
    };
    let ret = unsafe { raw::sd_ble_gatts_hvx(conn_handle, &params) };
    RawError::convert(ret)?;

    Ok(())
}

pub(crate) unsafe fn on_evt(ble_evt: *const raw::ble_evt_t) {
    let gatts_evt = get_union_field(ble_evt, &(*ble_evt).evt.gatts_evt);
    match (*ble_evt).header.evt_id as u32 {
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_EXCHANGE_MTU_REQUEST => {
            let conn_handle = gatts_evt.conn_handle;
            let params = get_union_field(ble_evt, &gatts_evt.params.exchange_mtu_request);
            let want_mtu = params.client_rx_mtu;
            let max_mtu = crate::Softdevice::steal().att_mtu;
            let mtu = want_mtu.min(max_mtu).max(raw::BLE_GATT_ATT_MTU_DEFAULT as u16);
            trace!("att mtu exchange: peer wants mtu {:?}, granting {:?}", want_mtu, mtu);

            let ret = { raw::sd_ble_gatts_exchange_mtu_reply(conn_handle, mtu) };
            if let Err(_err) = RawError::convert(ret) {
                warn!("sd_ble_gatts_exchange_mtu_reply err {:?}", _err);
                return;
            }

            connection::with_state_by_conn_handle(conn_handle, |state| {
                state.att_mtu = mtu;
            });
        }
        _ => {
            portal(gatts_evt.conn_handle).call(ble_evt);
        }
    }
}

const PORTAL_NEW: Portal<*const raw::ble_evt_t> = Portal::new();
static PORTALS: [Portal<*const raw::ble_evt_t>; CONNS_MAX] = [PORTAL_NEW; CONNS_MAX];
pub(crate) fn portal(conn_handle: u16) -> &'static Portal<*const raw::ble_evt_t> {
    &PORTALS[conn_handle as usize]
}
