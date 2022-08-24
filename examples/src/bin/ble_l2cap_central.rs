#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use core::ptr::NonNull;
use core::{mem, slice};

use defmt::{info, *};
use embassy_executor::Spawner;
use nrf_softdevice::ble::l2cap::Packet as _;
use nrf_softdevice::ble::{central, l2cap, Address, TxPower};
use nrf_softdevice::{raw, Softdevice};

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

impl Format for Packet {
    fn format(&self, fmt: Formatter) {
        defmt::write!(fmt, "Packet({:x})", &self.buf[..self.len])
    }
}

impl Packet {
    fn new(data: &[u8]) -> Self {
        let mut buf = unwrap!(Box::<PacketPool>::new([0; 512]));
        buf[..data.len()].copy_from_slice(data);
        Packet { len: data.len(), buf }
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
        conn_gatts: Some(raw::ble_gatts_conn_cfg_t { hvn_tx_queue_size: 0 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t { attr_tab_size: 512 }),
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
            rx_mps: 247,
            tx_mps: 247,
            rx_queue_size: 10,
            tx_queue_size: 10,
        }),
        ..Default::default()
    };

    let sd = Softdevice::enable(&config);
    unwrap!(spawner.spawn(softdevice_task(sd)));

    info!("Scanning for peer...");

    let config = central::ScanConfig {
        whitelist: None,
        tx_power: TxPower::ZerodBm,
        ..Default::default()
    };
    let res = central::scan(sd, &config, |params| unsafe {
        let mut data = slice::from_raw_parts(params.data.p_data, params.data.len as usize);
        while data.len() != 0 {
            let len = data[0] as usize;
            if data.len() < len + 1 {
                break;
            }
            if len < 1 {
                break;
            }
            let key = data[1];
            let value = &data[2..len + 1];

            if key == 0x06
                && value
                    == &[
                        0xeb, 0x04, 0x8b, 0xfd, 0x5b, 0x03, 0x21, 0xb5, 0xeb, 0x11, 0x65, 0x2f, 0x18, 0xce, 0x9c, 0x82,
                    ]
            {
                return Some(Address::from_raw(params.peer_addr));
            }
            data = &data[len + 1..];
        }
        None
    })
    .await;
    let address = unwrap!(res);
    info!("Scan found address {:?}", address);

    let addrs = &[&address];

    let mut config = central::ConnectConfig::default();
    config.scan_config.whitelist = Some(addrs);
    let conn = unwrap!(central::connect(sd, &config).await);
    info!("connected");

    let l = l2cap::L2cap::<Packet>::init(sd);
    let config = l2cap::Config { credits: 8 };
    let ch = unwrap!(l.setup(&conn, &config, PSM).await);
    info!("l2cap connected");

    for i in 0..10 {
        unwrap!(ch.tx(Packet::new(&[i; Packet::MTU])).await);
        info!("l2cap tx done");
    }
    futures::future::pending::<()>().await;
}
