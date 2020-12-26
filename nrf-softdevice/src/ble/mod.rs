//! Bluetooth Low Energy

mod connection;
mod events;
mod gatt_traits;
mod types;

pub use connection::*;
pub(crate) use events::*;
pub use gatt_traits::*;
pub use types::*;

#[cfg(feature = "ble-central")]
pub mod central;

#[cfg(feature = "ble-peripheral")]
pub mod peripheral;

#[cfg(feature = "ble-gatt-client")]
pub mod gatt_client;

#[cfg(feature = "ble-gatt-server")]
pub mod gatt_server;

#[cfg(feature = "ble-l2cap")]
pub mod l2cap;

use core::mem;

use crate::fmt::*;
use crate::{raw, RawError, Softdevice};

pub fn get_address(_sd: &Softdevice) -> Address {
    unsafe {
        let mut addr: raw::ble_gap_addr_t = mem::zeroed();
        let ret = raw::sd_ble_gap_addr_get(&mut addr);
        unwrap!(RawError::convert(ret), "sd_ble_gap_addr_get");
        Address::from_raw(addr)
    }
}

pub fn set_address(_sd: &Softdevice, addr: &Address) {
    unsafe {
        let addr = addr.into_raw();
        let ret = raw::sd_ble_gap_addr_set(&addr);
        unwrap!(RawError::convert(ret), "sd_ble_gap_addr_set");
    }
}
