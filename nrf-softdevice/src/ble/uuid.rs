use crate::error::Error;
use crate::sd;
use crate::util::*;

#[repr(transparent)]
pub struct Uuid {
    inner: sd::ble_uuid_t,
}

impl Uuid {
    pub const fn new_16(uuid: u16) -> Self {
        Self {
            inner: sd::ble_uuid_t {
                type_: sd::BLE_UUID_TYPE_BLE as u8,
                uuid,
            },
        }
    }

    pub fn new_128(uuid: &[u8; 16]) -> Self {
        let mut uuid_type: u8 = 0;
        let ret = unsafe { sd::sd_ble_uuid_vs_add(uuid.as_ptr() as _, &mut uuid_type as _) };
        match Error::convert(ret) {
            Ok(()) => {}
            Err(e) => depanic!("sd_ble_uuid_vs_add err {:?}", e),
        }

        Self {
            inner: sd::ble_uuid_t {
                type_: uuid_type,
                uuid: ((uuid[13] as u16) << 8) | (uuid[12] as u16),
            },
        }
    }

    pub unsafe fn as_raw_ptr(&self) -> *const sd::ble_uuid_t {
        &self.inner as _
    }
}
