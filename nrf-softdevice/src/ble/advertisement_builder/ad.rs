#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub struct AdvertisementDataType(u8);

impl AdvertisementDataType {
    pub const FLAGS: Self = Self(0x01);
    pub const INCOMPLETE_16_SERVICE_LIST: Self = Self(0x02);
    pub const COMPLETE_16_SERVICE_LIST: Self = Self(0x03);
    pub const INCOMPLETE_32_SERVICE_LIST: Self = Self(0x04);
    pub const COMPLETE_32_SERVICE_LIST: Self = Self(0x05);
    pub const INCOMPLETE_128_SERVICE_LIST: Self = Self(0x06);
    pub const COMPLETE_128_SERVICE_LIST: Self = Self(0x07);
    pub const SHORT_NAME: Self = Self(0x08);
    pub const FULL_NAME: Self = Self(0x09);
    pub const TXPOWER_LEVEL: Self = Self(0x0a);
    pub const PERIPHERAL_CONNECTION_INTERVAL_RANGE: Self = Self(0x12);
    pub const SERVICE_SOLICITATION_16: Self = Self(0x14);
    pub const SERVICE_SOLICITATION_128: Self = Self(0x15);
    pub const SERVICE_SOLICITATION_32: Self = Self(0x1f);
    pub const SERVICE_DATA_16: Self = Self(0x16);
    pub const SERVICE_DATA_32: Self = Self(0x20);
    pub const SERVICE_DATA_128: Self = Self(0x21);
    pub const APPEARANCE: Self = Self(0x19);
    pub const PUBLIC_TARGET_ADDRESS: Self = Self(0x17);
    pub const RANDOM_TARGET_ADDRESS: Self = Self(0x18);
    pub const ADVERTISING_INTERVAL: Self = Self(0x1a);
    pub const URI: Self = Self(0x24);
    pub const LE_SUPPORTED_FEATURES: Self = Self(0x27);
    pub const MANUFACTURER_SPECIFIC_DATA: Self = Self(0xff);

    pub const fn from_u8(value: u8) -> Self {
        Self(value)
    }

    pub const fn to_u8(self) -> u8 {
        self.0
    }
}

impl From<u8> for AdvertisementDataType {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<AdvertisementDataType> for u8 {
    fn from(value: AdvertisementDataType) -> Self {
        value.0
    }
}
