use core::mem::ManuallyDrop;

use super::Connection;
use crate::{raw, RawError};

#[cfg(feature = "ble-sec")]
pub struct PasskeyReply {
    conn: ManuallyDrop<Connection>,
}

#[cfg(feature = "ble-sec")]
impl Drop for PasskeyReply {
    fn drop(&mut self) {
        if let Err(_err) = unsafe { self.finalize(None) } {
            warn!("sd_ble_gap_auth_key_reply err {:?}", _err);
        }
    }
}

#[cfg(feature = "ble-sec")]
impl PasskeyReply {
    pub(crate) fn new(conn: Connection) -> Self {
        Self {
            conn: ManuallyDrop::new(conn),
        }
    }

    pub fn reply(mut self, passkey: Option<&[u8; 6]>) -> Result<(), RawError> {
        let res = unsafe { self.finalize(passkey) };
        core::mem::forget(self); // Prevent Drop from finalizing a second time
        res
    }

    /// # Safety
    ///
    /// This method must be called exactly once
    unsafe fn finalize(&mut self, passkey: Option<&[u8; 6]>) -> Result<(), RawError> {
        let res = if let Some(conn_handle) = self.conn.handle() {
            let ptr = passkey.map(|x| x.as_ptr()).unwrap_or(core::ptr::null());
            let ret = raw::sd_ble_gap_auth_key_reply(conn_handle, raw::BLE_GAP_AUTH_KEY_TYPE_PASSKEY as u8, ptr);
            RawError::convert(ret)
        } else {
            Err(RawError::InvalidState)
        };

        // Since conn is ManuallyDrop, we must drop it here
        ManuallyDrop::drop(&mut self.conn);
        res
    }
}

#[cfg(feature = "ble-sec")]
pub struct OutOfBandReply {
    conn: ManuallyDrop<Connection>,
}

#[cfg(feature = "ble-sec")]
impl Drop for OutOfBandReply {
    fn drop(&mut self) {
        if let Err(_err) = unsafe { self.finalize(None) } {
            warn!("sd_ble_gap_auth_key_reply err {:?}", _err);
        }
    }
}

#[cfg(feature = "ble-sec")]
impl OutOfBandReply {
    pub(crate) fn new(conn: Connection) -> Self {
        Self {
            conn: ManuallyDrop::new(conn),
        }
    }

    pub fn reply(mut self, oob: Option<&[u8; 16]>) -> Result<(), RawError> {
        let res = unsafe { self.finalize(oob) };
        core::mem::forget(self); // Prevent Drop from finalizing a second time
        res
    }

    /// # Safety
    ///
    /// This method must be called exactly once
    unsafe fn finalize(&mut self, oob: Option<&[u8; 16]>) -> Result<(), RawError> {
        let res = if let Some(conn_handle) = self.conn.handle() {
            let ptr = oob.map(|x| x.as_ptr()).unwrap_or(core::ptr::null());
            let ret = raw::sd_ble_gap_auth_key_reply(conn_handle, raw::BLE_GAP_AUTH_KEY_TYPE_OOB as u8, ptr);
            RawError::convert(ret)
        } else {
            Err(RawError::InvalidState)
        };

        // Since conn is ManuallyDrop, we must drop it here
        ManuallyDrop::drop(&mut self.conn);
        res
    }
}

pub struct SysAttrsReply {
    conn: ManuallyDrop<Connection>,
}

impl Drop for SysAttrsReply {
    fn drop(&mut self) {
        if let Err(_err) = unsafe { self.finalize(None) } {
            warn!("sd_ble_gatts_sys_attr_set err {:?}", _err);
        }
    }
}

impl SysAttrsReply {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: ManuallyDrop::new(conn),
        }
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn set_sys_attrs(mut self, sys_attrs: Option<&[u8]>) -> Result<(), RawError> {
        let res = unsafe { self.finalize(sys_attrs) };
        core::mem::forget(self); // Prevent Drop from finalizing a second time
        res
    }

    /// # Safety
    ///
    /// This method must be called exactly once
    unsafe fn finalize(&mut self, sys_attrs: Option<&[u8]>) -> Result<(), RawError> {
        let res = if let Some(conn_handle) = self.conn.handle() {
            let ptr = sys_attrs.map(|x| x.as_ptr()).unwrap_or(core::ptr::null());
            let len = sys_attrs.map(|x| x.len()).unwrap_or_default();
            let ret = raw::sd_ble_gatts_sys_attr_set(conn_handle, ptr, unwrap!(len.try_into()), 0);
            RawError::convert(ret)
        } else {
            Err(RawError::InvalidState)
        };

        // Since conn is ManuallyDrop, we must drop it here
        ManuallyDrop::drop(&mut self.conn);
        res
    }
}

#[cfg(feature = "ble-gatt-server")]
const DEFERRED_TYPE_READ: u8 = raw::BLE_GATTS_AUTHORIZE_TYPE_READ as u8;
#[cfg(feature = "ble-gatt-server")]
const DEFERRED_TYPE_WRITE: u8 = raw::BLE_GATTS_AUTHORIZE_TYPE_WRITE as u8;

#[cfg(feature = "ble-gatt-server")]
struct DeferredReply<const DEFERRED_TYPE: u8> {
    conn: ManuallyDrop<Connection>,
}

#[cfg(feature = "ble-gatt-server")]
impl<const DEFERRED_TYPE: u8> Drop for DeferredReply<DEFERRED_TYPE> {
    fn drop(&mut self) {
        warn!("DeferredReply<{}> dropped without reply", DEFERRED_TYPE);
        let res = unsafe { self.finalize(DEFERRED_TYPE, Err(super::GattError::AtterrAttributeNotFound)) };

        if let Err(_err) = res {
            warn!("sd_ble_gatts_rw_authorize_reply err {:?}", _err);
        }
    }
}

#[cfg(feature = "ble-gatt-server")]
impl<const DEFERRED_TYPE: u8> DeferredReply<DEFERRED_TYPE> {
    fn reply(mut self, res: Result<Option<&[u8]>, super::GattError>) -> Result<(), RawError> {
        assert!(res != Err(super::GattError::Success));
        let res = unsafe { self.finalize(DEFERRED_TYPE, res) };
        core::mem::forget(self);
        res
    }

    /// # Safety
    ///
    /// This method must be called exactly once
    unsafe fn finalize(
        &mut self,
        deferred_type: u8,
        res: Result<Option<&[u8]>, super::GattError>,
    ) -> Result<(), RawError> {
        let (gatt_status, update, p_data, len) = match res {
            Ok(Some(data)) => (super::GattError::Success, true, data.as_ptr(), data.len()),
            Ok(None) => (super::GattError::Success, false, core::ptr::null(), 0),
            Err(status) => (status, false, core::ptr::null(), 0),
        };

        let res = if let Some(handle) = self.conn.handle() {
            let params = raw::ble_gatts_authorize_params_t {
                gatt_status: u32::from(gatt_status) as u16,
                _bitfield_1: raw::ble_gatts_authorize_params_t::new_bitfield_1(u8::from(update)),
                offset: 0,
                len: len as u16,
                p_data,
            };

            let reply_params = raw::ble_gatts_rw_authorize_reply_params_t {
                type_: deferred_type,
                params: raw::ble_gatts_rw_authorize_reply_params_t__bindgen_ty_1 { read: params },
            };

            let ret = raw::sd_ble_gatts_rw_authorize_reply(handle, &reply_params);
            RawError::convert(ret)
        } else {
            Err(RawError::BleInvalidConnHandle)
        };

        // Since conn is ManuallyDrop, we must drop it here
        ManuallyDrop::drop(&mut self.conn);
        res
    }
}

#[cfg(feature = "ble-gatt-server")]
pub struct DeferredWriteReply(DeferredReply<DEFERRED_TYPE_WRITE>);

#[cfg(feature = "ble-gatt-server")]
/// Represents an in-progress deferred write request
impl DeferredWriteReply {
    pub(crate) fn new(conn: Connection) -> Self {
        DeferredWriteReply(DeferredReply {
            conn: ManuallyDrop::new(conn),
        })
    }

    pub fn conn(&self) -> &Connection {
        &self.0.conn
    }

    pub fn reply(self, res: Result<&[u8], super::GattError>) -> Result<(), RawError> {
        self.0.reply(res.map(Some))
    }
}

#[cfg(feature = "ble-gatt-server")]
pub struct DeferredReadReply(DeferredReply<DEFERRED_TYPE_READ>);

#[cfg(feature = "ble-gatt-server")]
/// Represents an in-progress deferred read request
impl DeferredReadReply {
    pub(crate) fn new(conn: Connection) -> Self {
        DeferredReadReply(DeferredReply {
            conn: ManuallyDrop::new(conn),
        })
    }

    pub fn conn(&self) -> &Connection {
        &self.0.conn
    }

    /// Finishes the read operation with `res`.
    ///
    /// If `res` is `Ok(None)`, the value of the attribute stored by the softdevice will be returned to the central.
    pub fn reply(self, res: Result<Option<&[u8]>, super::GattError>) -> Result<(), RawError> {
        self.0.reply(res)
    }
}
