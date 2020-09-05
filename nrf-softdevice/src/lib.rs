#![no_std]
#![feature(asm)]
#![feature(generic_associated_types)]
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

pub use cortex_m_rt::interrupt;

use defmt::{info, warn};

unsafe extern "C" fn fault_handler(id: u32, pc: u32, info: u32) {
    depanic!("fault_handler {:u32} {:u32} {:u32}", id, pc, info);
}

/// safety: call at most once
pub unsafe fn enable() {
    // TODO make this configurable via features or param
    let clock_cfg = sd::nrf_clock_lf_cfg_t {
        source: sd::NRF_CLOCK_LF_SRC_XTAL as u8,
        rc_ctiv: 0,
        rc_temp_ctiv: 0,
        accuracy: 7,
    };

    let ret = sd::sd_softdevice_enable(&clock_cfg as _, Some(fault_handler));
    match Error::convert(ret) {
        Ok(()) => {}
        Err(err) => depanic!("sd_softdevice_enable err {:?}", err),
    }

    extern "C" {
        static mut __sdata: u32;
    }

    // TODO configure the stack with sd_ble_cfg_set

    let app_ram_base: u32 = (&mut __sdata) as *mut u32 as u32;
    let mut wanted_app_ram_base = app_ram_base;
    let ret = sd::sd_ble_enable(&mut wanted_app_ram_base as _);
    match ret {
        sd::NRF_SUCCESS => {}
        sd::NRF_ERROR_NO_MEM => depanic!(
            "too little RAM for softdevice. Change your app's RAM start address to {:u32}",
            wanted_app_ram_base
        ),
        _ => depanic!("sd_ble_enable ret {:u32}", ret),
    }

    if wanted_app_ram_base < app_ram_base {
        warn!("You're giving more RAM to the softdevice than needed. You can change your app's RAM start address to {:u32}", wanted_app_ram_base);
    }

    interrupt::enable(interrupt::Interrupt::SWI2_EGU2);
}
