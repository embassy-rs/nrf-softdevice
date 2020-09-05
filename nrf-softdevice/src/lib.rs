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

    crate::interrupt::unmask(pac::Interrupt::SWI2_EGU2);
}
