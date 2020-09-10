#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use core::mem;
use cortex_m_rt::entry;
use defmt::info;

use nrf_softdevice::{raw, Error, Uuid};

#[static_executor::task]
async fn softdevice_task() {
    nrf_softdevice::run().await;
}

const GATT_BAS_SVC_UUID: Uuid = Uuid::new_16(0x180F);
const GATT_BAS_BATTERY_LEVEL_CHAR_UUID: Uuid = Uuid::new_16(0x2A19);

#[static_executor::task]
async fn bluetooth_task() {
    // There'll eventually be a safe API for creating GATT servers.
    // but for now this allows us to test ble_bas_central.

    let mut service_handle: u16 = 0;
    let ret = unsafe {
        raw::sd_ble_gatts_service_add(
            raw::BLE_GATTS_SRVC_TYPE_PRIMARY as u8,
            GATT_BAS_SVC_UUID.as_raw_ptr(),
            &mut service_handle as _,
        )
    };
    Error::convert(ret).dewrap();

    let mut val: u8 = 123;

    let mut cccd_attr_md: raw::ble_gatts_attr_md_t = unsafe { mem::zeroed() };
    cccd_attr_md.read_perm = raw::ble_gap_conn_sec_mode_t {
        _bitfield_1: raw::ble_gap_conn_sec_mode_t::new_bitfield_1(1, 1),
    };
    cccd_attr_md.write_perm = raw::ble_gap_conn_sec_mode_t {
        _bitfield_1: raw::ble_gap_conn_sec_mode_t::new_bitfield_1(1, 1),
    };
    cccd_attr_md.set_vloc(raw::BLE_GATTS_VLOC_STACK as u8);

    let mut attr_md: raw::ble_gatts_attr_md_t = unsafe { mem::zeroed() };
    attr_md.read_perm = raw::ble_gap_conn_sec_mode_t {
        _bitfield_1: raw::ble_gap_conn_sec_mode_t::new_bitfield_1(1, 1),
    };
    attr_md.write_perm = raw::ble_gap_conn_sec_mode_t {
        _bitfield_1: raw::ble_gap_conn_sec_mode_t::new_bitfield_1(1, 1),
    };
    attr_md.set_vloc(raw::BLE_GATTS_VLOC_STACK as u8);

    let mut attr: raw::ble_gatts_attr_t = unsafe { mem::zeroed() };
    attr.p_uuid = unsafe { GATT_BAS_BATTERY_LEVEL_CHAR_UUID.as_raw_ptr() };
    attr.p_attr_md = &attr_md as _;
    attr.init_len = 1;
    attr.max_len = 1;
    attr.p_value = &mut val;

    let mut char_md: raw::ble_gatts_char_md_t = unsafe { mem::zeroed() };
    char_md.char_props.set_read(1);
    char_md.char_props.set_notify(1);
    char_md.p_cccd_md = &mut cccd_attr_md;

    let mut char_handles: raw::ble_gatts_char_handles_t = unsafe { mem::zeroed() };

    let ret = unsafe {
        raw::sd_ble_gatts_characteristic_add(
            service_handle,
            &mut char_md as _,
            &mut attr as _,
            &mut char_handles as _,
        )
    };
    Error::convert(ret).dewrap();

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
        let conn = nrf_softdevice::gap::advertise(
            nrf_softdevice::gap::ConnectableAdvertisement::ScannableUndirected {
                adv_data,
                scan_data,
            },
        )
        .await
        .dewrap();

        info!("advertising done!");
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

    unsafe { nrf_softdevice::enable(&config) }

    unsafe {
        softdevice_task.spawn().dewrap();
        bluetooth_task.spawn().dewrap();

        static_executor::run();
    }
}
