#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(alloc_error_handler)]
#![allow(incomplete_features)]
extern crate alloc;

#[path = "../example_common.rs"]
mod example_common;

use core::mem;
use core::ptr::NonNull;
use cortex_m_rt::entry;
use defmt::*;
use embassy::executor::Executor;
use embassy::util::Forever;

use nrf_softdevice::ble::{l2cap, peripheral};
use nrf_softdevice::{ble, RawError};
use nrf_softdevice::{raw, Softdevice};

static EXECUTOR: Forever<Executor> = Forever::new();

const PSM: u16 = 0x2349;

#[embassy::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

#[embassy::task]
async fn bluetooth_task(sd: &'static Softdevice) {
    info!("My address: {:?}", ble::get_address(sd));

    #[rustfmt::skip]
    let adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x11, 0x06, 0xeb, 0x04, 0x8b, 0xfd, 0x5b, 0x03, 0x21, 0xb5, 0xeb, 0x11, 0x65, 0x2f, 0x18, 0xce, 0x9c, 0x82,
        0x02, 0x09, b'H',
    ];
    #[rustfmt::skip]
    let scan_data = &[ ];

    let l = l2cap::L2cap::<Packet>::init(sd);

    loop {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data,
            scan_data,
        };
        let conn = unwrap!(peripheral::advertise_connectable(sd, adv, &config).await);

        info!("advertising done!");

        let config = l2cap::Config {
            psm: PSM,
            credits: 8,
        };
        let ch = unwrap!(l.listen(&conn, &config).await);
        info!("l2cap connected");

        loop {
            let pkt = unwrap!(ch.rx().await);
            info!("rx: {:x}", pkt.0);
        }
    }
}

use alloc::vec::Vec;

struct Packet(Vec<u8>);
impl l2cap::Packet for Packet {
    const MTU: usize = 512;
    fn allocate() -> Option<NonNull<u8>> {
        let mut v = Vec::with_capacity(Self::MTU);
        let ptr = v.as_mut_ptr();
        mem::forget(v);
        info!("allocate {}", ptr as u32);
        NonNull::new(ptr)
    }

    fn into_raw_parts(mut self) -> (NonNull<u8>, usize) {
        let ptr = self.0.as_mut_ptr();
        let len = self.0.len();
        mem::forget(self);
        info!("into_raw_parts {}", ptr as u32);
        (unwrap!(NonNull::new(ptr)), len)
    }

    unsafe fn from_raw_parts(ptr: NonNull<u8>, len: usize) -> Self {
        info!("from_raw_parts {}", ptr.as_ptr() as u32);
        Self(Vec::from_raw_parts(ptr.as_ptr(), len, Self::MTU))
    }
}

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 4,
            rc_temp_ctiv: 2,
            accuracy: 7,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 20,
            event_length: 180,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 114 }),
        conn_gattc: Some(raw::ble_gattc_conn_cfg_t {
            write_cmd_tx_queue_size: 0,
        }),
        conn_gatts: Some(raw::ble_gatts_conn_cfg_t {
            hvn_tx_queue_size: 0,
        }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: 1024,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 5,
            central_role_count: 15,
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
        conn_l2cap: Some(raw::ble_l2cap_conn_cfg_t {
            ch_count: 1,
            rx_mps: 256,
            tx_mps: 256,
            rx_queue_size: 10,
            tx_queue_size: 10,
        }),
        ..Default::default()
    };

    unwrap!(RawError::convert(unsafe { raw::sd_clock_hfclk_request() }));

    let sd = Softdevice::enable(&config);

    let executor = EXECUTOR.put(Executor::new());
    executor.run(|spawner| {
        unwrap!(spawner.spawn(softdevice_task(sd)));
        unwrap!(spawner.spawn(bluetooth_task(sd)));
    });
}
