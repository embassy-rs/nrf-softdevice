#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use core::{mem, slice};

use defmt::*;
use embassy_executor::Spawner;
use nrf_softdevice::ble::central;
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
            rc_ctiv: 4,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 6,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 128 }),
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

    let config = central::ScanConfig::default();
    let res = central::scan(sd, &config, |params| unsafe {
        info!("AdvReport!");
        info!(
            "type: connectable={} scannable={} directed={} scan_response={} extended_pdu={} status={}",
            params.type_.connectable(),
            params.type_.scannable(),
            params.type_.directed(),
            params.type_.scan_response(),
            params.type_.extended_pdu(),
            params.type_.status()
        );
        info!(
            "addr: resolved={} type={} addr={:x}",
            params.peer_addr.addr_id_peer(),
            params.peer_addr.addr_type(),
            params.peer_addr.addr
        );
        let mut data = slice::from_raw_parts(params.data.p_data, params.data.len as usize);
        while data.len() != 0 {
            let len = data[0] as usize;
            if data.len() < len + 1 {
                warn!("Advertisement data truncated?");
                break;
            }
            if len < 1 {
                warn!("Advertisement data malformed?");
                break;
            }
            let key = data[1];
            let value = &data[2..len + 1];
            info!("value {}: {:x}", key, value);
            data = &data[len + 1..];
        }
        None
    })
    .await;
    unwrap!(res);
    info!("Scan returned");
}
