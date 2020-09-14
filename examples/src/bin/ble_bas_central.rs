#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::mem;
use cortex_m_rt::entry;
use defmt::info;

use nrf_softdevice::ble::{central, gatt_client, Address, Connection, Uuid};
use nrf_softdevice::raw;
use nrf_softdevice::Softdevice;

#[static_executor::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

struct BatteryServiceClient {
    conn: Connection,
    battery_level_value_handle: u16,
    battery_level_cccd_handle: u16,
}

const GATT_BAS_SVC_UUID: Uuid = Uuid::new_16(0x180F);
const GATT_BAS_BATTERY_LEVEL_CHAR_UUID: Uuid = Uuid::new_16(0x2A19);

// This is mostly boilerplate, ideally it'll be generated with a proc macro in the future.
impl gatt_client::Client for BatteryServiceClient {
    fn uuid() -> Uuid {
        return GATT_BAS_SVC_UUID;
    }

    fn new_undiscovered(conn: Connection) -> Self {
        Self {
            conn,
            battery_level_value_handle: 0,
            battery_level_cccd_handle: 0,
        }
    }

    fn discovered_characteristic(
        &mut self,
        characteristic: &gatt_client::Characteristic,
        descriptors: &[gatt_client::Descriptor],
    ) {
        if let Some(char_uuid) = characteristic.uuid {
            if char_uuid == GATT_BAS_BATTERY_LEVEL_CHAR_UUID {
                // TODO maybe check the char_props have the necessary operations allowed? read/write/notify/etc
                self.battery_level_value_handle = characteristic.handle_value;
                for desc in descriptors {
                    if let Some(desc_uuid) = desc.uuid {
                        if desc_uuid
                            == Uuid::new_16(raw::BLE_UUID_DESCRIPTOR_CLIENT_CHAR_CONFIG as u16)
                        {
                            self.battery_level_cccd_handle = desc.handle;
                        }
                    }
                }
            }
        }
    }

    fn discovery_complete(&mut self) -> Result<(), gatt_client::DiscoverError> {
        if self.battery_level_cccd_handle == 0 || self.battery_level_value_handle == 0 {
            return Err(gatt_client::DiscoverError::ServiceIncomplete);
        }
        Ok(())
    }
}

#[static_executor::task]
async fn ble_central_task(sd: &'static Softdevice) {
    let addrs = &[Address::new_random_static([
        0x59, 0xf9, 0xb1, 0x9c, 0x01, 0xf5,
    ])];

    let conn = central::connect(sd, addrs)
        .await
        .dexpect(intern!("connect"));
    info!("connected");

    let client: BatteryServiceClient = gatt_client::discover(&conn)
        .await
        .dexpect(intern!("discover"));

    info!(
        "discovered! {:u16} {:u16}",
        client.battery_level_value_handle, client.battery_level_cccd_handle
    );

    let buf = &mut [0; 16];
    let len = gatt_client::read(&conn, client.battery_level_value_handle, buf)
        .await
        .dexpect(intern!("read"));

    info!("read battery level: {:[u8]}", &buf[..len]);
}

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_XTAL as u8,
            rc_ctiv: 0,
            rc_temp_ctiv: 0,
            accuracy: 7,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 6,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 128 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 32768,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,
            central_role_count: 3,
            central_sec_count: 0,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: b"HelloRust" as *const u8 as _,
            current_len: 9,
            max_len: 9,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);

    unsafe {
        softdevice_task.spawn(sd).dewrap();
        ble_central_task.spawn(sd).dewrap();

        static_executor::run();
    }
}
