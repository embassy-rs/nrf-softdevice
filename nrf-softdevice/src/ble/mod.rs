//! Bluetooth Low Energy

mod connection;
pub use connection::*;
mod types;
pub use types::*;
mod events;
pub use events::*;

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
