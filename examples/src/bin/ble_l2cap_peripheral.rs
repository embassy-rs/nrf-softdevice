#![no_std]
#![no_main]

#[path = "../example_common.rs"]
mod example_common;

use core::mem;
use core::ptr::NonNull;

use defmt::*;
use embassy_executor::Spawner;
use nrf_softdevice::ble::{l2cap, peripheral};
use nrf_softdevice::{ble, raw, RawError, Softdevice};

const PSM: u16 = 0x2349;

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

use atomic_pool::{pool, Box};

pool!(PacketPool: [[u8; 512]; 10]);

struct Packet {
    len: usize,
    buf: Box<PacketPool>,
}

impl Packet {
    fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl l2cap::Packet for Packet {
    const MTU: usize = 512;
    fn allocate() -> Option<NonNull<u8>> {
        if let Some(buf) = Box::<PacketPool>::new([0; 512]) {
            let ptr = Box::into_raw(buf).cast::<u8>();
            info!("allocate {}", ptr.as_ptr() as u32);
            Some(ptr)
        } else {
            None
        }
    }

    fn into_raw_parts(self) -> (NonNull<u8>, usize) {
        let ptr = Box::into_raw(self.buf).cast::<u8>();
        let len = self.len;
        info!("into_raw_parts {}", ptr.as_ptr() as u32);
        (ptr, len)
    }

    unsafe fn from_raw_parts(ptr: NonNull<u8>, len: usize) -> Self {
        info!("from_raw_parts {}", ptr.as_ptr() as u32);
        Self {
            len,
            buf: Box::from_raw(ptr.cast::<[u8; 512]>()),
        }
    }
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
            conn_count: 20,
            event_length: 180,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 114 }),
        conn_gattc: Some(raw::ble_gattc_conn_cfg_t {
            write_cmd_tx_queue_size: 0,
        }),
        conn_gatts: Some(raw::ble_gatts_conn_cfg_t { hvn_tx_queue_size: 0 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT,
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
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(raw::BLE_GATTS_VLOC_STACK as u8),
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

    let sd = Softdevice::enable(&config);
    unwrap!(RawError::convert(unsafe { raw::sd_clock_hfclk_request() }));
    unwrap!(spawner.spawn(softdevice_task(sd)));

    info!("My address: {:?}", ble::get_address(sd));

    #[rustfmt::skip]
    let adv_data = &[
        0x02, 0x01, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x11, raw::BLE_GAP_AD_TYPE_128BIT_SERVICE_UUID_MORE_AVAILABLE as u8,
            // The 128bit UUID shared with l2cap_central.rs
            0xeb, 0x04, 0x8b, 0xfd, 0x5b, 0x03, 0x21, 0xb5,
            0xeb, 0x11, 0x65, 0x2f, 0x18, 0xce, 0x9c, 0x82,
        0x02, raw::BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME as u8,
            b'H',
    ];
    #[rustfmt::skip]
    let scan_data = &[ ];

    let l = l2cap::L2cap::<Packet>::init(sd);

    loop {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected { adv_data, scan_data };
        let conn = unwrap!(peripheral::advertise_connectable(sd, adv, &config).await);

        info!("advertising done!");

        let config = l2cap::Config { credits: 8 };
        let ch = unwrap!(l.listen(&conn, &config, PSM).await);
        info!("l2cap connected");

        loop {
            let pkt = unwrap!(ch.rx().await);
            info!("rx: {:x}", pkt.as_bytes());
        }
    }
}
