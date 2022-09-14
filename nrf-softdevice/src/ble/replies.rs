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
