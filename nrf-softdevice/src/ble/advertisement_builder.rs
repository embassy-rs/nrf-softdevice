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
}

impl AdvertisementDataType {
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
#[repr(u16)]
pub enum BasicService {
    GenericAccess = 0x1800,
    GenericAttribute,
    ImmediateAlert,
    LinkLoss,
    TxPower,
    CurrentTime,
    ReferenceTimeUpdate,
    NextDSTChange,
    Glucose,
    HealthThermometer,
    DeviceInformation,
    HeartRate = 0x180d,
    PhoneAlertStatus,
    Battery,
    BloodPressure,
    AlertNotification,
    HumanInterfaceDevice,
    ScanParameters,
    RunnnigSpeedAndCadence,
    AutomationIO,
    CyclingSpeedAndCadence,
    CyclingPower = 0x1818,
    LocationAndNavigation,
    EnvironmentalSensing,
    BodyComposition,
    UserData,
    WeightScale,
    BondManagement,
    ContinousGlucoseMonitoring,
    InternetProtocolSupport,
    IndoorPositioning,
    PulseOximeter,
    HTTPProxy,
    TransportDiscovery,
    ObjectTransfer,
    FitnessMachine,
    MeshProvisioning,
    MeshProxy,
    ReconnectionConfiguration,
    InsulinDelivery = 0x183a,
    BinarySensor,
    EmergencyConfiguration,
    AuthorizationControl,
    PhysicalActivityMonitor,
    ElapsedTime,
    GenericHealthSensor,
    AudioInputControl = 0x1843,
    VolumeControl,
    VolumeOffsetControl,
    CoordinatedSetIdentification,
    DeviceTime,
    MediaControl,
    GenericMediaControl, // why??
    ConstantToneExtension,
    TelephoneBearer,
    GenericTelephoneBearer,
    MicrophoneControl,
    AudioStreamControl,
    BroadcastAudioScan,
    PublishedAudioScan,
    BasicAudioCapabilities,
    BroadcastAudioAnnouncement,
    CommonAudio,
    HearingAccess,
    TelephonyAndMediaAudio,
    PublicBroadcastAnnouncement,
    ElectronicShelfLabel,
    GamingAudio,
    MeshProxySolicitation,
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
        let end = self.ptr + data.len();

        let mut i = 0;
        while self.ptr < K && i < data.len() {
            self.buf[self.ptr] = data[i];
            i += 1;
            self.ptr += 1;
        }

        self.ptr = end;
        self
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
    pub const fn services_16(self, complete: ServiceList, services: &[BasicService]) -> Self {
        let ad_type = match complete {
            ServiceList::Incomplete => AdvertisementDataType::INCOMPLETE_16_SERVICE_LIST,
            ServiceList::Complete => AdvertisementDataType::COMPLETE_16_SERVICE_LIST,
        };

        let mut res = self.write(&[(services.len() * 2) as u8 + 1, ad_type.to_u8()]);
        let mut i = 0;
        while i < services.len() {
            res = res.write(&(services[i] as u16).to_le_bytes());
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

    /// If the full name fits within the remaining space, it is used. Otherwise the short name is used.
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
