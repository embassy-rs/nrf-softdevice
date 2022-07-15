use crate::ble::replies::{OutOfBandReply, PasskeyReply};
use crate::ble::types::SecurityMode;
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

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NotSupported {}

pub trait BondHandler {
    /// The connection has been bonded and its encryption keys should now be stored.
    fn on_bonded(
        &self,
        conn: &Connection,
        key: &raw::ble_gap_enc_key_t,
        peer_id: Option<&raw::ble_gap_id_key_t>,
        peer_key: Option<&raw::ble_gap_enc_key_t>,
    );

    /// Search the store for a known peer identified by `master_id` and return its LTK
    fn get_key(&self, conn: &Connection, master_id: raw::ble_gap_master_id_t) -> Option<raw::ble_gap_enc_info_t>;

    #[cfg(feature = "ble-gatt-server")]
    /// Store the GATTS system attributes for `conn` if a bond exists
    fn save_sys_attrs(&self, conn: &super::Connection);

    #[cfg(feature = "ble-gatt-server")]
    /// Load the GATTS system attributes for the bond associated with `conn`
    fn load_sys_attrs(&self, setter: super::replies::SysAttrsReply);

    fn on_security_update(&self, _conn: &Connection, _security_mode: SecurityMode) {}

    fn io_capabilities(&self) -> IoCapabilities {
        IoCapabilities::None
    }

    /// Display `passkey` to the user for confirmation on the remote device.
    ///
    /// Must be supported if [`io_capabilities()`] is one of `DisplayOnly` or `KeyboardDisplay`.
    fn display_passkey(&self, _passkey: &[u8; 6]) -> Result<(), NotSupported> {
        Err(NotSupported {})
    }

    /// Allow the user to enter a passkey displayed on the remote device.
    ///
    /// Must be supported if [`io_capabilities()`] is one of `KeyboardOnly` or `KeyboardDisplay`.
    fn enter_passkey(&self, _reply: PasskeyReply) -> Result<(), NotSupported> {
        Err(NotSupported {})
    }

    /// Receive out-of-band authentication data.
    fn recv_out_of_band(&self, _reply: OutOfBandReply) -> Result<(), NotSupported> {
        Err(NotSupported {})
    }
}
