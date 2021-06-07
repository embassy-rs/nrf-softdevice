use num_enum::{FromPrimitive, IntoPrimitive};

use crate::raw;

/// All possible errors returned by softdevice calls.
#[rustfmt::skip]
#[repr(u32)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq, Clone, Copy, IntoPrimitive, FromPrimitive)]
pub enum RawError {
    /// This is not really an error, but is added here anyway, just in case someone mistakenly converts NRF_SUCCESS into RawError.
    Success = raw::NRF_SUCCESS,

    #[num_enum(default)]
    Unknown = 0xFFFFFFFF,

    SvcHandlerMissing = raw::NRF_ERROR_SVC_HANDLER_MISSING,
    SoftdeviceNotEnabled = raw::NRF_ERROR_SOFTDEVICE_NOT_ENABLED,
    Internal = raw::NRF_ERROR_INTERNAL,
    NoMem = raw::NRF_ERROR_NO_MEM,
    NotFound = raw::NRF_ERROR_NOT_FOUND,
    NotSupported = raw::NRF_ERROR_NOT_SUPPORTED,
    InvalidParam = raw::NRF_ERROR_INVALID_PARAM,
    InvalidState = raw::NRF_ERROR_INVALID_STATE,
    InvalidLength = raw::NRF_ERROR_INVALID_LENGTH,
    InvalidFlags = raw::NRF_ERROR_INVALID_FLAGS,
    InvalidData = raw::NRF_ERROR_INVALID_DATA,
    DataSize = raw::NRF_ERROR_DATA_SIZE,
    Timeout = raw::NRF_ERROR_TIMEOUT,
    Null = raw::NRF_ERROR_NULL,
    Forbidden = raw::NRF_ERROR_FORBIDDEN,
    InvalidAddr = raw::NRF_ERROR_INVALID_ADDR,
    Busy = raw::NRF_ERROR_BUSY,
    ConnCount = raw::NRF_ERROR_CONN_COUNT,
    Resources = raw::NRF_ERROR_RESOURCES,
    SdmLfclkSourceUnknown = raw::NRF_ERROR_SDM_LFCLK_SOURCE_UNKNOWN,
    SdmIncorrectInterruptConfiguration = raw::NRF_ERROR_SDM_INCORRECT_INTERRUPT_CONFIGURATION,
    SdmIncorrectClenr0 = raw::NRF_ERROR_SDM_INCORRECT_CLENR0,
    SocMutexAlreadyTaken = raw::NRF_ERROR_SOC_MUTEX_ALREADY_TAKEN,
    SocNvicInterruptNotAvailable = raw::NRF_ERROR_SOC_NVIC_INTERRUPT_NOT_AVAILABLE,
    SocNvicInterruptPriorityNotAllowed = raw::NRF_ERROR_SOC_NVIC_INTERRUPT_PRIORITY_NOT_ALLOWED,
    SocNvicShouldNotReturn = raw::NRF_ERROR_SOC_NVIC_SHOULD_NOT_RETURN,
    SocPowerModeUnknown = raw::NRF_ERROR_SOC_POWER_MODE_UNKNOWN,
    SocPowerPofThresholdUnknown = raw::NRF_ERROR_SOC_POWER_POF_THRESHOLD_UNKNOWN,
    SocPowerOffShouldNotReturn = raw::NRF_ERROR_SOC_POWER_OFF_SHOULD_NOT_RETURN,
    SocRandNotEnoughValues = raw::NRF_ERROR_SOC_RAND_NOT_ENOUGH_VALUES,
    SocPpiInvalidChannel = raw::NRF_ERROR_SOC_PPI_INVALID_CHANNEL,
    SocPpiInvalidGroup = raw::NRF_ERROR_SOC_PPI_INVALID_GROUP,
    BleNotEnabled = raw::BLE_ERROR_NOT_ENABLED,
    BleInvalidConnHandle = raw::BLE_ERROR_INVALID_CONN_HANDLE,
    BleInvalidAttrHandle = raw::BLE_ERROR_INVALID_ATTR_HANDLE,
    BleInvalidAdvHandle = raw::BLE_ERROR_INVALID_ADV_HANDLE,
    BleInvalidRole = raw::BLE_ERROR_INVALID_ROLE,
    BleBlockedByOtherLinks = raw::BLE_ERROR_BLOCKED_BY_OTHER_LINKS,
    BleGapUuidListMismatch = raw::BLE_ERROR_GAP_UUID_LIST_MISMATCH,
    #[cfg(feature="ble-peripheral")]
    BleGapDiscoverableWithWhitelist = raw::BLE_ERROR_GAP_DISCOVERABLE_WITH_WHITELIST,
    BleGapInvalidBleAddr = raw::BLE_ERROR_GAP_INVALID_BLE_ADDR,
    BleGapWhitelistInUse = raw::BLE_ERROR_GAP_WHITELIST_IN_USE,
    BleGapDeviceIdentitiesInUse = raw::BLE_ERROR_GAP_DEVICE_IDENTITIES_IN_USE,
    BleGapDeviceIdentitiesDuplicate = raw::BLE_ERROR_GAP_DEVICE_IDENTITIES_DUPLICATE,
    BleGattcProcNotPermitted = raw::BLE_ERROR_GATTC_PROC_NOT_PERMITTED,
    BleGattsInvalidAttrType = raw::BLE_ERROR_GATTS_INVALID_ATTR_TYPE,
    BleGattsSysAttrMissing = raw::BLE_ERROR_GATTS_SYS_ATTR_MISSING,
}

impl RawError {
    pub fn convert(ret: u32) -> Result<(), RawError> {
        if ret == raw::NRF_SUCCESS {
            Ok(())
        } else {
            Err(RawError::from(ret))
        }
    }
}
