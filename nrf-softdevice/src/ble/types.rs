use core::mem;
use core::num::NonZeroU16;

use crate::{raw, RawError};

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Uuid {
    inner: raw::ble_uuid_t,
}

impl Uuid {
    pub const fn from_raw(raw: raw::ble_uuid_t) -> Option<Self> {
        if raw.type_ == raw::BLE_UUID_TYPE_UNKNOWN as u8 {
            None
        } else {
            Some(Self { inner: raw })
        }
    }

    pub const fn new_16(uuid: u16) -> Self {
        Self {
            inner: raw::ble_uuid_t {
                type_: raw::BLE_UUID_TYPE_BLE as u8,
                uuid,
            },
        }
    }

    // Create a new 128-bit UUID.
    //
    // Note that `uuid` needs to be in little-endian format, i.e. opposite to what you would
    // normally write UUIDs.
    pub fn new_128(uuid: &[u8; 16]) -> Self {
        let mut uuid_type: u8 = 0;
        let ret = unsafe { raw::sd_ble_uuid_vs_add(uuid.as_ptr() as _, &mut uuid_type as _) };
        match RawError::convert(ret) {
            Ok(()) => {}
            Err(e) => panic!("sd_ble_uuid_vs_add err {:?}", e),
        }

        Self {
            inner: raw::ble_uuid_t {
                type_: uuid_type,
                uuid: ((uuid[13] as u16) << 8) | (uuid[12] as u16),
            },
        }
    }

    pub fn as_raw_ptr(&self) -> *const raw::ble_uuid_t {
        &self.inner as _
    }

    pub fn into_raw(self) -> raw::ble_uuid_t {
        self.inner
    }
}

impl Eq for Uuid {}
impl PartialEq for Uuid {
    fn eq(&self, other: &Uuid) -> bool {
        self.inner.type_ == other.inner.type_ && self.inner.uuid == other.inner.uuid
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Role {
    #[cfg(feature = "ble-central")]
    Central,
    #[cfg(feature = "ble-peripheral")]
    Peripheral,
}

impl Role {
    pub fn from_raw(raw: u8) -> Self {
        match raw as u32 {
            #[cfg(feature = "ble-central")]
            raw::BLE_GAP_ROLE_CENTRAL => Self::Central,
            #[cfg(feature = "ble-peripheral")]
            raw::BLE_GAP_ROLE_PERIPH => Self::Peripheral,
            _ => panic!("unknown role {:?}", raw),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SecurityMode {
    NoAccess,
    Open,
    JustWorks,
    Mitm,
    LescMitm,
    Signed,
    SignedMitm,
}

impl Default for SecurityMode {
    fn default() -> Self {
        Self::Open
    }
}

impl SecurityMode {
    pub fn try_from_raw(raw: raw::ble_gap_conn_sec_mode_t) -> Option<Self> {
        match (raw.sm(), raw.lv()) {
            (0, 0) => Some(SecurityMode::NoAccess),
            (1, 1) => Some(SecurityMode::Open),
            (1, 2) => Some(SecurityMode::JustWorks),
            (1, 3) => Some(SecurityMode::Mitm),
            (1, 4) => Some(SecurityMode::LescMitm),
            (2, 1) => Some(SecurityMode::Signed),
            (2, 2) => Some(SecurityMode::SignedMitm),
            _ => None,
        }
    }

    pub fn into_raw(self) -> raw::ble_gap_conn_sec_mode_t {
        let (sm, lv) = match self {
            SecurityMode::NoAccess => (0, 0),
            SecurityMode::Open => (1, 1),
            SecurityMode::JustWorks => (1, 2),
            SecurityMode::Mitm => (1, 3),
            SecurityMode::LescMitm => (1, 4),
            SecurityMode::Signed => (2, 1),
            SecurityMode::SignedMitm => (2, 2),
        };

        raw::ble_gap_conn_sec_mode_t {
            _bitfield_1: raw::ble_gap_conn_sec_mode_t::new_bitfield_1(sm, lv),
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddressType {
    /// Public (identity) address
    Public = 0x00,
    /// Random static (identity) address.
    RandomStatic = 0x01,
    /// Random private resolvable address.
    RandomPrivateResolvable = 0x02,
    /// Random private non-resolvable address.
    RandomPrivateNonResolvable = 0x03,
    /// An advertiser may advertise without its address. This type of advertising is called anonymous.
    Anonymous = 0x7F,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct InvalidAddressType;

impl TryFrom<u8> for AddressType {
    type Error = InvalidAddressType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0x00 {
            Ok(AddressType::Public)
        } else if value == 0x01 {
            Ok(AddressType::RandomStatic)
        } else if value == 0x02 {
            Ok(AddressType::RandomPrivateResolvable)
        } else if value == 0x03 {
            Ok(AddressType::RandomPrivateNonResolvable)
        } else if value == 0x7F {
            Ok(AddressType::Anonymous)
        } else {
            Err(InvalidAddressType)
        }
    }
}

// Note: this type MUST be layout-compatible with raw::ble_gap_addr_t
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Address {
    // bit 0: is resolved private address
    // bits 7-1: type
    pub flags: u8,
    pub bytes: [u8; 6],
}

impl PartialEq for Address {
    fn eq(&self, other: &Self) -> bool {
        // bit 0 of flags indicates a peer identity address resolved from a resolvable private address by the
        // Softdevice. It is irrelevant for comparing addresses.
        (self.flags & 0xfe) == (other.flags & 0xfe) && self.bytes == other.bytes
    }
}

impl Eq for Address {}

impl Address {
    pub const fn new(address_type: AddressType, bytes: [u8; 6]) -> Self {
        Self {
            flags: (address_type as u8) << 1,
            bytes,
        }
    }

    pub fn address_type(&self) -> AddressType {
        unwrap!((self.flags >> 1).try_into())
    }

    pub fn is_resolved_peer_id(&self) -> bool {
        (self.flags & 1) != 0
    }

    pub fn bytes(&self) -> [u8; 6] {
        self.bytes
    }

    pub fn as_raw(&self) -> &raw::ble_gap_addr_t {
        // Safety: `Self` has the same layout as `raw::ble_gap_addr_t` and all bit patterns are valid
        unsafe { mem::transmute(self) }
    }

    pub fn from_raw(raw: raw::ble_gap_addr_t) -> Self {
        // Safety: `Self` has the same layout as `raw::ble_gap_addr_t` and all bit patterns are valid
        unsafe { mem::transmute(raw) }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Address {
    fn format(&self, fmt: defmt::Formatter) {
        if self.is_resolved_peer_id() {
            defmt::write!(fmt, "{:?}(resolved):{=[u8]:x}", self.address_type(), self.bytes())
        } else {
            defmt::write!(fmt, "{:?}:{=[u8]:x}", self.address_type(), self.bytes())
        }
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Eq, PartialEq, Copy, Clone)]
#[repr(i8)]
pub enum TxPower {
    Minus40dBm = -40,
    Minus20dBm = -20,
    Minus16dBm = -16,
    Minus12dBm = -12,
    Minus8dBm = -8,
    Minus4dBm = -4,
    ZerodBm = 0,
    #[cfg(feature = "s140")]
    Plus2dBm = 2,
    Plus3dBm = 3,
    Plus4dBm = 4,
    #[cfg(feature = "s140")]
    Plus5dBm = 5,
    #[cfg(feature = "s140")]
    Plus6dBm = 6,
    #[cfg(feature = "s140")]
    Plus7dBm = 7,
    #[cfg(feature = "s140")]
    Plus8dBm = 8,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Phy {
    /// 1Mbps phy
    M1 = 1,
    /// 2Mbps phy
    M2 = 2,
    /// Coded phy (125kbps, S=8)
    #[cfg(feature = "s140")]
    Coded = 4,
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum PhySet {
    /// 1Mbps phy
    M1 = 1,
    /// 2Mbps phy
    M2 = 2,
    /// 1Mbps + 2Mbps phys
    M1M2 = 3,
    /// Coded phy (125kbps, S=8)
    #[cfg(feature = "s140")]
    Coded = 4,
    /// 1Mbps and Coded phys
    #[cfg(feature = "s140")]
    M1Coded = 5,
    /// 2Mbps and Coded phys
    #[cfg(feature = "s140")]
    M2Coded = 6,
    /// 1Mbps, 2Mbps and Coded phys
    #[cfg(feature = "s140")]
    M1M2Coded = 7,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MasterId {
    /// Encrypted diversifier
    pub ediv: u16,
    /// Random number
    pub rand: [u8; 8],
}

impl MasterId {
    pub fn as_raw(&self) -> &raw::ble_gap_master_id_t {
        // Safety: `Self` has the same layout as `raw::ble_gap_master_id_t` and no uninitialized (padding) bytes
        unsafe { mem::transmute(self) }
    }

    pub fn from_raw(raw: raw::ble_gap_master_id_t) -> Self {
        MasterId {
            ediv: raw.ediv,
            rand: raw.rand,
        }
    }
}

// Note: this type MUST be layout-compatible with raw::ble_gap_enc_info_t
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct EncryptionInfo {
    /// Long term key
    pub ltk: [u8; 16],
    pub flags: u8,
}

impl EncryptionInfo {
    pub fn as_raw(&self) -> &raw::ble_gap_enc_info_t {
        // Safety: `Self` has the same layout as `raw::ble_gap_enc_info_t` and all bit patterns are valid
        unsafe { mem::transmute(self) }
    }

    pub fn from_raw(raw: raw::ble_gap_enc_info_t) -> Self {
        // Safety: `raw::ble_gap_enc_info_t` has the same layout as `Self` and all bit patterns are valid
        unsafe { mem::transmute(raw) }
    }
}

// Note: this type MUST be layout-compatible with raw::ble_gap_irk_t
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IdentityResolutionKey {
    irk: [u8; 16],
}

impl IdentityResolutionKey {
    pub fn from_raw(raw: raw::ble_gap_irk_t) -> Self {
        Self { irk: raw.irk }
    }

    pub fn as_raw(&self) -> &raw::ble_gap_irk_t {
        // Safety: `Self` has the same layout as `raw::ble_gap_irk_t` and all bit patterns are valid
        unsafe { core::mem::transmute(self) }
    }
}

// Note: this type MUST be layout-compatible with raw::ble_gap_id_key_t
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IdentityKey {
    /// Identity resolution key
    pub irk: IdentityResolutionKey,
    /// Address
    pub addr: Address,
}

impl IdentityKey {
    pub fn is_match(&self, addr: Address) -> bool {
        match addr.address_type() {
            AddressType::Public | AddressType::RandomStatic => self.addr == addr,
            AddressType::RandomPrivateResolvable => {
                let local_hash = random_address_hash(self.irk, addr.bytes()[3..].try_into().unwrap());
                addr.bytes()[..3] == local_hash
            }
            AddressType::RandomPrivateNonResolvable | AddressType::Anonymous => false,
        }
    }

    pub fn from_raw(raw: raw::ble_gap_id_key_t) -> Self {
        Self {
            irk: IdentityResolutionKey::from_raw(raw.id_info),
            addr: Address::from_raw(raw.id_addr_info),
        }
    }

    pub fn from_addr(addr: Address) -> Self {
        Self {
            irk: Default::default(),
            addr,
        }
    }

    pub fn as_raw(&self) -> &raw::ble_gap_id_key_t {
        // Safety: `Self` has the same layout as `raw::ble_gap_id_key_t` and all bit patterns are valid
        unsafe { core::mem::transmute(self) }
    }
}

fn random_address_hash(key: IdentityResolutionKey, r: [u8; 3]) -> [u8; 3] {
    let mut cleartext = [0; 16];
    cleartext[13..].copy_from_slice(&r);
    cleartext[13..].reverse(); // big-endian to little-endian

    let mut ecb_hal_data: raw::nrf_ecb_hal_data_t = raw::nrf_ecb_hal_data_t {
        key: key.irk,
        cleartext,
        ciphertext: [0; 16],
    };

    ecb_hal_data.key.reverse(); // big-endian to little-endian

    // Can only return NRF_SUCCESS
    let _ = unsafe { raw::sd_ecb_block_encrypt(&mut ecb_hal_data) };

    let mut res: [u8; 3] = ecb_hal_data.ciphertext[13..].try_into().unwrap();
    res.reverse(); // little-endian to big-endian
    res
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct GattError(NonZeroU16);

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct GattStatus(u16);

impl GattError {
    pub const fn new(err: u16) -> Option<Self> {
        match NonZeroU16::new(err) {
            Some(n) => Some(Self(n)),
            None => None,
        }
    }

    /// Construct an arbitrary ATT protocol error code
    pub const fn from_att_error(err: u8) -> Self {
        Self(unsafe { NonZeroU16::new_unchecked(0x100 + err as u16) })
    }

    pub const fn to_status(self) -> GattStatus {
        GattStatus(self.0.get())
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for GattError {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::Format::format(&self.to_status(), fmt)
    }
}

impl core::fmt::Debug for GattError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.to_status(), fmt)
    }
}

impl From<GattError> for u16 {
    fn from(value: GattError) -> Self {
        value.0.get()
    }
}

impl GattStatus {
    pub const SUCCESS: GattStatus = GattStatus(raw::BLE_GATT_STATUS_SUCCESS as u16);

    pub const fn new(status: u16) -> Self {
        Self(status)
    }

    pub const fn is_app_error(self) -> bool {
        self.0 >= raw::BLE_GATT_STATUS_ATTERR_APP_BEGIN as u16 && self.0 <= raw::BLE_GATT_STATUS_ATTERR_APP_END as u16
    }

    pub const fn to_result(self) -> Result<(), GattError> {
        match NonZeroU16::new(self.0) {
            None => Ok(()),
            Some(err) => Err(GattError(err)),
        }
    }
}

impl From<GattError> for GattStatus {
    fn from(value: GattError) -> Self {
        value.to_status()
    }
}

impl From<u16> for GattStatus {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<GattStatus> for u16 {
    fn from(value: GattStatus) -> Self {
        value.0
    }
}

macro_rules! error_codes {
    (
        $(
            $(#[$docs:meta])*
            ($konst:ident, $raw:ident, $phrase:expr);
        )+
    ) => {
        impl GattError {
        $(
            $(#[$docs])*
            pub const $konst: GattError = GattError(unsafe { NonZeroU16::new_unchecked(raw::$raw as u16) });
        )+
        }

        #[cfg(feature = "defmt")]
        impl defmt::Format for GattStatus {
            fn format(&self, fmt: defmt::Formatter) {
                if self.is_app_error() {
                    defmt::write!(fmt, "Application Error: 0x{:02x}", self.0 as u8);
                } else {
                    match *self {
                        Self::SUCCESS => defmt::write!(fmt, "Success"),
                        $(
                        Self::$konst => defmt::write!(fmt, $phrase),
                        )+
                        _ => defmt::write!(fmt, "Unknown GATT status: 0x{:04x}", self.0),
                    }
                }
            }
        }

        impl core::fmt::Debug for GattStatus {
            fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                if self.is_app_error() {
                    core::write!(fmt, "Application Error: 0x{:02x}", self.0 as u8)
                } else {
                    match *self {
                        Self::SUCCESS => core::write!(fmt, "Success"),
                        $(
                        Self::$konst => core::write!(fmt, $phrase),
                        )+
                        _ => core::write!(fmt, "Unknown GATT status: 0x{:04x}", self.0),
                    }
                }
            }
        }


        impl GattStatus {
        $(
            $(#[$docs])*
            pub const $konst: GattStatus = GattError::$konst.to_status();
        )+
        }
    }
}

error_codes! {
    /// Unknown or not applicable status.
    (UNKNOWN, BLE_GATT_STATUS_UNKNOWN, "Unknown");
    /// ATT Error: Invalid Error Code.
    (ATTERR_INVALID, BLE_GATT_STATUS_ATTERR_INVALID, "Invalid Error Code");
    /// ATT Error: Invalid Attribute Handle.
    (ATTERR_INVALID_HANDLE, BLE_GATT_STATUS_ATTERR_INVALID_HANDLE, "Invalid Handle");
    /// ATT Error: Read not permitted.
    (ATTERR_READ_NOT_PERMITTED, BLE_GATT_STATUS_ATTERR_READ_NOT_PERMITTED, "Read Not Permitted");
    /// ATT Error: Write not permitted.
    (ATTERR_WRITE_NOT_PERMITTED, BLE_GATT_STATUS_ATTERR_WRITE_NOT_PERMITTED, "Write Not Permitted");
    /// ATT Error: Used in ATT as Invalid PDU.
    (ATTERR_INVALID_PDU, BLE_GATT_STATUS_ATTERR_INVALID_PDU, "Invalid PDU");
    /// ATT Error: Authenticated link required.
    (ATTERR_INSUF_AUTHENTICATION, BLE_GATT_STATUS_ATTERR_INSUF_AUTHENTICATION, "Insufficient Authentication");
    /// ATT Error: Used in ATT as Request Not Supported.
    (ATTERR_REQUEST_NOT_SUPPORTED, BLE_GATT_STATUS_ATTERR_REQUEST_NOT_SUPPORTED, "Request Not Supported");
    /// ATT Error: Offset specified was past the end of the attribute.
    (ATTERR_INVALID_OFFSET, BLE_GATT_STATUS_ATTERR_INVALID_OFFSET, "Invalid Offset");
    /// ATT Error: Used in ATT as Insufficient Authorization.
    (ATTERR_INSUF_AUTHORIZATION, BLE_GATT_STATUS_ATTERR_INSUF_AUTHORIZATION, "Insufficient Authorization");
    /// ATT Error: Used in ATT as Prepare Queue Full.
    (ATTERR_PREPARE_QUEUE_FULL, BLE_GATT_STATUS_ATTERR_PREPARE_QUEUE_FULL, "Prepare Queue Full");
    /// ATT Error: Used in ATT as Attribute not found.
    (ATTERR_ATTRIBUTE_NOT_FOUND, BLE_GATT_STATUS_ATTERR_ATTRIBUTE_NOT_FOUND, "Attribute Not Found");
    /// ATT Error: Attribute cannot be read or written using read/write blob requests.
    (ATTERR_ATTRIBUTE_NOT_LONG, BLE_GATT_STATUS_ATTERR_ATTRIBUTE_NOT_LONG, "Attribute Not Long");
    /// ATT Error: Encryption key size used is insufficient.
    (ATTERR_INSUF_ENC_KEY_SIZE, BLE_GATT_STATUS_ATTERR_INSUF_ENC_KEY_SIZE, "Insufficient Encryption Key Size");
    /// ATT Error: Invalid value size.
    (ATTERR_INVALID_ATT_VAL_LENGTH, BLE_GATT_STATUS_ATTERR_INVALID_ATT_VAL_LENGTH, "Invalid Attribute Value Size");
    /// ATT Error: Very unlikely error.
    (ATTERR_UNLIKELY_ERROR, BLE_GATT_STATUS_ATTERR_UNLIKELY_ERROR, "Unlikely Error");
    /// ATT Error: Encrypted link required.
    (ATTERR_INSUF_ENCRYPTION, BLE_GATT_STATUS_ATTERR_INSUF_ENCRYPTION, "Insufficient Encryption");
    /// ATT Error: Attribute type is not a supported grouping attribute.
    (ATTERR_UNSUPPORTED_GROUP_TYPE, BLE_GATT_STATUS_ATTERR_UNSUPPORTED_GROUP_TYPE, "Unsupported Group Type");
    /// ATT Error: Insufficient resources.
    (ATTERR_INSUF_RESOURCES, BLE_GATT_STATUS_ATTERR_INSUF_RESOURCES, "Insufficient Resources");
    /// ATT Common Profile and Service Error: Write request rejected.
    (ATTERR_CPS_WRITE_REQ_REJECTED, BLE_GATT_STATUS_ATTERR_CPS_WRITE_REQ_REJECTED, "Write Request Rejected");
    /// ATT Common Profile and Service Error: Client Characteristic Configuration Descriptor improperly configured.
    (ATTERR_CPS_CCCD_CONFIG_ERROR, BLE_GATT_STATUS_ATTERR_CPS_CCCD_CONFIG_ERROR, "Client Characteristic Configration Descriptor Improperly Configured");
    /// ATT Common Profile and Service Error: Procedure Already in Progress.
    (ATTERR_CPS_PROC_ALR_IN_PROG, BLE_GATT_STATUS_ATTERR_CPS_PROC_ALR_IN_PROG, "Procedure Already in Progress");
    /// ATT Common Profile and Service Error: Out Of Range.
    (ATTERR_CPS_OUT_OF_RANGE, BLE_GATT_STATUS_ATTERR_CPS_OUT_OF_RANGE, "Out of Range");
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct HciStatus(u8);

impl HciStatus {
    pub const fn new(status: u8) -> Self {
        Self(status)
    }
}

impl From<u8> for HciStatus {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<HciStatus> for u8 {
    fn from(value: HciStatus) -> Self {
        value.0
    }
}

macro_rules! hci_status_codes {
    (
        $(
            $(#[$docs:meta])*
            ($konst:ident, $raw:expr, $phrase:expr);
        )+
    ) => {
        impl HciStatus {
        $(
            $(#[$docs])*
            pub const $konst: HciStatus = HciStatus($raw as u8);
        )+
        }

        #[cfg(feature = "defmt")]
        impl defmt::Format for HciStatus {
            fn format(&self, fmt: defmt::Formatter) {
                match *self {
                    $(
                    Self::$konst => defmt::write!(fmt, $phrase),
                    )+
                    _ => defmt::write!(fmt, "Unknown HCI status: 0x{:02x}", self.0),
                }
            }
        }

        impl core::fmt::Debug for HciStatus {
            fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match *self {
                    $(
                    Self::$konst => core::write!(fmt, $phrase),
                    )+
                    _ => core::write!(fmt, "Unknown HCI status: 0x{:02x}", self.0),
                }
            }
        }
    }
}

hci_status_codes! {
    /// Success
    (SUCCESS, raw::BLE_HCI_STATUS_CODE_SUCCESS, "Success");
    /// Unknown HCI Command
    (UNKNOWN_BTLE_COMMAND, raw::BLE_HCI_STATUS_CODE_UNKNOWN_BTLE_COMMAND, "Unknown HCI Command");
    /// Unknown Connection Identifier
    (UNKNOWN_CONNECTION_IDENTIFIER, raw::BLE_HCI_STATUS_CODE_UNKNOWN_CONNECTION_IDENTIFIER, "Unknown Connection Identifier");
    /// Authentication Failure
    (AUTHENTICATION_FAILURE, raw::BLE_HCI_AUTHENTICATION_FAILURE, "Authentication Failure");
    /// PIN Or Key Missing
    (PIN_OR_KEY_MISSING, raw::BLE_HCI_STATUS_CODE_PIN_OR_KEY_MISSING, "PIN Or Key Missing");
    /// Memory Capacity Exceeded
    (MEMORY_CAPACITY_EXCEEDED, raw::BLE_HCI_MEMORY_CAPACITY_EXCEEDED, "Memory Capacity Exceeded");
    /// Connection Timeout
    (CONNECTION_TIMEOUT, raw::BLE_HCI_CONNECTION_TIMEOUT, "Connection Timeout");
    /// Command Disallowed
    (COMMAND_DISALLOWED, raw::BLE_HCI_STATUS_CODE_COMMAND_DISALLOWED, "Command Disallowed");
    /// Invalid HCI Command Parameters
    (INVALID_BTLE_COMMAND_PARAMETERS, raw::BLE_HCI_STATUS_CODE_INVALID_BTLE_COMMAND_PARAMETERS, "Invalid HCI Command Parameters");
    /// Remote User Terminated Connection
    (REMOTE_USER_TERMINATED_CONNECTION, raw::BLE_HCI_REMOTE_USER_TERMINATED_CONNECTION, "Remote User Terminated Connection");
    /// Remote Device Terminated Connection due to Low Resources
    (REMOTE_DEV_TERMINATION_DUE_TO_LOW_RESOURCES, raw::BLE_HCI_REMOTE_DEV_TERMINATION_DUE_TO_LOW_RESOURCES, "Remote Device Terminated Connection due to Low Resources");
    /// Remote Device Terminated Connection due to Power Off
    (REMOTE_DEV_TERMINATION_DUE_TO_POWER_OFF, raw::BLE_HCI_REMOTE_DEV_TERMINATION_DUE_TO_POWER_OFF, "Remote Device Terminated Connection due to Power Off");
    /// Connection Terminated by Local Host
    (LOCAL_HOST_TERMINATED_CONNECTION, raw::BLE_HCI_LOCAL_HOST_TERMINATED_CONNECTION, "Connection Terminated by Local Host");
    /// Unsupported Remote Feature
    (UNSUPPORTED_REMOTE_FEATURE, raw::BLE_HCI_UNSUPPORTED_REMOTE_FEATURE, "Unsupported Remote Feature");
    /// Invalid LMP Parameters
    (INVALID_LMP_PARAMETERS, raw::BLE_HCI_STATUS_CODE_INVALID_LMP_PARAMETERS, "Invalid LMP Parameters");
    /// Unspecified Error
    (UNSPECIFIED_ERROR, raw::BLE_HCI_STATUS_CODE_UNSPECIFIED_ERROR, "Unspecified Error");
    /// LMP Response Timeout
    (LMP_RESPONSE_TIMEOUT, raw::BLE_HCI_STATUS_CODE_LMP_RESPONSE_TIMEOUT, "LMP Response Timeout");
    /// LMP Error Transaction Collision
    (LMP_ERROR_TRANSACTION_COLLISION, raw::BLE_HCI_STATUS_CODE_LMP_ERROR_TRANSACTION_COLLISION, "LMP Error Transaction Collision");
    /// LMP PDU Not Allowed
    (LMP_PDU_NOT_ALLOWED, raw::BLE_HCI_STATUS_CODE_LMP_PDU_NOT_ALLOWED, "LMP PDU Not Allowed");
    /// Instant Passed
    (INSTANT_PASSED, raw::BLE_HCI_INSTANT_PASSED, "Instant Passed");
    /// Pairing With Unit Key Not Supported
    (PAIRING_WITH_UNIT_KEY_UNSUPPORTED, raw::BLE_HCI_PAIRING_WITH_UNIT_KEY_UNSUPPORTED, "Pairing With Unit Key Not Supported");
    /// Different Transaction Collision
    (DIFFERENT_TRANSACTION_COLLISION, raw::BLE_HCI_DIFFERENT_TRANSACTION_COLLISION, "Different Transaction Collision");
    /// Parameter Out Of Mandatory Range
    (PARAMETER_OUT_OF_MANDATORY_RANGE, raw::BLE_HCI_PARAMETER_OUT_OF_MANDATORY_RANGE, "Parameter Out Of Mandatory Range");
    /// Controller Busy
    (CONTROLLER_BUSY, raw::BLE_HCI_CONTROLLER_BUSY, "Controller Busy");
    /// Unacceptable Connection Parameters
    (CONN_INTERVAL_UNACCEPTABLE, raw::BLE_HCI_CONN_INTERVAL_UNACCEPTABLE, "Unacceptable Connection Parameters");
    /// Advertising Timeout
    (DIRECTED_ADVERTISER_TIMEOUT, raw::BLE_HCI_DIRECTED_ADVERTISER_TIMEOUT, "Advertising Timeout");
    /// Connection Terminated due to MIC Failure
    (CONN_TERMINATED_DUE_TO_MIC_FAILURE, raw::BLE_HCI_CONN_TERMINATED_DUE_TO_MIC_FAILURE, "Connection Terminated due to MIC Failure");
    /// Connection Failed to be Established
    (CONN_FAILED_TO_BE_ESTABLISHED, raw::BLE_HCI_CONN_FAILED_TO_BE_ESTABLISHED, "Connection Failed to be Established");
}
