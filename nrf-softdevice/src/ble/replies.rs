use super::Connection;
use crate::{raw, RawError};

#[cfg(feature = "ble-bond")]
pub struct PasskeyReply {
    conn: Option<Connection>,
}

#[cfg(feature = "ble-bond")]
impl Drop for PasskeyReply {
    fn drop(&mut self) {
        if let Some(conn_handle) = self.conn.take().and_then(|x| x.handle()) {
            let ret = unsafe {
                raw::sd_ble_gap_auth_key_reply(conn_handle, raw::BLE_GAP_AUTH_KEY_TYPE_NONE as u8, core::ptr::null())
            };

            if let Err(_err) = RawError::convert(ret) {
                warn!("sd_ble_gap_auth_key_reply err {:?}", _err);
            }
        }
    }
}

#[cfg(feature = "ble-bond")]
impl PasskeyReply {
    pub(crate) fn new(conn: Connection) -> Self {
        Self { conn: Some(conn) }
    }

    pub fn reply(mut self, passkey: &[u8; 6]) -> Result<(), RawError> {
        if let Some(conn_handle) = self.conn.take().and_then(|x| x.handle()) {
            unsafe {
                RawError::convert(raw::sd_ble_gap_auth_key_reply(
                    conn_handle,
                    raw::BLE_GAP_AUTH_KEY_TYPE_PASSKEY as u8,
                    passkey.as_ptr(),
                ))
            }
        } else {
            Err(RawError::InvalidState)
        }
    }
}

#[cfg(feature = "ble-bond")]
pub struct OutOfBandReply {
    conn: Option<Connection>,
}

#[cfg(feature = "ble-bond")]
impl Drop for OutOfBandReply {
    fn drop(&mut self) {
        if let Some(conn_handle) = self.conn.take().and_then(|x| x.handle()) {
            let ret = unsafe {
                raw::sd_ble_gap_auth_key_reply(conn_handle, raw::BLE_GAP_AUTH_KEY_TYPE_NONE as u8, core::ptr::null())
            };

            if let Err(_err) = RawError::convert(ret) {
                warn!("sd_ble_gap_auth_key_reply err {:?}", _err);
            }
        }
    }
}

#[cfg(feature = "ble-bond")]
impl OutOfBandReply {
    pub(crate) fn new(conn: Connection) -> Self {
        Self { conn: Some(conn) }
    }

    pub fn reply(mut self, oob: &[u8; 16]) -> Result<(), RawError> {
        if let Some(conn_handle) = self.conn.take().and_then(|x| x.handle()) {
            unsafe {
                RawError::convert(raw::sd_ble_gap_auth_key_reply(
                    conn_handle,
                    raw::BLE_GAP_AUTH_KEY_TYPE_OOB as u8,
                    oob.as_ptr(),
                ))
            }
        } else {
            Err(RawError::InvalidState)
        }
    }
}

pub struct SysAttrsReply {
    conn: Option<Connection>,
}

impl Drop for SysAttrsReply {
    fn drop(&mut self) {
        if let Some(conn_handle) = self.conn.take().and_then(|x| x.handle()) {
            let ret = unsafe { raw::sd_ble_gatts_sys_attr_set(conn_handle, core::ptr::null(), 0, 0) };

            if let Err(_err) = RawError::convert(ret) {
                warn!("sd_ble_gatts_sys_attr_set err {:?}", _err);
            }
        }
    }
}

impl SysAttrsReply {
    pub fn new(conn: Connection) -> Self {
        Self { conn: Some(conn) }
    }

    pub fn connection(&self) -> &Connection {
        unwrap!(self.conn.as_ref())
    }

    pub fn set_sys_attrs(mut self, sys_attrs: &[u8]) -> Result<(), RawError> {
        let conn_handle = unwrap!(self.conn.take().and_then(|x| x.handle()));

        unsafe {
            RawError::convert(raw::sd_ble_gatts_sys_attr_set(
                conn_handle,
                sys_attrs.as_ptr(),
                unwrap!(sys_attrs.len().try_into()),
                0,
            ))
        }
    }
}
