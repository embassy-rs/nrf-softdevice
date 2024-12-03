#![no_std]
#![no_main]

#[path = "../example_common.rs"]
mod example_common;

use core::cell::{Cell, RefCell};
use core::mem;

use defmt::{info, *};
use embassy_executor::Spawner;
use embassy_nrf::interrupt;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{with_timeout, Duration, Timer};
use nrf_softdevice::ble::security::{IoCapabilities, SecurityHandler};
use nrf_softdevice::ble::{
    central, gatt_client, Address, AddressType, Connection, EncryptError, EncryptionInfo, IdentityKey, MasterId,
    SecurityMode,
};
use nrf_softdevice::{raw, Softdevice};
use static_cell::StaticCell;

const PERIPHERAL_REQUESTS_SECURITY: bool = false;

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[derive(Debug, Clone, Copy)]
struct Peer {
    master_id: MasterId,
    key: EncryptionInfo,
    peer_id: IdentityKey,
}

pub struct Bonder {
    peer: Cell<Option<Peer>>,
    sys_attrs: RefCell<heapless::Vec<u8, 62>>,
    secured: Signal<ThreadModeRawMutex, bool>,
}

impl Default for Bonder {
    fn default() -> Self {
        Bonder {
            peer: Cell::new(None),
            sys_attrs: Default::default(),
            secured: Signal::new(),
        }
    }
}

impl SecurityHandler for Bonder {
    fn io_capabilities(&self) -> IoCapabilities {
        IoCapabilities::DisplayOnly
    }

    fn can_bond(&self, _conn: &Connection) -> bool {
        true
    }

    fn display_passkey(&self, passkey: &[u8; 6]) {
        info!("The passkey is \"{:a}\"", passkey)
    }

    fn on_bonded(&self, _conn: &Connection, master_id: MasterId, key: EncryptionInfo, peer_id: IdentityKey) {
        debug!("storing bond for: id: {}, key: {}", master_id, key);

        // In a real application you would want to signal another task to permanently store the keys in non-volatile memory here.
        self.sys_attrs.borrow_mut().clear();
        self.peer.set(Some(Peer {
            master_id,
            key,
            peer_id,
        }));
    }

    fn on_security_update(&self, _conn: &Connection, security_mode: SecurityMode) {
        match security_mode {
            SecurityMode::NoAccess | SecurityMode::Open => self.secured.signal(false),
            _ => self.secured.signal(true),
        }
    }

    fn save_sys_attrs(&self, _conn: &Connection) {
        self.secured.signal(false);
    }

    fn get_key(&self, _conn: &Connection, master_id: MasterId) -> Option<EncryptionInfo> {
        debug!("getting bond for: id: {}", master_id);

        self.peer
            .get()
            .and_then(|peer| (master_id == peer.master_id).then_some(peer.key))
    }

    fn get_peripheral_key(&self, conn: &Connection) -> Option<(MasterId, EncryptionInfo)> {
        self.peer.get().and_then(|peer| {
            peer.peer_id
                .is_match(conn.peer_address())
                .then_some((peer.master_id, peer.key))
        })
    }
}

#[nrf_softdevice::gatt_client(uuid = "180f")]
struct BatteryServiceClient {
    #[characteristic(uuid = "2a19", read, write, notify)]
    battery_level: u8,
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    // First we get the peripherals access crate.
    let mut config = embassy_nrf::config::Config::default();
    config.gpiote_interrupt_priority = interrupt::Priority::P2;
    config.time_interrupt_priority = interrupt::Priority::P2;
    let _p = embassy_nrf::init(config);

    let config = nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 6,
            event_length: 6,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 128 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 3,
            central_role_count: 3,
            central_sec_count: 1,
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

    static BONDER: StaticCell<Bonder> = StaticCell::new();
    let bonder = BONDER.init(Bonder::default());

    loop {
        let addrs = &[&Address::new(
            AddressType::RandomStatic,
            [0x99, 0xbc, 0x25, 0x5d, 0x1e, 0xf7],
        )];
        let mut config = central::ConnectConfig::default();
        config.scan_config.whitelist = Some(addrs);
        info!("scanning");
        bonder.secured.reset();
        let conn = unwrap!(central::connect_with_security(sd, &config, bonder).await);
        info!("connected");

        info!("encrypting connection");
        let secured = if PERIPHERAL_REQUESTS_SECURITY {
            bonder.secured.wait().await
        } else {
            match conn.encrypt() {
                Ok(()) => {
                    if bonder.secured.wait().await {
                        true
                    } else {
                        warn!("failed to encrypt connection with stored keys, requesting pairing");
                        if let Err(err) = conn.request_pairing() {
                            error!("failed to initiate pairing: {:?}", err);
                            continue;
                        }
                        bonder.secured.wait().await
                    }
                }
                Err(EncryptError::PeerKeysNotFound) => {
                    info!("peer keys not found, requesting pairing");
                    if let Err(err) = conn.request_pairing() {
                        error!("failed to initiate pairing: {:?}", err);
                        continue;
                    }
                    bonder.secured.wait().await
                }
                Err(err) => {
                    error!("failed to initiate encryption: {:?}", err);
                    continue;
                }
            }
        };

        if !secured {
            error!("failed to create secure connection");
            continue;
        }

        let client: BatteryServiceClient = unwrap!(gatt_client::discover(&conn).await);

        // Read
        let val = unwrap!(client.battery_level_read().await);
        info!("read battery level: {}", val);

        // Write, set it to 42
        unwrap!(client.battery_level_write(&42).await);
        info!("Wrote battery level!");

        // Read to check it's changed
        let val = unwrap!(client.battery_level_read().await);
        info!("read battery level: {}", val);

        // Enable battery level notifications from the peripheral
        client.battery_level_cccd_write(true).await.unwrap();

        // Receive notifications

        let _ = with_timeout(
            Duration::from_secs(30),
            gatt_client::run(&conn, &client, |event| match event {
                BatteryServiceClientEvent::BatteryLevelNotification(val) => {
                    info!("battery level notification: {}", val);
                }
            }),
        )
        .await;
        let _ = conn.disconnect();

        info!("Disconnected, waiting before reconnecting");
        Timer::after_secs(10).await;
    }
}
