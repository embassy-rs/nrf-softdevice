#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
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

    pub const fn raw(self) -> u16 {
        self.0
    }
}

// impl From<u16> for ServiceUuid16 {
//     fn from(value: u16) -> Self {
//         ServiceUuid16(value)
//     }
// }

// impl From<ServiceUuid16> for u16 {
//     fn from(value: ServiceUuid16) -> Self {
//         value.0
//     }
// }
