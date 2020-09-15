#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::mem;
use cortex_m_rt::entry;
use defmt::info;

use nrf_softdevice::ble::gatt_server::{Characteristic, CharacteristicHandles, RegisterError};
use nrf_softdevice::ble::{gatt_server, peripheral, Uuid};
use nrf_softdevice::{raw, RawError, Softdevice};

#[static_executor::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

const GATT_BAS_SVC_UUID: Uuid = Uuid::new_16(0x180F);
const GATT_BAS_BATTERY_LEVEL_CHAR_UUID: Uuid = Uuid::new_16(0x2A19);

struct BatteryServiceServer {
    battery_level_value_handle: u16,
    battery_level_cccd_handle: u16,
}

impl gatt_server::Server for BatteryServiceServer {
    fn uuid() -> Uuid {
        GATT_BAS_SVC_UUID
    }

    fn register<F>(service_handle: u16, mut register_char: F) -> Result<Self, RegisterError>
    where
        F: FnMut(Characteristic, &[u8]) -> Result<CharacteristicHandles, RegisterError>,
    {
        let battery_level = register_char(
            Characteristic {
                uuid: GATT_BAS_BATTERY_LEVEL_CHAR_UUID,
                can_indicate: false,
                can_notify: true,
                can_read: true,
                can_write: true,
                max_len: 1,
            },
            &[123],
        )?;

        Ok(Self {
            battery_level_cccd_handle: battery_level.cccd_handle,
            battery_level_value_handle: battery_level.value_handle,
        })
    }
}

#[static_executor::task]
async fn gatt_server_task(sd: &'static Softdevice, server: BatteryServiceServer) {
    gatt_server::run(sd, &server).await
}

#[static_executor::task]
async fn bluetooth_task(sd: &'static Softdevice) {
    let server: BatteryServiceServer = gatt_server::register(sd).dewrap();
    unsafe { gatt_server_task.spawn(sd, server).dewrap() };

    #[rustfmt::skip]
    let adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x09, 0x18,
        0x0a, 0x09, b'H', b'e', b'l', b'l', b'o', b'R', b'u', b's', b't',
    ];
    #[rustfmt::skip]
    let scan_data = &[
        0x03, 0x03, 0x09, 0x18,
    ];

    loop {
        let conn = peripheral::advertise(
            sd,
            peripheral::ConnectableAdvertisement::ScannableUndirected {
                adv_data,
                scan_data,
            },
        )
        .await
        .dewrap();

        info!("advertising done!");

        // Detach the connection so it isn't disconnected when dropped.
        conn.detach();
    }
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
        bluetooth_task.spawn(sd).dewrap();

        static_executor::run();
    }
}
