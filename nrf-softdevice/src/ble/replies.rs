use core::mem::ManuallyDrop;

use super::Connection;
use crate::{raw, RawError};

#[cfg(feature = "ble-bond")]
pub struct PasskeyReply {
    conn: ManuallyDrop<Connection>,
}

#[cfg(feature = "ble-bond")]
impl Drop for PasskeyReply {
    fn drop(&mut self) {
        if let Some(conn_handle) = self.conn.handle() {
            let ret = unsafe {
                raw::sd_ble_gap_auth_key_reply(conn_handle, raw::BLE_GAP_AUTH_KEY_TYPE_NONE as u8, core::ptr::null())
            };

            if let Err(_err) = RawError::convert(ret) {
                warn!("sd_ble_gap_auth_key_reply err {:?}", _err);
            }
        }

        // Since conn is ManuallyDrop, we must drop it here
        unsafe { ManuallyDrop::drop(&mut self.conn) };
    }
}

#[cfg(feature = "ble-bond")]
impl PasskeyReply {
    pub(crate) fn new(conn: Connection) -> Self {
        Self {
            conn: ManuallyDrop::new(conn),
        }
    }

    pub fn reply(mut self, passkey: Option<&[u8; 6]>) -> Result<(), RawError> {
        let res = if let Some(conn_handle) = self.conn.handle() {
            let ptr = passkey.map(|x| x.as_ptr()).unwrap_or(core::ptr::null());
            RawError::convert(unsafe {
                raw::sd_ble_gap_auth_key_reply(conn_handle, raw::BLE_GAP_AUTH_KEY_TYPE_PASSKEY as u8, ptr)
            })
        } else {
            Err(RawError::InvalidState)
        };

        // Drop the connection but forget self so `sd_ble_gap_auth_key_reply` is not called twice.
        unsafe { ManuallyDrop::drop(&mut self.conn) };
        core::mem::forget(self);

        res
    }
}

#[cfg(feature = "ble-bond")]
pub struct OutOfBandReply {
    conn: ManuallyDrop<Connection>,
}

#[cfg(feature = "ble-bond")]
impl Drop for OutOfBandReply {
    fn drop(&mut self) {
        if let Some(conn_handle) = self.conn.handle() {
            let ret = unsafe {
                raw::sd_ble_gap_auth_key_reply(conn_handle, raw::BLE_GAP_AUTH_KEY_TYPE_NONE as u8, core::ptr::null())
            };

            if let Err(_err) = RawError::convert(ret) {
                warn!("sd_ble_gap_auth_key_reply err {:?}", _err);
            }
        }

        // Since conn is ManuallyDrop, we must drop it here
        unsafe { ManuallyDrop::drop(&mut self.conn) };
    }
}

#[cfg(feature = "ble-bond")]
impl OutOfBandReply {
    pub(crate) fn new(conn: Connection) -> Self {
        Self {
            conn: ManuallyDrop::new(conn),
        }
    }

    pub fn reply(mut self, oob: Option<&[u8; 16]>) -> Result<(), RawError> {
        let res = if let Some(conn_handle) = self.conn.handle() {
            let ptr = oob.map(|x| x.as_ptr()).unwrap_or(core::ptr::null());
            RawError::convert(unsafe {
                raw::sd_ble_gap_auth_key_reply(conn_handle, raw::BLE_GAP_AUTH_KEY_TYPE_OOB as u8, ptr)
            })
        } else {
            Err(RawError::InvalidState)
        };

        // Drop the connection but forget self so `sd_ble_gap_auth_key_reply` is not called twice.
        unsafe { ManuallyDrop::drop(&mut self.conn) };
        core::mem::forget(self);

        res
    }
}

pub struct SysAttrsReply {
    conn: ManuallyDrop<Connection>,
}

impl Drop for SysAttrsReply {
    fn drop(&mut self) {
        if let Some(conn_handle) = self.conn.handle() {
            let ret = unsafe { raw::sd_ble_gatts_sys_attr_set(conn_handle, core::ptr::null(), 0, 0) };

            if let Err(_err) = RawError::convert(ret) {
                warn!("sd_ble_gatts_sys_attr_set err {:?}", _err);
            }
        }

        // Since conn is ManuallyDrop, we must drop it here
        unsafe { ManuallyDrop::drop(&mut self.conn) };
    }
}

impl SysAttrsReply {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn: ManuallyDrop::new(conn),
        }
    }

    pub fn connection(&self) -> &Connection {
        &*self.conn
    }

    pub fn set_sys_attrs(mut self, sys_attrs: Option<&[u8]>) -> Result<(), RawError> {
        let res = if let Some(conn_handle) = self.conn.handle() {
            let ptr = sys_attrs.map(|x| x.as_ptr()).unwrap_or(core::ptr::null());
            let len = sys_attrs.map(|x| x.len()).unwrap_or_default();

            let ret = unsafe { raw::sd_ble_gatts_sys_attr_set(conn_handle, ptr, unwrap!(len.try_into()), 0) };
            RawError::convert(ret)
        } else {
            Err(RawError::InvalidState)
        };

        // Drop the connection but forget self so `sd_ble_gatts_sys_attr_set` is not called twice.
        unsafe { ManuallyDrop::drop(&mut self.conn) };
        core::mem::forget(self);

        res
    }
}
