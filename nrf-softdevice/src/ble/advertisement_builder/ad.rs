#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub struct AdvertisementDataType(u8);

impl AdvertisementDataType {
    pub const FLAGS: AdvertisementDataType = AdvertisementDataType(0x01);
    pub const INCOMPLETE_16_SERVICE_LIST: AdvertisementDataType = AdvertisementDataType(0x02);
    pub const COMPLETE_16_SERVICE_LIST: AdvertisementDataType = AdvertisementDataType(0x03);
    pub const INCOMPLETE_32_SERVICE_LIST: AdvertisementDataType = AdvertisementDataType(0x04);
    pub const COMPLETE_32_SERVICE_LIST: AdvertisementDataType = AdvertisementDataType(0x05);
    pub const INCOMPLETE_128_SERVICE_LIST: AdvertisementDataType = AdvertisementDataType(0x06);
    pub const COMPLETE_128_SERVICE_LIST: AdvertisementDataType = AdvertisementDataType(0x07);
    pub const SHORT_NAME: AdvertisementDataType = AdvertisementDataType(0x08);
    pub const FULL_NAME: AdvertisementDataType = AdvertisementDataType(0x09);
    pub const TXPOWER_LEVEL: AdvertisementDataType = AdvertisementDataType(0x0a);
    pub const PERIPHERAL_CONNECTION_INTERVAL_RANGE: AdvertisementDataType = AdvertisementDataType(0x12);
    pub const SERVICE_SOLICITATION_16: AdvertisementDataType = AdvertisementDataType(0x14);
    pub const SERVICE_SOLICITATION_128: AdvertisementDataType = AdvertisementDataType(0x15);
    pub const SERVICE_SOLICITATION_32: AdvertisementDataType = AdvertisementDataType(0x1f);
    pub const SERVICE_DATA_16: AdvertisementDataType = AdvertisementDataType(0x16);
    pub const SERVICE_DATA_32: AdvertisementDataType = AdvertisementDataType(0x20);
    pub const SERVICE_DATA_128: AdvertisementDataType = AdvertisementDataType(0x21);
    pub const APPEARANCE: AdvertisementDataType = AdvertisementDataType(0x19);
    pub const PUBLIC_TARGET_ADDRESS: AdvertisementDataType = AdvertisementDataType(0x17);
    pub const RANDOM_TARGET_ADDRESS: AdvertisementDataType = AdvertisementDataType(0x18);
    pub const ADVERTISING_INTERVAL: AdvertisementDataType = AdvertisementDataType(0x1a);
    pub const URI: AdvertisementDataType = AdvertisementDataType(0x24);
    pub const LE_SUPPORTED_FEATURES: AdvertisementDataType = AdvertisementDataType(0x27);
    pub const MANUFACTURER_SPECIFIC_DATA: AdvertisementDataType = AdvertisementDataType(0xff);

    pub const fn from_u8(value: u8) -> Self {
        AdvertisementDataType(value)
    }

    pub const fn to_u8(self) -> u8 {
        self.0
    }
}

impl From<u8> for AdvertisementDataType {
    fn from(value: u8) -> Self {
        AdvertisementDataType(value)
    }
}

impl From<AdvertisementDataType> for u8 {
    fn from(value: AdvertisementDataType) -> Self {
        value.0
    }
}
