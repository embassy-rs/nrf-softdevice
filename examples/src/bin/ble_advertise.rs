#![no_std]
#![no_main]

#[path = "../example_common.rs"]
mod example_common;

use core::mem;

use defmt::{info, *};
use embassy_executor::Spawner;
use nrf_softdevice::ble::peripheral;
use nrf_softdevice::{raw, Softdevice};

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 256 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t { attr_tab_size: 32768 }),
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
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(raw::BLE_GATTS_VLOC_STACK as u8),
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    unwrap!(spawner.spawn(softdevice_task(sd)));

    #[rustfmt::skip]
    let adv_data = &[
        0x02, raw::BLE_GAP_AD_TYPE_FLAGS as u8, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, raw::BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_COMPLETE as u8, 0x09, 0x18,
        0x0a, raw::BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME as u8, b'H', b'e', b'l', b'l', b'o', b'R', b'u', b's', b't',
    ];
    #[rustfmt::skip]
    let scan_data = &[
        0x03, raw::BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_COMPLETE as u8, 0x09, 0x18,
    ];

    let mut config = peripheral::Config::default();
    config.interval = 50;
    let adv = peripheral::NonconnectableAdvertisement::ScannableUndirected { adv_data, scan_data };
    unwrap!(peripheral::advertise(sd, adv, &config).await);
}
