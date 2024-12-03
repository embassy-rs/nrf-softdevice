use crate::ble::gap::default_security_params;
use crate::ble::replies::{OutOfBandReply, PasskeyReply};
use crate::ble::types::{EncryptionInfo, IdentityKey, MasterId, SecurityMode};
use crate::ble::Connection;
use crate::raw;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IoCapabilities {
    None,
    DisplayYesNo,
    DisplayOnly,
    KeyboardOnly,
    KeyboardDisplay,
}

impl IoCapabilities {
    pub fn to_raw(self) -> u8 {
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

    /// Return `true` to request man-in-the-middle protection
    fn request_mitm_protection(&self, _conn: &Connection) -> bool {
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
    ///
    /// This is used for connections in the peripheral role.
    fn get_key(&self, _conn: &Connection, _master_id: MasterId) -> Option<EncryptionInfo> {
        None
    }

    #[cfg(feature = "ble-central")]
    /// Search the store for a known peer matching the connection address and return its `master_id` and LTK.
    ///
    /// This is used for connections in the central role. The keys should be found based on the peer address.
    /// If the peer address type is RandomPrivateResolvable it must be resolved using the stored IdentityKey.
    fn get_peripheral_key(&self, _conn: &Connection) -> Option<(MasterId, EncryptionInfo)> {
        None
    }

    #[cfg(feature = "ble-gatt-server")]
    /// Store the GATTS system attributes for `conn` if a bond exists
    fn save_sys_attrs(&self, _conn: &Connection) {}

    #[cfg(feature = "ble-gatt-server")]
    /// Load the GATTS system attributes for the bond associated with `conn`
    ///
    /// If no system attributes have been stored for this peer, you should call
    /// [set_sys_attrs][super::gatt_server::set_sys_attrs] with a `sys_attrs` parameter of `None`.
    fn load_sys_attrs(&self, conn: &Connection) {
        if let Err(err) = super::gatt_server::set_sys_attrs(conn, None) {
            warn!("SecurityHandler failed to set sys attrs: {:?}", err);
        }
    }

    /// The raw security parameters to use for authentication.
    fn security_params(&self, conn: &Connection) -> raw::ble_gap_sec_params_t {
        let mut sec_params = default_security_params();

        sec_params.set_oob(self.can_recv_out_of_band(conn) as u8);
        sec_params.set_io_caps(self.io_capabilities().to_raw());
        sec_params.set_mitm(self.request_mitm_protection(conn) as u8);

        if self.can_bond(conn) {
            sec_params.set_bond(1);
            sec_params.kdist_own.set_enc(1);
            sec_params.kdist_own.set_id(1);
            sec_params.kdist_peer.set_enc(1);
            sec_params.kdist_peer.set_id(1);
        }

        sec_params
    }
}
