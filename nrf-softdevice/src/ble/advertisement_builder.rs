#[cfg(feature = "defmt")]
use defmt::Format;

const LEGACY_PAYLOAD_LEN: usize = 31;
const EXTENDED_PAYLOAD_LEN: usize = 254;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum Error {
    Oversize { expected: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(Format))]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(Format))]
#[repr(u8)]
pub enum Flag {
    LimitedDiscovery = 0b1,
    GeneralDiscovery = 0b10,
    #[allow(non_camel_case_types)]
    LE_Only = 0b100,

    // i don't understand these but in case people want them
    Bit3 = 0b1000,
    Bit4 = 0b10000,
    // the rest are "reserved for future use"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub enum ServiceList {
    Incomplete,
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(Format))]
pub struct ServiceUuid16(u16);

impl ServiceUuid16 {
    pub const GENERIC_ACCESS: ServiceUuid16 = ServiceUuid16(0x1800);
    pub const GENERIC_ATTRIBUTE: ServiceUuid16 = ServiceUuid16(0x1801);
    pub const IMMEDIATE_ALERT: ServiceUuid16 = ServiceUuid16(0x1802);
    pub const LINK_LOSS: ServiceUuid16 = ServiceUuid16(0x1803);
    pub const TX_POWER: ServiceUuid16 = ServiceUuid16(0x1804);
    pub const CURRENT_TIME: ServiceUuid16 = ServiceUuid16(0x1805);
    pub const REFERENCE_TIME_UPDATE: ServiceUuid16 = ServiceUuid16(0x1806);
    pub const NEXT_DST_CHANGE: ServiceUuid16 = ServiceUuid16(0x1807);
    pub const GLUCOSE: ServiceUuid16 = ServiceUuid16(0x1808);
    pub const HEALTH_THERMOMETER: ServiceUuid16 = ServiceUuid16(0x1809);
    pub const DEVICE_INFORMATION: ServiceUuid16 = ServiceUuid16(0x180A);
    pub const HEART_RATE: ServiceUuid16 = ServiceUuid16(0x180D);
    pub const PHONE_ALERT_STATUS: ServiceUuid16 = ServiceUuid16(0x180E);
    pub const BATTERY: ServiceUuid16 = ServiceUuid16(0x180F);
    pub const BLOOD_PRESSURE: ServiceUuid16 = ServiceUuid16(0x1810);
    pub const ALERT_NOTIFICATION: ServiceUuid16 = ServiceUuid16(0x1811);
    pub const HUMAN_INTERFACE_DEVICE: ServiceUuid16 = ServiceUuid16(0x1812);
    pub const SCAN_PARAMETERS: ServiceUuid16 = ServiceUuid16(0x1813);
    pub const RUNNNIG_SPEED_AND_CADENCE: ServiceUuid16 = ServiceUuid16(0x1814);
    pub const AUTOMATION_IO: ServiceUuid16 = ServiceUuid16(0x1815);
    pub const CYCLING_SPEED_AND_CADENCE: ServiceUuid16 = ServiceUuid16(0x1816);
    pub const CYCLING_POWER: ServiceUuid16 = ServiceUuid16(0x1818);
    pub const LOCATION_AND_NAVIGATION: ServiceUuid16 = ServiceUuid16(0x1819);
    pub const ENVIRONMENTAL_SENSING: ServiceUuid16 = ServiceUuid16(0x181A);
    pub const BODY_COMPOSITION: ServiceUuid16 = ServiceUuid16(0x181B);
    pub const USER_DATA: ServiceUuid16 = ServiceUuid16(0x181C);
    pub const WEIGHT_SCALE: ServiceUuid16 = ServiceUuid16(0x181D);
    pub const BOND_MANAGEMENT: ServiceUuid16 = ServiceUuid16(0x181E);
    pub const CONTINOUS_GLUCOSE_MONITORING: ServiceUuid16 = ServiceUuid16(0x181F);
    pub const INTERNET_PROTOCOL_SUPPORT: ServiceUuid16 = ServiceUuid16(0x1820);
    pub const INDOOR_POSITIONING: ServiceUuid16 = ServiceUuid16(0x1821);
    pub const PULSE_OXIMETER: ServiceUuid16 = ServiceUuid16(0x1822);
    pub const HTTP_PROXY: ServiceUuid16 = ServiceUuid16(0x1823);
    pub const TRANSPORT_DISCOVERY: ServiceUuid16 = ServiceUuid16(0x1824);
    pub const OBJECT_TRANSFER: ServiceUuid16 = ServiceUuid16(0x1825);
    pub const FITNESS_MACHINE: ServiceUuid16 = ServiceUuid16(0x1826);
    pub const MESH_PROVISIONING: ServiceUuid16 = ServiceUuid16(0x1827);
    pub const MESH_PROXY: ServiceUuid16 = ServiceUuid16(0x1828);
    pub const RECONNECTION_CONFIGURATION: ServiceUuid16 = ServiceUuid16(0x1829);
    pub const INSULIN_DELIVERY: ServiceUuid16 = ServiceUuid16(0x183A);
    pub const BINARY_SENSOR: ServiceUuid16 = ServiceUuid16(0x183B);
    pub const EMERGENCY_CONFIGURATION: ServiceUuid16 = ServiceUuid16(0x183C);
    pub const AUTHORIZATION_CONTROL: ServiceUuid16 = ServiceUuid16(0x183D);
    pub const PHYSICAL_ACTIVITY_MONITOR: ServiceUuid16 = ServiceUuid16(0x183E);
    pub const ELAPSED_TIME: ServiceUuid16 = ServiceUuid16(0x183F);
    pub const GENERIC_HEALTH_SENSOR: ServiceUuid16 = ServiceUuid16(0x1840);
    pub const AUDIO_INPUT_CONTROL: ServiceUuid16 = ServiceUuid16(0x1843);
    pub const VOLUME_CONTROL: ServiceUuid16 = ServiceUuid16(0x1844);
    pub const VOLUME_OFFSET_CONTROL: ServiceUuid16 = ServiceUuid16(0x1845);
    pub const COORDINATED_SET_IDENTIFICATION: ServiceUuid16 = ServiceUuid16(0x1846);
    pub const DEVICE_TIME: ServiceUuid16 = ServiceUuid16(0x1847);
    pub const MEDIA_CONTROL: ServiceUuid16 = ServiceUuid16(0x1848);
    pub const GENERIC_MEDIA_CONTROL: ServiceUuid16 = ServiceUuid16(0x1849);
    pub const CONSTANT_TONE_EXTENSION: ServiceUuid16 = ServiceUuid16(0x184A);
    pub const TELEPHONE_BEARER: ServiceUuid16 = ServiceUuid16(0x184B);
    pub const GENERIC_TELEPHONE_BEARER: ServiceUuid16 = ServiceUuid16(0x184C);
    pub const MICROPHONE_CONTROL: ServiceUuid16 = ServiceUuid16(0x184D);
    pub const AUDIO_STREAM_CONTROL: ServiceUuid16 = ServiceUuid16(0x184E);
    pub const BROADCAST_AUDIO_SCAN: ServiceUuid16 = ServiceUuid16(0x184F);
    pub const PUBLISHED_AUDIO_SCAN: ServiceUuid16 = ServiceUuid16(0x1850);
    pub const BASIC_AUDIO_CAPABILITIES: ServiceUuid16 = ServiceUuid16(0x1851);
    pub const BROADCAST_AUDIO_ANNOUNCEMENT: ServiceUuid16 = ServiceUuid16(0x1852);
    pub const COMMON_AUDIO: ServiceUuid16 = ServiceUuid16(0x1853);
    pub const HEARING_ACCESS: ServiceUuid16 = ServiceUuid16(0x1854);
    pub const TELEPHONY_AND_MEDIA_AUDIO: ServiceUuid16 = ServiceUuid16(0x1855);
    pub const PUBLIC_BROADCAST_ANNOUNCEMENT: ServiceUuid16 = ServiceUuid16(0x1856);
    pub const ELECTRONIC_SHELF_LABEL: ServiceUuid16 = ServiceUuid16(0x1847);
    pub const GAMING_AUDIO: ServiceUuid16 = ServiceUuid16(0x1858);
    pub const MESH_PROXY_SOLICITATION: ServiceUuid16 = ServiceUuid16(0x1859);

    pub const fn from_u16(value: u16) -> Self {
        ServiceUuid16(value)
    }

    pub const fn to_u16(self) -> u16 {
        self.0
    }
}

impl From<u16> for ServiceUuid16 {
    fn from(value: u16) -> Self {
        ServiceUuid16(value)
    }
}

impl From<ServiceUuid16> for u16 {
    fn from(value: ServiceUuid16) -> Self {
        value.0
    }
}

pub struct AdvertisementBuilder<const N: usize> {
    buf: [u8; N],
    ptr: usize,
}

pub struct AdvertisementPayload<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> AsRef<[u8]> for AdvertisementPayload<N> {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl<const N: usize> core::ops::Deref for AdvertisementPayload<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buf[..self.len]
    }
}

impl<const K: usize> AdvertisementBuilder<K> {
    pub const fn new() -> Self {
        Self { buf: [0; K], ptr: 0 }
    }

    const fn write(mut self, data: &[u8]) -> Self {
        if self.ptr + data.len() <= K {
            let mut i = 0;
            while i < data.len() {
                self.buf[self.ptr] = data[i];
                i += 1;
                self.ptr += 1;
            }
        } else {
            // Overflow, but still track how much data was attempted to be written
            self.ptr += data.len();
        }

        self
    }

    pub const fn capacity() -> usize {
        K
    }

    pub const fn len(&self) -> usize {
        self.ptr
    }

    /// Write raw bytes to the advertisement data.
    ///
    /// *Note: The length is automatically computed and prepended.*
    pub const fn raw(self, ad: AdvertisementDataType, data: &[u8]) -> Self {
        self.write(&[data.len() as u8 + 1, ad.to_u8()]).write(data)
    }

    /// Get the resulting advertisement payload.
    ///
    /// Returns `Error::Oversize` if more than `K` bytes were written to the builder.
    pub const fn try_build(self) -> Result<AdvertisementPayload<K>, Error> {
        if self.ptr <= K {
            Ok(AdvertisementPayload {
                buf: self.buf,
                len: self.ptr,
            })
        } else {
            Err(Error::Oversize { expected: self.ptr })
        }
    }

    /// Get the resulting advertisement payload.
    ///
    /// Panics if more than `K` bytes were written to the builder.
    pub const fn build(self) -> AdvertisementPayload<K> {
        // Use core::assert! even if defmt is enabled because it is const
        core::assert!(self.ptr <= K, "advertisement exceeded buffer length");

        AdvertisementPayload {
            buf: self.buf,
            len: self.ptr,
        }
    }

    /// Add flags to the advertisement data.
    pub const fn flags(self, flags: &[Flag]) -> Self {
        let mut i = 0;
        let mut bits = 0;
        while i < flags.len() {
            bits |= flags[i] as u8;
            i += 1;
        }

        self.raw(AdvertisementDataType::FLAGS, &[bits])
    }

    /// Add a list of 16-bit service uuids to the advertisement data.
    pub const fn services_16(self, complete: ServiceList, services: &[ServiceUuid16]) -> Self {
        let ad_type = match complete {
            ServiceList::Incomplete => AdvertisementDataType::INCOMPLETE_16_SERVICE_LIST,
            ServiceList::Complete => AdvertisementDataType::COMPLETE_16_SERVICE_LIST,
        };

        let mut res = self.write(&[(services.len() * 2) as u8 + 1, ad_type.to_u8()]);
        let mut i = 0;
        while i < services.len() {
            res = res.write(&(services[i].to_u16()).to_le_bytes());
            i += 1;
        }
        res
    }

    /// Add a list of 128-bit service uuids to the advertisement data.
    ///
    /// Note that each UUID in the list needs to be in little-endian format, i.e. opposite to what you would
    /// normally write UUIDs.
    pub const fn services_128(self, complete: ServiceList, services: &[[u8; 16]]) -> Self {
        let ad_type = match complete {
            ServiceList::Incomplete => AdvertisementDataType::INCOMPLETE_128_SERVICE_LIST,
            ServiceList::Complete => AdvertisementDataType::COMPLETE_128_SERVICE_LIST,
        };

        let mut res = self.write(&[(services.len() * 16) as u8 + 1, ad_type.to_u8()]);
        let mut i = 0;
        while i < services.len() {
            res = res.write(&services[i]);
            i += 1;
        }
        res
    }

    /// Add a name to the advertisement data.
    pub const fn short_name(self, name: &str) -> Self {
        self.raw(AdvertisementDataType::SHORT_NAME, name.as_bytes())
    }

    /// Add a name to the advertisement data.
    pub const fn full_name(self, name: &str) -> Self {
        self.raw(AdvertisementDataType::FULL_NAME, name.as_bytes())
    }

    /// Adds the provided string as a name, truncating and typing as needed.
    ///
    /// *Note: This modifier should be placed last.*
    pub const fn adapt_name(self, name: &str) -> Self {
        let p = self.ptr;
        if p + 2 + name.len() <= K {
            self.full_name(name)
        } else {
            let mut res = self.write(&[(K - p) as u8, AdvertisementDataType::SHORT_NAME.to_u8()]);
            let mut i: usize = 0;
            let bytes = name.as_bytes();
            while res.ptr < K {
                res.buf[res.ptr] = bytes[i];
                res.ptr += 1;
                i += 1;
            }
            res
        }
    }
}

pub type LegacyAdvertisementBuilder = AdvertisementBuilder<LEGACY_PAYLOAD_LEN>;
pub type ExtendedAdvertisementBuilder = AdvertisementBuilder<EXTENDED_PAYLOAD_LEN>;

pub type LegacyAdvertisementPayload = AdvertisementPayload<LEGACY_PAYLOAD_LEN>;
pub type ExtendedAdvertisementPayload = AdvertisementPayload<EXTENDED_PAYLOAD_LEN>;
