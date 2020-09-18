use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::ble;
use crate::interrupt;
use crate::raw;
use crate::util::*;
use crate::RawError;

unsafe extern "C" fn fault_handler(id: u32, pc: u32, info: u32) {
    depanic!("fault_handler {:u32} {:u32} {:u32}", id, pc, info);
}

/// Singleton instance of the enabled softdevice.
///
/// The `Softdevice` instance can be obtaind by enabling it with [`Softdevice::enable`]. Once
/// enabled, it can be used to establish Bluetooth connections with [`ble::central`] and [`ble::peripheral`].
///
/// Disabling the softdevice is not supported due to the complexity of a safe implementation. Consider resetting the CPU instead.
pub struct Softdevice {
    // Prevent Send, Sync
    _private: PhantomData<*mut ()>,
}

/// Softdevice configuration.
///
/// Fields set to None will use a default configuration.
#[derive(Default)]
pub struct Config {
    pub clock: Option<raw::nrf_clock_lf_cfg_t>,
    pub conn_gap: Option<raw::ble_gap_conn_cfg_t>,
    pub conn_gattc: Option<raw::ble_gattc_conn_cfg_t>,
    pub conn_gatts: Option<raw::ble_gatts_conn_cfg_t>,
    pub conn_gatt: Option<raw::ble_gatt_conn_cfg_t>,
    #[cfg(feature = "ble-l2cap")]
    pub conn_l2cap: Option<raw::ble_l2cap_conn_cfg_t>,
    pub common_vs_uuid: Option<raw::ble_common_cfg_vs_uuid_t>,
    pub gap_role_count: Option<raw::ble_gap_cfg_role_count_t>,
    pub gap_device_name: Option<raw::ble_gap_cfg_device_name_t>,
    pub gap_ppcp_incl: Option<raw::ble_gap_cfg_ppcp_incl_cfg_t>,
    pub gap_car_incl: Option<raw::ble_gap_cfg_car_incl_cfg_t>,
    pub gatts_service_changed: Option<raw::ble_gatts_cfg_service_changed_t>,
    pub gatts_attr_tab_size: Option<raw::ble_gatts_cfg_attr_tab_size_t>,
}

const APP_CONN_CFG_TAG: u8 = 1;

fn get_app_ram_base() -> u32 {
    extern "C" {
        static mut __sdata: u32;
    }

    unsafe { &mut __sdata as *mut u32 as u32 }
}

fn cfg_set(id: u32, cfg: &raw::ble_cfg_t) {
    let app_ram_base = get_app_ram_base();
    let ret = unsafe { raw::sd_ble_cfg_set(id, cfg, app_ram_base) };
    match RawError::convert(ret) {
        Ok(()) => {}
        Err(RawError::NoMem) => {}
        Err(err) => depanic!("sd_ble_cfg_set {:istr} err {:?}", cfg_id_str(id), err),
    }
}

fn cfg_id_str(id: u32) -> defmt::Str {
    match id {
        raw::BLE_CONN_CFGS_BLE_CONN_CFG_GAP => defmt::intern!("BLE_CONN_CFGS_BLE_CONN_CFG_GAP"),
        raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATTC => defmt::intern!("BLE_CONN_CFGS_BLE_CONN_CFG_GATTC"),
        raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATTS => defmt::intern!("BLE_CONN_CFGS_BLE_CONN_CFG_GATTS"),
        raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATT => defmt::intern!("BLE_CONN_CFGS_BLE_CONN_CFG_GATT"),
        #[cfg(feature = "ble-l2cap")]
        raw::BLE_CONN_CFGS_BLE_CONN_CFG_L2CAP => defmt::intern!("BLE_CONN_CFGS_BLE_CONN_CFG_L2CAP"),
        raw::BLE_COMMON_CFGS_BLE_COMMON_CFG_VS_UUID => {
            defmt::intern!("BLE_COMMON_CFGS_BLE_COMMON_CFG_VS_UUID")
        }
        raw::BLE_GAP_CFGS_BLE_GAP_CFG_ROLE_COUNT => {
            defmt::intern!("BLE_GAP_CFGS_BLE_GAP_CFG_ROLE_COUNT")
        }
        raw::BLE_GAP_CFGS_BLE_GAP_CFG_DEVICE_NAME => {
            defmt::intern!("BLE_GAP_CFGS_BLE_GAP_CFG_DEVICE_NAME")
        }
        raw::BLE_GAP_CFGS_BLE_GAP_CFG_PPCP_INCL_CONFIG => {
            defmt::intern!("BLE_GAP_CFGS_BLE_GAP_CFG_PPCP_INCL_CONFIG")
        }
        raw::BLE_GAP_CFGS_BLE_GAP_CFG_CAR_INCL_CONFIG => {
            defmt::intern!("BLE_GAP_CFGS_BLE_GAP_CFG_CAR_INCL_CONFIG")
        }
        raw::BLE_GATTS_CFGS_BLE_GATTS_CFG_SERVICE_CHANGED => {
            defmt::intern!("BLE_GATTS_CFGS_BLE_GATTS_CFG_SERVICE_CHANGED")
        }
        raw::BLE_GATTS_CFGS_BLE_GATTS_CFG_ATTR_TAB_SIZE => {
            defmt::intern!("BLE_GATTS_CFGS_BLE_GATTS_CFG_ATTR_TAB_SIZE")
        }
        _ => defmt::intern!("(unknown)"),
    }
}

static ENABLED: AtomicBool = AtomicBool::new(false);
static mut SOFTDEVICE: Softdevice = Softdevice {
    _private: PhantomData,
};

impl Softdevice {
    /// Enable the softdevice with the requested configuration.
    ///
    /// # Panics
    /// - Panics if the requested configuration requires more memory than reserved for the softdevice. In that case, you can give more memory to the softdevice by editing the RAM start address in `memory.x`. The required start address is logged using `defmt` prior to panic.
    /// - Panics if the requested configuration has too high memory requirements for the softdevice. The softdevice supports a maximum dynamic memory size of 64kb.
    /// - Panics if called multiple times. Must be called at most once.
    pub fn enable(config: &Config) -> &'static Softdevice {
        if ENABLED.compare_and_swap(false, true, Ordering::AcqRel) {
            depanic!("nrf_softdevice::enable() called multiple times.")
        }

        let p_clock_lf_cfg = config.clock.as_ref().map(|x| x as _).unwrap_or(ptr::null());
        let ret = unsafe { raw::sd_softdevice_enable(p_clock_lf_cfg, Some(fault_handler)) };
        match RawError::convert(ret) {
            Ok(()) => {}
            Err(err) => depanic!("sd_softdevice_enable err {:?}", err),
        }

        let app_ram_base = get_app_ram_base();

        // Set at least one GAP config so conn_cfg_tag 1 (APP_CONN_CFG_TAG) is usable.
        // If you set none, it seems the softdevice won't let you use it, requiring a conn_cfg_tag of 0 (raw::BLE_CONN_CFG_TAG_DEFAULT) instead.
        let val = config.conn_gap.unwrap_or(raw::ble_gap_conn_cfg_t {
            conn_count: raw::BLE_GAP_CONN_COUNT_DEFAULT as u8,
            event_length: raw::BLE_GAP_EVENT_LENGTH_DEFAULT as u16,
        });
        cfg_set(
            raw::BLE_CONN_CFGS_BLE_CONN_CFG_GAP,
            &raw::ble_cfg_t {
                conn_cfg: raw::ble_conn_cfg_t {
                    conn_cfg_tag: APP_CONN_CFG_TAG,
                    params: raw::ble_conn_cfg_t__bindgen_ty_1 { gap_conn_cfg: val },
                },
            },
        );

        if let Some(val) = config.conn_gatt {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATT,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 { gatt_conn_cfg: val },
                    },
                },
            );
        }

        if let Some(val) = config.conn_gattc {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATTC,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 {
                            gattc_conn_cfg: val,
                        },
                    },
                },
            );
        }

        if let Some(val) = config.conn_gatts {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATTS,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 {
                            gatts_conn_cfg: val,
                        },
                    },
                },
            );
        }

        #[cfg(feature = "ble-l2cap")]
        if let Some(val) = config.conn_l2cap {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_L2CAP,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 {
                            l2cap_conn_cfg: val,
                        },
                    },
                },
            );
        }

        if let Some(val) = config.common_vs_uuid {
            cfg_set(
                raw::BLE_COMMON_CFGS_BLE_COMMON_CFG_VS_UUID,
                &raw::ble_cfg_t {
                    common_cfg: raw::ble_common_cfg_t { vs_uuid_cfg: val },
                },
            );
        }

        if let Some(val) = config.gap_role_count {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_ROLE_COUNT,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t {
                        role_count_cfg: val,
                    },
                },
            );
        }

        if let Some(val) = config.gap_device_name {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_DEVICE_NAME,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t {
                        device_name_cfg: val,
                    },
                },
            );
        }

        if let Some(val) = config.gap_ppcp_incl {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_PPCP_INCL_CONFIG,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t {
                        ppcp_include_cfg: val,
                    },
                },
            );
        }

        if let Some(val) = config.gap_car_incl {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_CAR_INCL_CONFIG,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t {
                        car_include_cfg: val,
                    },
                },
            );
        }
        if let Some(val) = config.gatts_service_changed {
            cfg_set(
                raw::BLE_GATTS_CFGS_BLE_GATTS_CFG_SERVICE_CHANGED,
                &raw::ble_cfg_t {
                    gatts_cfg: raw::ble_gatts_cfg_t {
                        service_changed: val,
                    },
                },
            );
        }
        if let Some(val) = config.gatts_attr_tab_size {
            cfg_set(
                raw::BLE_GATTS_CFGS_BLE_GATTS_CFG_ATTR_TAB_SIZE,
                &raw::ble_cfg_t {
                    gatts_cfg: raw::ble_gatts_cfg_t { attr_tab_size: val },
                },
            );
        }

        let mut wanted_app_ram_base = app_ram_base;
        let ret = unsafe { raw::sd_ble_enable(&mut wanted_app_ram_base as _) };
        match RawError::convert(ret) {
            Ok(()) => {}
            Err(RawError::NoMem) => {
                if wanted_app_ram_base <= app_ram_base {
                    depanic!("selected configuration has too high RAM requirements.")
                } else {
                    depanic!("too little RAM for softdevice. Change your app's RAM start address to {:u32}", wanted_app_ram_base);
                }
            }
            Err(err) => depanic!("sd_ble_enable err {:?}", err),
        }

        if wanted_app_ram_base < app_ram_base {
            warn!("You're giving more RAM to the softdevice than needed. You can change your app's RAM start address to {:u32}", wanted_app_ram_base);
        }

        #[cfg(feature = "nrf52810")]
        interrupt::enable(interrupt::Interrupt::SWI2);
        #[cfg(not(feature = "nrf52810"))]
        interrupt::enable(interrupt::Interrupt::SWI2_EGU2);

        unsafe { &SOFTDEVICE }
    }

    /// Return an instance to the softdevice without checking whether
    /// it is enabled or not. This is only safe if the softdevice is enabled
    /// (a call to [`enable`] has returned without error)
    pub unsafe fn steal() -> &'static Softdevice {
        &SOFTDEVICE
    }

    /// Runs the softdevice event handling loop.
    ///
    /// It must be called in its own async task after enabling the softdevice
    /// and before doing any operation. Failure to doing so will cause async operations to never finish.
    pub async fn run(&self) {
        crate::events::run().await;
    }
}
