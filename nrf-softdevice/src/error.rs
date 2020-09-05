use num_enum::{FromPrimitive, IntoPrimitive};

use crate::sd;

#[rustfmt::skip]
#[repr(u32)]
#[derive(defmt::Format, IntoPrimitive, FromPrimitive)]
pub enum Error {
    // This is not really an error, but IMO it's better to add it
    // anyway, just in case mistakenly someone converts NRF_SUCCESS into Error.
    // if they see "Success" they'll easily realize their mistake, if they see "Unknown" it'd be confusing.
    Success = sd::NRF_SUCCESS,

    #[num_enum(default)]
    Unknown = 0xFFFFFFFF,

    SvcHandlerMissing = sd::NRF_ERROR_SVC_HANDLER_MISSING,
    SoftdeviceNotEnabled = sd::NRF_ERROR_SOFTDEVICE_NOT_ENABLED,
    Internal = sd::NRF_ERROR_INTERNAL,
    NoMem = sd::NRF_ERROR_NO_MEM,
    NotFound = sd::NRF_ERROR_NOT_FOUND,
    NotSupported = sd::NRF_ERROR_NOT_SUPPORTED,
    InvalidParam = sd::NRF_ERROR_INVALID_PARAM,
    InvalidState = sd::NRF_ERROR_INVALID_STATE,
    InvalidLength = sd::NRF_ERROR_INVALID_LENGTH,
    InvalidFlags = sd::NRF_ERROR_INVALID_FLAGS,
    InvalidData = sd::NRF_ERROR_INVALID_DATA,
    DataSize = sd::NRF_ERROR_DATA_SIZE,
    Timeout = sd::NRF_ERROR_TIMEOUT,
    Null = sd::NRF_ERROR_NULL,
    Forbidden = sd::NRF_ERROR_FORBIDDEN,
    InvalidAddr = sd::NRF_ERROR_INVALID_ADDR,
    Busy = sd::NRF_ERROR_BUSY,
    ConnCount = sd::NRF_ERROR_CONN_COUNT,
    Resources = sd::NRF_ERROR_RESOURCES,
    SdmLfclkSourceUnknown = sd::NRF_ERROR_SDM_LFCLK_SOURCE_UNKNOWN,
    SdmIncorrectInterruptConfiguration = sd::NRF_ERROR_SDM_INCORRECT_INTERRUPT_CONFIGURATION,
    SdmIncorrectClenr0 = sd::NRF_ERROR_SDM_INCORRECT_CLENR0,
    SocMutexAlreadyTaken = sd::NRF_ERROR_SOC_MUTEX_ALREADY_TAKEN,
    SocNvicInterruptNotAvailable = sd::NRF_ERROR_SOC_NVIC_INTERRUPT_NOT_AVAILABLE,
    SocNvicInterruptPriorityNotAllowed = sd::NRF_ERROR_SOC_NVIC_INTERRUPT_PRIORITY_NOT_ALLOWED,
    SocNvicShouldNotReturn = sd::NRF_ERROR_SOC_NVIC_SHOULD_NOT_RETURN,
    SocPowerModeUnknown = sd::NRF_ERROR_SOC_POWER_MODE_UNKNOWN,
    SocPowerPofThresholdUnknown = sd::NRF_ERROR_SOC_POWER_POF_THRESHOLD_UNKNOWN,
    SocPowerOffShouldNotReturn = sd::NRF_ERROR_SOC_POWER_OFF_SHOULD_NOT_RETURN,
    SocRandNotEnoughValues = sd::NRF_ERROR_SOC_RAND_NOT_ENOUGH_VALUES,
    SocPpiInvalidChannel = sd::NRF_ERROR_SOC_PPI_INVALID_CHANNEL,
    SocPpiInvalidGroup = sd::NRF_ERROR_SOC_PPI_INVALID_GROUP,
    BleNotEnabled = sd::BLE_ERROR_NOT_ENABLED,
    BleInvalidConnHandle = sd::BLE_ERROR_INVALID_CONN_HANDLE,
    BleInvalidAttrHandle = sd::BLE_ERROR_INVALID_ATTR_HANDLE,
    BleInvalidAdvHandle = sd::BLE_ERROR_INVALID_ADV_HANDLE,
    BleInvalidRole = sd::BLE_ERROR_INVALID_ROLE,
    BleBlockedByOtherLinks = sd::BLE_ERROR_BLOCKED_BY_OTHER_LINKS,
    BleGapUuidListMismatch = sd::BLE_ERROR_GAP_UUID_LIST_MISMATCH,
    BleGapDiscoverableWithWhitelist = sd::BLE_ERROR_GAP_DISCOVERABLE_WITH_WHITELIST,
    BleGapInvalidBleAddr = sd::BLE_ERROR_GAP_INVALID_BLE_ADDR,
    BleGapWhitelistInUse = sd::BLE_ERROR_GAP_WHITELIST_IN_USE,
    BleGapDeviceIdentitiesInUse = sd::BLE_ERROR_GAP_DEVICE_IDENTITIES_IN_USE,
    BleGapDeviceIdentitiesDuplicate = sd::BLE_ERROR_GAP_DEVICE_IDENTITIES_DUPLICATE,
    BleGattcProcNotPermitted = sd::BLE_ERROR_GATTC_PROC_NOT_PERMITTED,
    BleGattsInvalidAttrType = sd::BLE_ERROR_GATTS_INVALID_ATTR_TYPE,
    BleGattsSysAttrMissing = sd::BLE_ERROR_GATTS_SYS_ATTR_MISSING,
}

impl Error {
    pub fn convert(err: u32) -> Result<(), Error> {
        if err == sd::NRF_SUCCESS {
            Ok(())
        } else {
            Err(Error::from(err))
        }
    }
}
