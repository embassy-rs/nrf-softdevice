use crate::ble::SecurityMode;
use crate::raw;

// Missing:
// - Characteristic presentation format

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AttributeMetadata {
    pub read: SecurityMode,
    pub write: SecurityMode,
    pub variable_len: bool,
    pub deferred_read: bool,
    pub deferred_write: bool,
}

impl AttributeMetadata {
    pub fn with_security(security: SecurityMode) -> Self {
        AttributeMetadata {
            read: security,
            write: security,
            ..Default::default()
        }
    }

    pub fn read_security(mut self, security: SecurityMode) -> Self {
        self.read = security;
        self
    }

    pub fn write_security(mut self, security: SecurityMode) -> Self {
        self.write = security;
        self
    }

    pub(crate) fn into_raw(self) -> raw::ble_gatts_attr_md_t {
        self.into_raw_inner(raw::BLE_GATTS_VLOC_STACK as u8)
    }

    #[cfg(feature = "alloc")]
    pub(crate) fn into_raw_user(self) -> raw::ble_gatts_attr_md_t {
        self.into_raw_inner(raw::BLE_GATTS_VLOC_USER as u8)
    }

    fn into_raw_inner(self, vloc: u8) -> raw::ble_gatts_attr_md_t {
        raw::ble_gatts_attr_md_t {
            read_perm: self.read.into_raw(),
            write_perm: self.write.into_raw(),
            _bitfield_1: raw::ble_gatts_attr_md_t::new_bitfield_1(
                self.variable_len.into(),
                vloc,
                self.deferred_read.into(),
                self.deferred_write.into(),
            ),
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Attribute<T: AsRef<[u8]>> {
    pub metadata: AttributeMetadata,
    pub value: T,
    pub max_len: u16,
}

impl<T: AsRef<[u8]>> Attribute<T> {
    pub fn new(value: T) -> Self {
        let max_len = unwrap!(value.as_ref().len().try_into());
        Attribute {
            max_len,
            value,
            metadata: Default::default(),
        }
    }

    pub fn security(mut self, security: SecurityMode) -> Self {
        self.metadata.read = security;
        self.metadata.write = security;
        self
    }

    pub fn read_security(mut self, security: SecurityMode) -> Self {
        self.metadata.read = security;
        self
    }

    pub fn write_security(mut self, security: SecurityMode) -> Self {
        self.metadata.write = security;
        self
    }

    pub fn variable_len(mut self, max_len: u16) -> Self {
        self.max_len = max_len;
        self.metadata.variable_len = true;
        self
    }

    pub fn deferred(mut self) -> Self {
        self.metadata.deferred_read = true;
        self.metadata.deferred_write = true;
        self
    }

    pub fn deferred_read(mut self) -> Self {
        self.metadata.deferred_read = true;
        self
    }

    pub fn deferred_write(mut self) -> Self {
        self.metadata.deferred_write = true;
        self
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UserDescription {
    pub metadata: Option<AttributeMetadata>,
    pub value: &'static [u8],
    pub max_len: u16,
}

#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Properties {
    pub broadcast: bool,
    pub read: bool,
    pub write_without_response: bool,
    pub write: bool,
    pub notify: bool,
    pub indicate: bool,
    pub signed_write: bool,
    pub queued_write: bool,
    pub write_user_description: bool,
}

impl Properties {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn broadcast(mut self) -> Self {
        self.broadcast = true;
        self
    }

    pub fn read(mut self) -> Self {
        self.read = true;
        self
    }

    pub fn write_without_response(mut self) -> Self {
        self.write_without_response = true;
        self
    }

    pub fn write(mut self) -> Self {
        self.write = true;
        self
    }

    pub fn notify(mut self) -> Self {
        self.notify = true;
        self
    }

    pub fn indicate(mut self) -> Self {
        self.indicate = true;
        self
    }

    pub fn signed_write(mut self) -> Self {
        self.signed_write = true;
        self
    }

    pub fn queued_write(mut self) -> Self {
        self.queued_write = true;
        self
    }

    pub fn write_user_description(mut self) -> Self {
        self.write_user_description = true;
        self
    }

    pub(crate) fn into_raw(self) -> (raw::ble_gatt_char_props_t, raw::ble_gatt_char_ext_props_t) {
        (
            raw::ble_gatt_char_props_t {
                _bitfield_1: raw::ble_gatt_char_props_t::new_bitfield_1(
                    self.broadcast.into(),
                    self.read.into(),
                    self.write_without_response.into(),
                    self.write.into(),
                    self.notify.into(),
                    self.indicate.into(),
                    self.signed_write.into(),
                ),
            },
            raw::ble_gatt_char_ext_props_t {
                _bitfield_1: raw::ble_gatt_char_ext_props_t::new_bitfield_1(
                    self.queued_write.into(),
                    self.write_user_description.into(),
                ),
            },
        )
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Metadata {
    pub properties: Properties,
    pub user_description: Option<UserDescription>,
    pub cccd: Option<AttributeMetadata>,
    pub sccd: Option<AttributeMetadata>,
}

impl Metadata {
    pub fn new(properties: Properties) -> Self {
        let cccd = if properties.indicate || properties.notify {
            Some(AttributeMetadata::default())
        } else {
            None
        };

        let sccd = if properties.broadcast {
            Some(AttributeMetadata::default())
        } else {
            None
        };

        Metadata {
            properties,
            cccd,
            sccd,
            ..Default::default()
        }
    }

    pub fn with_security(properties: Properties, write_security: SecurityMode) -> Self {
        let cccd = if properties.indicate || properties.notify {
            Some(AttributeMetadata::default().write_security(write_security))
        } else {
            None
        };

        let sccd = if properties.broadcast {
            Some(AttributeMetadata::default().write_security(write_security))
        } else {
            None
        };

        Metadata {
            properties,
            cccd,
            sccd,
            ..Default::default()
        }
    }
}
