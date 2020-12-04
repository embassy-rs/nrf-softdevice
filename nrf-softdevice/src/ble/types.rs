use crate::raw;
use crate::RawError;

use crate::util::{panic, *};

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

#[derive(defmt::Format, Copy, Clone, Eq, PartialEq)]
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
            _ => panic!("unknown role {:u8}", raw),
        }
    }
}

#[repr(transparent)]
pub struct Address {
    pub(crate) inner: raw::ble_gap_addr_t,
}

impl Address {
    pub fn new_public(address: [u8; 6]) -> Self {
        Self {
            inner: raw::ble_gap_addr_t {
                addr: address,
                _bitfield_1: raw::ble_gap_addr_t::new_bitfield_1(
                    0,
                    raw::BLE_GAP_ADDR_TYPE_PUBLIC as u8,
                ),
            },
        }
    }
    pub fn new_random_static(address: [u8; 6]) -> Self {
        Self {
            inner: raw::ble_gap_addr_t {
                addr: address,
                _bitfield_1: raw::ble_gap_addr_t::new_bitfield_1(
                    0,
                    raw::BLE_GAP_ADDR_TYPE_RANDOM_STATIC as u8,
                ),
            },
        }
    }
}

impl defmt::Format for Address {
    fn format(&self, fmt: &mut defmt::Formatter) {
        defmt::write!(fmt, "{:[u8;6]}", self.inner.addr)
    }
}

#[derive(defmt::Format, Eq, PartialEq, Copy, Clone)]
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

#[derive(defmt::Format, Eq, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum Phy {
    _1M = 1,
    _2M = 2,
    Coded = 4,
}
