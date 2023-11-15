use crate::ble::replies::{OutOfBandReply, PasskeyReply};
use crate::ble::types::{EncryptionInfo, IdentityKey, MasterId, SecurityMode};
use crate::ble::Connection;
use crate::raw;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum IoCapabilities {
    None,
    DisplayYesNo,
    DisplayOnly,
    KeyboardOnly,
    KeyboardDisplay,
}

impl IoCapabilities {
    pub(crate) fn to_io_caps(self) -> u8 {
        unwrap!(match self {
            IoCapabilities::None => raw::BLE_GAP_IO_CAPS_NONE,
            IoCapabilities::DisplayYesNo => raw::BLE_GAP_IO_CAPS_DISPLAY_YESNO,
            IoCapabilities::DisplayOnly => raw::BLE_GAP_IO_CAPS_DISPLAY_ONLY,
            IoCapabilities::KeyboardOnly => raw::BLE_GAP_IO_CAPS_KEYBOARD_ONLY,
            IoCapabilities::KeyboardDisplay => raw::BLE_GAP_IO_CAPS_KEYBOARD_DISPLAY,
        }
        .try_into())
    }
}

pub trait SecurityHandler {
    fn io_capabilities(&self) -> IoCapabilities {
        IoCapabilities::None
    }

    /// Returns `true` if the device can receive out-of-band authentication data.
    fn can_recv_out_of_band(&self, _conn: &Connection) -> bool {
        false
    }

    /// Returns `true` if the device can save bonding keys for `_conn`
    fn can_bond(&self, _conn: &Connection) -> bool {
        false
    }

    /// Display `passkey` to the user for confirmation on the remote device.
    ///
    /// Must be implemented if [`io_capabilities()`][Self::io_capabilities] is one of `DisplayOnly`, `DisplayYesNo`, or `KeyboardDisplay`.
    fn display_passkey(&self, _passkey: &[u8; 6]) {
        panic!("SecurityHandler::display_passkey is not implemented");
    }

    /// Allow the user to enter a passkey displayed on the remote device.
    ///
    /// Must be implemented if [`io_capabilities()`][Self::io_capabilities] is one of `KeyboardOnly` or `KeyboardDisplay`.
    fn enter_passkey(&self, _reply: PasskeyReply) {
        panic!("SecurityHandler::enter_passkey is not implemented");
    }

    /// Receive out-of-band authentication data.
    ///
    /// Must be implemented if [`can_recv_out_of_band()`][Self::can_recv_out_of_band] ever returns `true`.
    fn recv_out_of_band(&self, _reply: OutOfBandReply) {
        panic!("SecurityHandler::recv_out_of_band is not implemented");
    }

    /// Called when the [`SecurityMode`] of a [`Connection`] has changed.
    fn on_security_update(&self, _conn: &Connection, _security_mode: SecurityMode) {}

    /// The connection has been bonded and its encryption keys should now be stored.
    ///
    /// Must be implemented if [`can_bond`][Self::can_bond] ever returns `true`.
    fn on_bonded(&self, _conn: &Connection, _master_id: MasterId, _key: EncryptionInfo, _peer_id: IdentityKey) {
        panic!("SecurityHandler::on_bonded not implemented")
    }

    /// Search the store for a known peer identified by `master_id` and return its LTK.
    fn get_key(&self, _conn: &Connection, _master_id: MasterId) -> Option<EncryptionInfo> {
        None
    }

    #[cfg(feature = "ble-gatt-server")]
    /// Store the GATTS system attributes for `conn` if a bond exists
    fn save_sys_attrs(&self, _conn: &super::Connection) {}

    #[cfg(feature = "ble-gatt-server")]
    /// Load the GATTS system attributes for the bond associated with `conn`
    ///
    /// If no system attributes have been stored for this peer, you should call
    /// [set_sys_attrs][super::gatt_server::set_sys_attrs] with a `sys_attrs` parameter of `None`.
    fn load_sys_attrs(&self, conn: &super::Connection) {
        if let Err(err) = super::gatt_server::set_sys_attrs(conn, None) {
            warn!("SecurityHandler failed to set sys attrs: {:?}", err);
        }
    }
}
