#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub struct ServiceUuid16(u16);

impl ServiceUuid16 {
    pub const GENERIC_ACCESS: Self = Self(0x1800);
    pub const GENERIC_ATTRIBUTE: Self = Self(0x1801);
    pub const IMMEDIATE_ALERT: Self = Self(0x1802);
    pub const LINK_LOSS: Self = Self(0x1803);
    pub const TX_POWER: Self = Self(0x1804);
    pub const CURRENT_TIME: Self = Self(0x1805);
    pub const REFERENCE_TIME_UPDATE: Self = Self(0x1806);
    pub const NEXT_DST_CHANGE: Self = Self(0x1807);
    pub const GLUCOSE: Self = Self(0x1808);
    pub const HEALTH_THERMOMETER: Self = Self(0x1809);
    pub const DEVICE_INFORMATION: Self = Self(0x180A);
    pub const HEART_RATE: Self = Self(0x180D);
    pub const PHONE_ALERT_STATUS: Self = Self(0x180E);
    pub const BATTERY: Self = Self(0x180F);
    pub const BLOOD_PRESSURE: Self = Self(0x1810);
    pub const ALERT_NOTIFICATION: Self = Self(0x1811);
    pub const HUMAN_INTERFACE_DEVICE: Self = Self(0x1812);
    pub const SCAN_PARAMETERS: Self = Self(0x1813);
    pub const RUNNNIG_SPEED_AND_CADENCE: Self = Self(0x1814);
    pub const AUTOMATION_IO: Self = Self(0x1815);
    pub const CYCLING_SPEED_AND_CADENCE: Self = Self(0x1816);
    pub const CYCLING_POWER: Self = Self(0x1818);
    pub const LOCATION_AND_NAVIGATION: Self = Self(0x1819);
    pub const ENVIRONMENTAL_SENSING: Self = Self(0x181A);
    pub const BODY_COMPOSITION: Self = Self(0x181B);
    pub const USER_DATA: Self = Self(0x181C);
    pub const WEIGHT_SCALE: Self = Self(0x181D);
    pub const BOND_MANAGEMENT: Self = Self(0x181E);
    pub const CONTINOUS_GLUCOSE_MONITORING: Self = Self(0x181F);
    pub const INTERNET_PROTOCOL_SUPPORT: Self = Self(0x1820);
    pub const INDOOR_POSITIONING: Self = Self(0x1821);
    pub const PULSE_OXIMETER: Self = Self(0x1822);
    pub const HTTP_PROXY: Self = Self(0x1823);
    pub const TRANSPORT_DISCOVERY: Self = Self(0x1824);
    pub const OBJECT_TRANSFER: Self = Self(0x1825);
    pub const FITNESS_MACHINE: Self = Self(0x1826);
    pub const MESH_PROVISIONING: Self = Self(0x1827);
    pub const MESH_PROXY: Self = Self(0x1828);
    pub const RECONNECTION_CONFIGURATION: Self = Self(0x1829);
    pub const INSULIN_DELIVERY: Self = Self(0x183A);
    pub const BINARY_SENSOR: Self = Self(0x183B);
    pub const EMERGENCY_CONFIGURATION: Self = Self(0x183C);
    pub const AUTHORIZATION_CONTROL: Self = Self(0x183D);
    pub const PHYSICAL_ACTIVITY_MONITOR: Self = Self(0x183E);
    pub const ELAPSED_TIME: Self = Self(0x183F);
    pub const GENERIC_HEALTH_SENSOR: Self = Self(0x1840);
    pub const AUDIO_INPUT_CONTROL: Self = Self(0x1843);
    pub const VOLUME_CONTROL: Self = Self(0x1844);
    pub const VOLUME_OFFSET_CONTROL: Self = Self(0x1845);
    pub const COORDINATED_SET_IDENTIFICATION: Self = Self(0x1846);
    pub const DEVICE_TIME: Self = Self(0x1847);
    pub const MEDIA_CONTROL: Self = Self(0x1848);
    pub const GENERIC_MEDIA_CONTROL: Self = Self(0x1849);
    pub const CONSTANT_TONE_EXTENSION: Self = Self(0x184A);
    pub const TELEPHONE_BEARER: Self = Self(0x184B);
    pub const GENERIC_TELEPHONE_BEARER: Self = Self(0x184C);
    pub const MICROPHONE_CONTROL: Self = Self(0x184D);
    pub const AUDIO_STREAM_CONTROL: Self = Self(0x184E);
    pub const BROADCAST_AUDIO_SCAN: Self = Self(0x184F);
    pub const PUBLISHED_AUDIO_SCAN: Self = Self(0x1850);
    pub const BASIC_AUDIO_CAPABILITIES: Self = Self(0x1851);
    pub const BROADCAST_AUDIO_ANNOUNCEMENT: Self = Self(0x1852);
    pub const COMMON_AUDIO: Self = Self(0x1853);
    pub const HEARING_ACCESS: Self = Self(0x1854);
    pub const TELEPHONY_AND_MEDIA_AUDIO: Self = Self(0x1855);
    pub const PUBLIC_BROADCAST_ANNOUNCEMENT: Self = Self(0x1856);
    pub const ELECTRONIC_SHELF_LABEL: Self = Self(0x1847);
    pub const GAMING_AUDIO: Self = Self(0x1858);
    pub const MESH_PROXY_SOLICITATION: Self = Self(0x1859);

    pub const fn from_u16(value: u16) -> Self {
        Self(value)
    }

    pub const fn to_u16(self) -> u16 {
        self.0
    }
}

impl From<u16> for ServiceUuid16 {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<ServiceUuid16> for u16 {
    fn from(value: ServiceUuid16) -> Self {
        value.0
    }
}
