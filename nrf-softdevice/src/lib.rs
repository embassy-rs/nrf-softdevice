#![no_std]
#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]

pub(crate) mod util;

#[cfg(not(any(feature = "ble-central", feature = "ble-peripheral",)))]
compile_error!("You must activate at least one of the following features: ble-central, ble-peripheral");

#[cfg(not(any(
    feature = "s112",
    feature = "s113",
    feature = "s122",
    feature = "s132",
    feature = "s140"
)))]
compile_error!("No softdevice feature activated. You must activate exactly one of the following features: s112, s113, s122, s132, s140");

// I don't know how to do this with less than O(n^2)... :(
#[cfg(any(
    all(feature = "s112", feature = "s113"),
    all(feature = "s112", feature = "s122"),
    all(feature = "s112", feature = "s132"),
    all(feature = "s112", feature = "s140"),
    all(feature = "s113", feature = "s122"),
    all(feature = "s113", feature = "s132"),
    all(feature = "s113", feature = "s140"),
    all(feature = "s122", feature = "s132"),
    all(feature = "s122", feature = "s140"),
    all(feature = "s132", feature = "s140"),
))]
compile_error!("Multiple softdevice features activated. You must activate exactly one of the following features: s112, s113, s122, s132, s140");

#[cfg(not(any(
    feature = "nrf52805",
    feature = "nrf52810",
    feature = "nrf52811",
    feature = "nrf52820",
    feature = "nrf52832",
    feature = "nrf52833",
    feature = "nrf52840",
)))]
compile_error!("No chip feature activated. You must activate exactly one of the following features: nrf52810, nrf52811, nrf52832, nrf52833, nrf52840");

#[cfg(any(
    all(feature = "nrf52805", feature = "nrf52810"),
    all(feature = "nrf52805", feature = "nrf52811"),
    all(feature = "nrf52805", feature = "nrf52820"),
    all(feature = "nrf52805", feature = "nrf52832"),
    all(feature = "nrf52805", feature = "nrf52833"),
    all(feature = "nrf52805", feature = "nrf52840"),
    all(feature = "nrf52810", feature = "nrf52811"),
    all(feature = "nrf52810", feature = "nrf52820"),
    all(feature = "nrf52810", feature = "nrf52832"),
    all(feature = "nrf52810", feature = "nrf52833"),
    all(feature = "nrf52810", feature = "nrf52840"),
    all(feature = "nrf52811", feature = "nrf52820"),
    all(feature = "nrf52811", feature = "nrf52832"),
    all(feature = "nrf52811", feature = "nrf52833"),
    all(feature = "nrf52811", feature = "nrf52840"),
    all(feature = "nrf52820", feature = "nrf52832"),
    all(feature = "nrf52820", feature = "nrf52833"),
    all(feature = "nrf52820", feature = "nrf52840"),
    all(feature = "nrf52832", feature = "nrf52833"),
    all(feature = "nrf52832", feature = "nrf52840"),
    all(feature = "nrf52833", feature = "nrf52840"),
))]
compile_error!("Multile chip features activated. You must activate exactly one of the following features: nrf52810, nrf52811, nrf52832, nrf52833, nrf52840");

// https://www.nordicsemi.com/Software-and-tools/Software/Bluetooth-Software
//
//      | Central  Peripheral  L2CAP-CoC | nrf52805  nrf52810  nrf52811  nrf52820  nrf52832  nrf52833, nrf52840
// -----|--------------------------------|--------------------------------------------------------------------------
// s112 |              X                 |    X         X         X         X         X
// s113 |              X           X     |    X         X         X         X         X         X         X
// s122 |    X                           |                                  X                   X
// s132 |    X         X           X     |              X                             X
// s140 |    X         X           X     |                        X         X                   X         X

#[cfg(not(any(
    all(feature = "nrf52805", feature = "s112"),
    all(feature = "nrf52805", feature = "s113"),
    all(feature = "nrf52810", feature = "s112"),
    all(feature = "nrf52810", feature = "s113"),
    all(feature = "nrf52810", feature = "s132"),
    all(feature = "nrf52811", feature = "s112"),
    all(feature = "nrf52811", feature = "s113"),
    all(feature = "nrf52811", feature = "s140"),
    all(feature = "nrf52820", feature = "s112"),
    all(feature = "nrf52820", feature = "s113"),
    all(feature = "nrf52820", feature = "s122"),
    all(feature = "nrf52820", feature = "s140"),
    all(feature = "nrf52832", feature = "s112"),
    all(feature = "nrf52832", feature = "s113"),
    all(feature = "nrf52832", feature = "s132"),
    all(feature = "nrf52833", feature = "s113"),
    all(feature = "nrf52833", feature = "s122"),
    all(feature = "nrf52833", feature = "s140"),
    all(feature = "nrf52840", feature = "s113"),
    all(feature = "nrf52840", feature = "s140"),
)))]
compile_error!("The selected chip and softdevice are not compatible.");

#[cfg(all(
    feature = "ble-central",
    not(any(feature = "s122", feature = "s132", feature = "s140"))
))]
compile_error!("The selected softdevice does not support ble-central.");

#[cfg(all(
    feature = "ble-peripheral",
    not(any(feature = "s112", feature = "s113", feature = "s132", feature = "s140"))
))]
compile_error!("The selected softdevice does not support ble-peripheral.");

#[cfg(all(
    feature = "ble-l2cap",
    not(any(feature = "s113", feature = "s132", feature = "s140"))
))]
compile_error!("The selected softdevice does not support ble-l2cap.");

#[cfg(feature = "nrf52805")]
use nrf52805_pac as pac;
#[cfg(feature = "nrf52810")]
use nrf52810_pac as pac;
#[cfg(feature = "nrf52811")]
use nrf52811_pac as pac;
#[cfg(feature = "nrf52820")]
use nrf52820_pac as pac;
#[cfg(feature = "nrf52832")]
use nrf52832_pac as pac;
#[cfg(feature = "nrf52833")]
use nrf52833_pac as pac;
#[cfg(feature = "nrf52840")]
use nrf52840_pac as pac;
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

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

#[cfg(feature = "critical-section-impl")]
mod critical_section_impl;

mod events;
pub use events::*;
mod flash;
pub use flash::*;
mod raw_error;
pub use raw_error::*;
pub mod ble;
mod softdevice;
pub use softdevice::*;

mod temperature;
pub use temperature::temperature_celsius;

mod random;
pub use nrf_softdevice_macro::*;
pub use random::random_bytes;
