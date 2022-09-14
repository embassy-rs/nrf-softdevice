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
        let res = unsafe { self.finalize(raw::BLE_GATT_STATUS_ATTERR_UNLIKELY_ERROR as u16, &[], false) };

        if let Err(_err) = res {
            warn!("sd_ble_gatts_rw_authorize_reply err {:?}", _err);
        }
    }
}

#[cfg(feature = "ble-gatt-server")]
impl<const DEFERRED_TYPE: u8> DeferredReply<DEFERRED_TYPE> {
    fn reply(mut self, status: u16, data: &[u8], update: bool) -> Result<(), RawError> {
        let res = unsafe { self.finalize(status, data.as_ref(), update) };
        core::mem::forget(self);
        res
    }

    /// # Safety
    ///
    /// This method must be called exactly once
    unsafe fn finalize(&mut self, status: u16, data: &[u8], update: bool) -> Result<(), RawError> {
        let res = if let Some(handle) = self.conn.handle() {
            let params = raw::ble_gatts_authorize_params_t {
                gatt_status: status,
                _bitfield_1: raw::ble_gatts_authorize_params_t::new_bitfield_1(u8::from(update)),
                offset: 0,
                len: data.len() as u16,
                p_data: data.as_ptr(),
            };

            let reply_params = raw::ble_gatts_rw_authorize_reply_params_t {
                type_: DEFERRED_TYPE,
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
impl DeferredWriteReply {
    pub(crate) fn new(conn: Connection) -> Self {
        DeferredWriteReply(DeferredReply {
            conn: ManuallyDrop::new(conn),
        })
    }

    pub fn conn(&self) -> &Connection {
        &self.0.conn
    }

    pub fn reply<T: AsRef<[u8]>>(self, status: u16, data: &T) -> Result<(), RawError> {
        let update = u32::from(status) == raw::BLE_GATT_STATUS_SUCCESS;
        self.0.reply(status, data.as_ref(), update)
    }
}

#[cfg(feature = "ble-gatt-server")]
pub struct DeferredReadReply(DeferredReply<DEFERRED_TYPE_READ>);

#[cfg(feature = "ble-gatt-server")]
impl DeferredReadReply {
    pub(crate) fn new(conn: Connection) -> Self {
        DeferredReadReply(DeferredReply {
            conn: ManuallyDrop::new(conn),
        })
    }

    pub fn conn(&self) -> &Connection {
        &self.0.conn
    }

    pub fn reply<T: AsRef<[u8]>>(self, status: u16, data: &T, update: bool) -> Result<(), RawError> {
        self.0.reply(status, data.as_ref(), update)
    }
}
