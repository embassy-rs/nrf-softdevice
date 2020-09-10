#![no_std]
#![feature(asm)]
#![feature(generic_associated_types)]
#![feature(const_in_array_repeat_expressions)]
#![feature(type_alias_impl_trait)]
#![feature(const_fn)]

pub(crate) mod util;

// This is here so that the rest of the crate can easily use the right PAC and SD crates.
// TODO change this dynamically based on features.
pub(crate) use nrf52840_pac as pac;
pub(crate) use nrf_softdevice_s140 as sd;

pub mod interrupt;

mod events;
pub use events::*;
mod flash;
pub use flash::*;
mod error;
pub use error::*;
mod ble;
pub use ble::*;

pub use cortex_m_rt::interrupt;

// ====================
use core::ptr;

use crate::util::*;

unsafe extern "C" fn fault_handler(id: u32, pc: u32, info: u32) {
    depanic!("fault_handler {:u32} {:u32} {:u32}", id, pc, info);
}

#[derive(Default)]
pub struct Config {
    pub clock: Option<sd::nrf_clock_lf_cfg_t>,
    pub conn_gap: Option<sd::ble_gap_conn_cfg_t>,
    pub conn_gattc: Option<sd::ble_gattc_conn_cfg_t>,
    pub conn_gatts: Option<sd::ble_gatts_conn_cfg_t>,
    pub conn_gatt: Option<sd::ble_gatt_conn_cfg_t>,
    pub conn_l2cap: Option<sd::ble_l2cap_conn_cfg_t>,
    pub common_vs_uuid: Option<sd::ble_common_cfg_vs_uuid_t>,
    pub gap_role_count: Option<sd::ble_gap_cfg_role_count_t>,
    pub gap_device_name: Option<sd::ble_gap_cfg_device_name_t>,
    pub gap_ppcp_incl: Option<sd::ble_gap_cfg_ppcp_incl_cfg_t>,
    pub gap_car_incl: Option<sd::ble_gap_cfg_car_incl_cfg_t>,
    pub gatts_service_changed: Option<sd::ble_gatts_cfg_service_changed_t>,
    pub gatts_attr_tab_size: Option<sd::ble_gatts_cfg_attr_tab_size_t>,
}

const APP_CONN_CFG_TAG: u8 = 1;

unsafe fn get_app_ram_base() -> u32 {
    extern "C" {
        static mut __sdata: u32;
    }

    (&mut __sdata) as *mut u32 as u32
}

unsafe fn cfg_set(id: u32, cfg: &sd::ble_cfg_t) {
    let app_ram_base = get_app_ram_base();
    let ret = sd::sd_ble_cfg_set(id, cfg, app_ram_base);
    match Error::convert(ret) {
        Ok(()) => {}
        Err(Error::NoMem) => {}
        Err(err) => depanic!("sd_ble_cfg_set {:istr} err {:?}", cfg_id_str(id), err),
    }
}

/// safety: call at most once
pub unsafe fn enable(config: &Config) {
    let p_clock_lf_cfg = config.clock.as_ref().map(|x| x as _).unwrap_or(ptr::null());
    let ret = sd::sd_softdevice_enable(p_clock_lf_cfg, Some(fault_handler));
    match Error::convert(ret) {
        Ok(()) => {}
        Err(err) => depanic!("sd_softdevice_enable err {:?}", err),
    }

    // TODO configure the stack with sd_ble_cfg_set

    let app_ram_base = get_app_ram_base();

    // Set at least one GAP config so APP_CONN_CFG_TAG is usable.
    // If you set none, it seem the softdevice won't let you use it, requiring 0 instead.
    let val = config.conn_gap.unwrap_or(sd::ble_gap_conn_cfg_t {
        conn_count: sd::BLE_GAP_CONN_COUNT_DEFAULT as u8,
        event_length: sd::BLE_GAP_EVENT_LENGTH_DEFAULT as u16,
    });
    cfg_set(
        sd::BLE_CONN_CFGS_BLE_CONN_CFG_GAP,
        &sd::ble_cfg_t {
            conn_cfg: sd::ble_conn_cfg_t {
                conn_cfg_tag: APP_CONN_CFG_TAG,
                params: sd::ble_conn_cfg_t__bindgen_ty_1 { gap_conn_cfg: val },
            },
        },
    );

    if let Some(val) = config.conn_gatt {
        cfg_set(
            sd::BLE_CONN_CFGS_BLE_CONN_CFG_GATT,
            &sd::ble_cfg_t {
                conn_cfg: sd::ble_conn_cfg_t {
                    conn_cfg_tag: APP_CONN_CFG_TAG,
                    params: sd::ble_conn_cfg_t__bindgen_ty_1 { gatt_conn_cfg: val },
                },
            },
        );
    }

    if let Some(val) = config.conn_gattc {
        cfg_set(
            sd::BLE_CONN_CFGS_BLE_CONN_CFG_GATTC,
            &sd::ble_cfg_t {
                conn_cfg: sd::ble_conn_cfg_t {
                    conn_cfg_tag: APP_CONN_CFG_TAG,
                    params: sd::ble_conn_cfg_t__bindgen_ty_1 {
                        gattc_conn_cfg: val,
                    },
                },
            },
        );
    }

    if let Some(val) = config.conn_gatts {
        cfg_set(
            sd::BLE_CONN_CFGS_BLE_CONN_CFG_GATTS,
            &sd::ble_cfg_t {
                conn_cfg: sd::ble_conn_cfg_t {
                    conn_cfg_tag: APP_CONN_CFG_TAG,
                    params: sd::ble_conn_cfg_t__bindgen_ty_1 {
                        gatts_conn_cfg: val,
                    },
                },
            },
        );
    }

    if let Some(val) = config.conn_l2cap {
        cfg_set(
            sd::BLE_CONN_CFGS_BLE_CONN_CFG_L2CAP,
            &sd::ble_cfg_t {
                conn_cfg: sd::ble_conn_cfg_t {
                    conn_cfg_tag: APP_CONN_CFG_TAG,
                    params: sd::ble_conn_cfg_t__bindgen_ty_1 {
                        l2cap_conn_cfg: val,
                    },
                },
            },
        );
    }

    if let Some(val) = config.common_vs_uuid {
        cfg_set(
            sd::BLE_COMMON_CFGS_BLE_COMMON_CFG_VS_UUID,
            &sd::ble_cfg_t {
                common_cfg: sd::ble_common_cfg_t { vs_uuid_cfg: val },
            },
        );
    }

    if let Some(val) = config.gap_role_count {
        cfg_set(
            sd::BLE_GAP_CFGS_BLE_GAP_CFG_ROLE_COUNT,
            &sd::ble_cfg_t {
                gap_cfg: sd::ble_gap_cfg_t {
                    role_count_cfg: val,
                },
            },
        );
    }

    if let Some(val) = config.gap_device_name {
        cfg_set(
            sd::BLE_GAP_CFGS_BLE_GAP_CFG_DEVICE_NAME,
            &sd::ble_cfg_t {
                gap_cfg: sd::ble_gap_cfg_t {
                    device_name_cfg: val,
                },
            },
        );
    }

    if let Some(val) = config.gap_ppcp_incl {
        cfg_set(
            sd::BLE_GAP_CFGS_BLE_GAP_CFG_PPCP_INCL_CONFIG,
            &sd::ble_cfg_t {
                gap_cfg: sd::ble_gap_cfg_t {
                    ppcp_include_cfg: val,
                },
            },
        );
    }

    if let Some(val) = config.gap_car_incl {
        cfg_set(
            sd::BLE_GAP_CFGS_BLE_GAP_CFG_CAR_INCL_CONFIG,
            &sd::ble_cfg_t {
                gap_cfg: sd::ble_gap_cfg_t {
                    car_include_cfg: val,
                },
            },
        );
    }
    if let Some(val) = config.gatts_service_changed {
        cfg_set(
            sd::BLE_GATTS_CFGS_BLE_GATTS_CFG_SERVICE_CHANGED,
            &sd::ble_cfg_t {
                gatts_cfg: sd::ble_gatts_cfg_t {
                    service_changed: val,
                },
            },
        );
    }
    if let Some(val) = config.gatts_attr_tab_size {
        cfg_set(
            sd::BLE_GATTS_CFGS_BLE_GATTS_CFG_ATTR_TAB_SIZE,
            &sd::ble_cfg_t {
                gatts_cfg: sd::ble_gatts_cfg_t { attr_tab_size: val },
            },
        );
    }

    let mut wanted_app_ram_base = app_ram_base;
    let ret = sd::sd_ble_enable(&mut wanted_app_ram_base as _);
    match Error::convert(ret) {
        Ok(()) => {}
        Err(Error::NoMem) => {
            if wanted_app_ram_base <= app_ram_base {
                depanic!("selected configuration has too high RAM requirements.")
            } else {
                depanic!(
                    "too little RAM for softdevice. Change your app's RAM start address to {:u32}",
                    wanted_app_ram_base
                );
            }
        }
        Err(err) => depanic!("sd_ble_enable err {:?}", err),
    }

    if wanted_app_ram_base < app_ram_base {
        warn!("You're giving more RAM to the softdevice than needed. You can change your app's RAM start address to {:u32}", wanted_app_ram_base);
    }

    interrupt::enable(interrupt::Interrupt::SWI2_EGU2);
}

fn cfg_id_str(id: u32) -> defmt::Str {
    match id {
        sd::BLE_CONN_CFGS_BLE_CONN_CFG_GAP => defmt::intern!("BLE_CONN_CFGS_BLE_CONN_CFG_GAP"),
        sd::BLE_CONN_CFGS_BLE_CONN_CFG_GATTC => defmt::intern!("BLE_CONN_CFGS_BLE_CONN_CFG_GATTC"),
        sd::BLE_CONN_CFGS_BLE_CONN_CFG_GATTS => defmt::intern!("BLE_CONN_CFGS_BLE_CONN_CFG_GATTS"),
        sd::BLE_CONN_CFGS_BLE_CONN_CFG_GATT => defmt::intern!("BLE_CONN_CFGS_BLE_CONN_CFG_GATT"),
        sd::BLE_CONN_CFGS_BLE_CONN_CFG_L2CAP => defmt::intern!("BLE_CONN_CFGS_BLE_CONN_CFG_L2CAP"),
        sd::BLE_COMMON_CFGS_BLE_COMMON_CFG_VS_UUID => {
            defmt::intern!("BLE_COMMON_CFGS_BLE_COMMON_CFG_VS_UUID")
        }
        sd::BLE_GAP_CFGS_BLE_GAP_CFG_ROLE_COUNT => {
            defmt::intern!("BLE_GAP_CFGS_BLE_GAP_CFG_ROLE_COUNT")
        }
        sd::BLE_GAP_CFGS_BLE_GAP_CFG_DEVICE_NAME => {
            defmt::intern!("BLE_GAP_CFGS_BLE_GAP_CFG_DEVICE_NAME")
        }
        sd::BLE_GAP_CFGS_BLE_GAP_CFG_PPCP_INCL_CONFIG => {
            defmt::intern!("BLE_GAP_CFGS_BLE_GAP_CFG_PPCP_INCL_CONFIG")
        }
        sd::BLE_GAP_CFGS_BLE_GAP_CFG_CAR_INCL_CONFIG => {
            defmt::intern!("BLE_GAP_CFGS_BLE_GAP_CFG_CAR_INCL_CONFIG")
        }
        sd::BLE_GATTS_CFGS_BLE_GATTS_CFG_SERVICE_CHANGED => {
            defmt::intern!("BLE_GATTS_CFGS_BLE_GATTS_CFG_SERVICE_CHANGED")
        }
        sd::BLE_GATTS_CFGS_BLE_GATTS_CFG_ATTR_TAB_SIZE => {
            defmt::intern!("BLE_GATTS_CFGS_BLE_GATTS_CFG_ATTR_TAB_SIZE")
        }
        _ => defmt::intern!("(unknown)"),
    }
}
