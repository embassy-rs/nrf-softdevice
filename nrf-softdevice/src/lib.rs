#![no_std]
#![feature(asm)]
#![feature(generic_associated_types)]
#![feature(const_in_array_repeat_expressions)]
#![feature(type_alias_impl_trait)]
#![feature(const_fn)]

pub(crate) mod util;

#[cfg(feature = "nrf52810")]
pub use nrf52810_pac as pac;
#[cfg(feature = "nrf52832")]
pub use nrf52832_pac as pac;
#[cfg(feature = "nrf52833")]
pub use nrf52833_pac as pac;
#[cfg(feature = "nrf52840")]
pub use nrf52840_pac as pac;

#[cfg(feature = "s112")]
pub use nrf_softdevice_s112 as raw;
#[cfg(feature = "s113")]
pub use nrf_softdevice_s113 as raw;
#[cfg(feature = "s122")]
pub use nrf_softdevice_s122 as raw;
#[cfg(feature = "s132")]
pub use nrf_softdevice_s132 as raw;
#[cfg(feature = "s140")]
pub use nrf_softdevice_s140 as raw;

pub mod interrupt;

mod events;
pub use events::*;
mod flash;
pub use flash::*;
mod error;
pub use error::*;
pub mod ble;
mod softdevice;
pub use softdevice::*;

pub use cortex_m_rt::interrupt;
