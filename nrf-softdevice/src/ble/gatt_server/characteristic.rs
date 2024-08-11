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
    pub const fn new() -> Self {
        Self {
            broadcast: false,
            read: false,
            write_without_response: false,
            write: false,
            notify: false,
            indicate: false,
            signed_write: false,
            queued_write: false,
            write_user_description: false,
        }
    }

    pub const fn broadcast(mut self) -> Self {
        self.broadcast = true;
        self
    }

    pub const fn read(mut self) -> Self {
        self.read = true;
        self
    }

    pub const fn write_without_response(mut self) -> Self {
        self.write_without_response = true;
        self
    }

    pub const fn write(mut self) -> Self {
        self.write = true;
        self
    }

    pub const fn notify(mut self) -> Self {
        self.notify = true;
        self
    }

    pub const fn indicate(mut self) -> Self {
        self.indicate = true;
        self
    }

    pub const fn signed_write(mut self) -> Self {
        self.signed_write = true;
        self
    }

    pub const fn queued_write(mut self) -> Self {
        self.queued_write = true;
        self
    }

    pub const fn write_user_description(mut self) -> Self {
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
pub struct Presentation {
    pub format: u8,
    pub exponent: i8,
    pub unit: u16,
    pub name_space: u8,
    pub description: u16,
}

impl Presentation {
    pub(crate) fn into_raw(self) -> raw::ble_gatts_char_pf_t {
        raw::ble_gatts_char_pf_t {
            format: self.format.into(),
            exponent: self.exponent.into(),
            unit: self.unit.into(),
            name_space: self.name_space.into(),
            desc: self.description.into(),
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Metadata {
    pub properties: Properties,
    pub user_description: Option<UserDescription>,
    pub cpfd: Option<Presentation>,
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

    #[deprecated = "Use new(properties).security(write_security) instead."]
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

    pub fn presentation(self, presentation: Presentation) -> Self {
        let cpfd = Some(presentation);
        Metadata { cpfd, ..self }
    }

    pub fn security(self, write_security: SecurityMode) -> Self {
        let cccd = self.cccd.map(|cccd| cccd.write_security(write_security));
        let sccd = self.sccd.map(|sccd| sccd.write_security(write_security));
        Metadata { cccd, sccd, ..self }
    }
}
