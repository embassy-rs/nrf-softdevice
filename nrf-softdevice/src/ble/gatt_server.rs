//! Generic Attribute server. GATT servers offer functionality to clients.
//!
//! Typically the peripheral device is the GATT server, but it is not necessary.
//! In a connection any device can be server and client, and even both can be both at the same time.

use core::convert::TryFrom;

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

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum WriteOp {
    Request = 1,
    Command,
    SignedWriteCommmand,
    PrepareWriteRequest,
    CancelPreparedWrites,
    ExecutePreparedWrites,
}

pub struct InvalidWriteOpError;

impl TryFrom<u8> for WriteOp {
    type Error = InvalidWriteOpError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match u32::from(value) {
            raw::BLE_GATTS_OP_WRITE_REQ => Ok(WriteOp::Request),
            raw::BLE_GATTS_OP_WRITE_CMD => Ok(WriteOp::Command),
            raw::BLE_GATTS_OP_SIGN_WRITE_CMD => Ok(WriteOp::SignedWriteCommmand),
            raw::BLE_GATTS_OP_PREP_WRITE_REQ => Ok(WriteOp::PrepareWriteRequest),
            raw::BLE_GATTS_OP_EXEC_WRITE_REQ_CANCEL => Ok(WriteOp::CancelPreparedWrites),
            raw::BLE_GATTS_OP_EXEC_WRITE_REQ_NOW => Ok(WriteOp::ExecutePreparedWrites),
            _ => Err(InvalidWriteOpError),
        }
    }
}

pub trait Server: Sized {
    type Event;

    fn on_write(&self, conn: &Connection, handle: u16, op: WriteOp, offset: usize, data: &[u8]) -> Option<Self::Event>;

    /// Handle reads of characteristics built with the
    /// [`deferred_read`][characteristic::AttributeMetadata::deferred_read] flag set.
    ///
    /// Your [Server] must provide an implementation of this method if any of your characteristics has that flag set.
    fn on_deferred_read(&self, handle: u16, offset: usize, reply: DeferredReadReply) -> Option<Self::Event> {
        let _ = (handle, offset, reply);
        panic!("on_deferred_read needs to be implemented for this gatt server");
    }

    /// Handle writes of characteristics built with the
    /// [`deferred_write`][characteristic::AttributeMetadata::deferred_write] flag set.
    ///
    /// Your [Server] must provide an implementation of this method if any of your characteristics has that flag set.
    fn on_deferred_write(
        &self,
        handle: u16,
        op: WriteOp,
        offset: usize,
        data: &[u8],
        reply: DeferredWriteReply,
    ) -> Option<Self::Event> {
        let _ = (handle, op, offset, data, reply);
        panic!("on_deferred_write needs to be implemented for this gatt server");
    }

    /// Callback to indicate that one or more characteristic notifications have been transmitted.
    fn on_notify_tx_complete(&self, conn: &Connection, count: u8) -> Option<Self::Event> {
        let _ = (conn, count);
        None
    }

    /// Callback to indicate that the indication of a characteristic has been received by the client.
    fn on_indicate_confirm(&self, conn: &Connection, handle: u16) -> Option<Self::Event> {
        let _ = (conn, handle);
        None
    }

    /// Callback to indicate that the services changed indication has been received by the client.
    fn on_services_changed_confirm(&self, conn: &Connection) -> Option<Self::Event> {
        let _ = conn;
        None
    }

    fn on_timeout(&self, conn: &Connection) -> Option<Self::Event> {
        let _ = conn;
        None
    }
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

pub async fn run<'m, F, S>(conn: &Connection, server: &S, mut f: F) -> DisconnectedError
where
    F: FnMut(S::Event),
    S: Server,
{
    let conn_handle = match conn.with_state(|state| state.check_connected()) {
        Ok(handle) => handle,
        Err(DisconnectedError) => return DisconnectedError,
    };

    portal(conn_handle)
        .wait_many(|ble_evt| unsafe {
            let ble_evt = &*ble_evt;
            if u32::from(ble_evt.header.evt_id) == raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED {
                return Some(DisconnectedError);
            }

            // If evt_id is not BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED, then it must be a GATTS event
            let gatts_evt = get_union_field(ble_evt, &ble_evt.evt.gatts_evt);
            let conn = unwrap!(Connection::from_handle(gatts_evt.conn_handle));
            let evt = match ble_evt.header.evt_id as u32 {
                raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_SYS_ATTR_MISSING => {
                    let _params = get_union_field(ble_evt, &gatts_evt.params.sys_attr_missing);
                    trace!("gatts sys attr missing conn={:?}", gatts_evt.conn_handle);

                    if let Some(conn) = Connection::from_handle(gatts_evt.conn_handle) {
                        #[cfg(feature = "ble-sec")]
                        if let Some(handler) = conn.security_handler() {
                            handler.load_sys_attrs(&conn);
                        } else if let Err(err) = set_sys_attrs(&conn, None) {
                            warn!("gatt_server failed to set sys attrs: {:?}", err);
                        }

                        #[cfg(not(feature = "ble-sec"))]
                        if let Err(err) = set_sys_attrs(&conn, None) {
                            warn!("gatt_server failed to set sys attrs: {:?}", err);
                        }
                    }

                    None
                }
                raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_WRITE => {
                    let params = get_union_field(ble_evt, &gatts_evt.params.write);
                    let offset = usize::from(params.offset);
                    let v = get_flexarray(ble_evt, &params.data, params.len as usize);
                    trace!("gatts write handle={:?} data={:?}", params.handle, v);

                    match params.op.try_into() {
                        Ok(op) => server.on_write(&conn, params.handle, op, offset, v),
                        Err(_) => {
                            error!("gatt_server invalid write op: {}", params.op);
                            None
                        }
                    }
                }
                raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_RW_AUTHORIZE_REQUEST => {
                    let params = get_union_field(ble_evt, &gatts_evt.params.authorize_request);
                    match params.type_ as u32 {
                        raw::BLE_GATTS_AUTHORIZE_TYPE_READ => {
                            let responder = DeferredReadReply::new(conn);
                            let params = get_union_field(ble_evt, &params.request.read);
                            trace!("gatts authorize read request handle={:?}", params.handle);
                            server.on_deferred_read(params.handle, usize::from(params.offset), responder)
                        }
                        raw::BLE_GATTS_AUTHORIZE_TYPE_WRITE => {
                            let responder = DeferredWriteReply::new(conn);
                            let params = get_union_field(ble_evt, &params.request.write);
                            let offset = usize::from(params.offset);
                            let v = get_flexarray(ble_evt, &params.data, params.len as usize);
                            trace!("gatts authorize write handle={:?} data={:?}", params.handle, v);

                            match params.op.try_into() {
                                Ok(op) => server.on_deferred_write(params.handle, op, offset, v, responder),
                                Err(_) => {
                                    error!("gatt_server invalid write op: {}", params.op);
                                    None
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
                }
                raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVN_TX_COMPLETE => {
                    let params = get_union_field(ble_evt, &gatts_evt.params.hvn_tx_complete);
                    server.on_notify_tx_complete(&conn, params.count)
                }
                raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVC => {
                    let params = get_union_field(ble_evt, &gatts_evt.params.hvc);
                    server.on_indicate_confirm(&conn, params.handle)
                }
                raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_SC_CONFIRM => server.on_services_changed_confirm(&conn),
                raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_TIMEOUT => server.on_timeout(&conn),
                _ => None,
            };

            if let Some(evt) = evt {
                f(evt)
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GetSysAttrsError {
    DataSize(usize),
    Disconnected,
    Raw(RawError),
}

impl From<DisconnectedError> for GetSysAttrsError {
    fn from(_: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

pub fn get_sys_attrs(conn: &Connection, buf: &mut [u8]) -> Result<usize, GetSysAttrsError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;

    let mut len = unwrap!(u16::try_from(buf.len()));
    let ret = unsafe { raw::sd_ble_gatts_sys_attr_get(conn_handle, buf.as_mut_ptr(), &mut len, 0) };
    match RawError::convert(ret) {
        Ok(()) => Ok(usize::from(len)),
        Err(RawError::DataSize) => Err(GetSysAttrsError::DataSize(usize::from(len))),
        Err(err) => Err(GetSysAttrsError::Raw(err)),
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SetSysAttrsError {
    Disconnected,
    Raw(RawError),
}

impl From<DisconnectedError> for SetSysAttrsError {
    fn from(_: DisconnectedError) -> Self {
        Self::Disconnected
    }
}

pub fn set_sys_attrs(conn: &Connection, sys_attrs: Option<&[u8]>) -> Result<(), SetSysAttrsError> {
    let conn_handle = conn.with_state(|state| state.check_connected())?;
    let ptr = sys_attrs.map(|x| x.as_ptr()).unwrap_or(core::ptr::null());
    let len = sys_attrs.map(|x| x.len()).unwrap_or_default();
    let ret = unsafe { raw::sd_ble_gatts_sys_attr_set(conn_handle, ptr, unwrap!(len.try_into()), 0) };
    RawError::convert(ret).map_err(SetSysAttrsError::Raw)
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
