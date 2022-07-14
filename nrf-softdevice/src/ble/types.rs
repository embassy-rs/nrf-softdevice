use core::mem;

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

    pub unsafe fn as_raw_ptr(&self) -> *const raw::ble_uuid_t {
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

// Note: this type MUST be layout-compatible with raw::ble_gap_addr_t
#[repr(C)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Address {
    // bit 0: is resolved private address
    // bits 7-1: type
    pub flags: u8,
    pub bytes: [u8; 6],
}

impl Address {
    pub const fn new(address_type: AddressType, bytes: [u8; 6]) -> Self {
        Self {
            flags: (address_type as u8) << 1,
            bytes,
        }
    }

    pub fn address_type(&self) -> AddressType {
        unsafe { mem::transmute(self.flags >> 1) }
    }

    pub fn bytes(&self) -> [u8; 6] {
        self.bytes
    }

    pub fn into_raw(&self) -> raw::ble_gap_addr_t {
        unsafe { mem::transmute(*self) }
    }

    pub unsafe fn from_raw(raw: raw::ble_gap_addr_t) -> Self {
        mem::transmute(raw)
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Address {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{:?}:{=[u8]:x}", self.address_type(), self.bytes())
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
    Plus2dBm = 2,
    Plus3dBm = 3,
    Plus4dBm = 4,
    Plus5dBm = 5,
    Plus6dBm = 6,
    Plus7dBm = 7,
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
