use crate::raw;
use crate::RawError;

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

    pub fn new_128(uuid: &[u8; 16]) -> Self {
        let mut uuid_type: u8 = 0;
        let ret = unsafe { raw::sd_ble_uuid_vs_add(uuid.as_ptr() as _, &mut uuid_type as _) };
        match RawError::convert(ret) {
            Ok(()) => {}
            Err(e) => depanic!("sd_ble_uuid_vs_add err {:?}", e),
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
            _ => depanic!("unknown role {:u8}", raw),
        }
    }
}

#[repr(transparent)]
pub struct Address {
    inner: raw::ble_gap_addr_t,
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
