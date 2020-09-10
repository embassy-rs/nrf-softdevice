#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::mem;
use cortex_m_rt::entry;
use defmt::info;

use nrf_softdevice::{gap, gatt_client, uuid::Uuid};
use nrf_softdevice_s140 as sd;

#[static_executor::task]
async fn softdevice_task() {
    nrf_softdevice::run().await;
}

struct BatteryServiceClient {
    battery_level_value_handle: u16,
    battery_level_cccd_handle: u16,
}

const GATT_BAS_SVC_UUID: Uuid = Uuid::new_16(0x180F);
const GATT_BAS_BATTERY_LEVEL_CHAR_UUID: Uuid = Uuid::new_16(0x2A19);

impl gatt_client::Client for BatteryServiceClient {
    fn uuid() -> Uuid {
        return GATT_BAS_SVC_UUID;
    }

    fn new_undiscovered() -> Self {
        Self {
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
                            == Uuid::new_16(sd::BLE_UUID_DESCRIPTOR_CLIENT_CHAR_CONFIG as u16)
                        {
                            self.battery_level_cccd_handle = desc.handle;
                        }
                    }
                }
            }
        }
    }
    fn discovery_complete(&mut self) -> Result<(), gatt_client::DiscoveryError> {
        if self.battery_level_cccd_handle == 0 || self.battery_level_value_handle == 0 {
            return Err(gatt_client::DiscoveryError::ServiceIncomplete);
        }
        Ok(())
    }
}

#[static_executor::task]
async fn ble_central_task() {
    let addrs = &[gap::Address::new_random_static([
        0x59, 0xf9, 0xb1, 0x9c, 0x01, 0xf5,
    ])];

    let conn = gap::connect(addrs).await.dewrap();
    info!("connected");

    let svc: BatteryServiceClient = conn.discover().await.dewrap();
    info!(
        "discovered! {:u16} {:u16}",
        svc.battery_level_value_handle, svc.battery_level_cccd_handle
    );
}

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let config = nrf_softdevice::Config {
        clock: Some(sd::nrf_clock_lf_cfg_t {
            source: sd::NRF_CLOCK_LF_SRC_XTAL as u8,
            rc_ctiv: 0,
            rc_temp_ctiv: 0,
            accuracy: 7,
        }),
        conn_gap: Some(sd::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 6,
        }),
        conn_gatt: Some(sd::ble_gatt_conn_cfg_t { att_mtu: 128 }),
        gatts_attr_tab_size: Some(sd::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 32768,
        }),
        gap_role_count: Some(sd::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,
            central_role_count: 3,
            central_sec_count: 0,
            _bitfield_1: sd::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(sd::ble_gap_cfg_device_name_t {
            p_value: b"HelloRust" as *const u8 as _,
            current_len: 9,
            max_len: 9,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: sd::ble_gap_cfg_device_name_t::new_bitfield_1(
                sd::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };

    unsafe { nrf_softdevice::enable(&config) }

    unsafe {
        softdevice_task.spawn().unwrap();
        //interrupt_task.spawn().unwrap();
        //flash_task.spawn().unwrap();
        //bluetooth_task.spawn().unwrap();
        ble_central_task.spawn().unwrap();

        static_executor::run();
    }
}
