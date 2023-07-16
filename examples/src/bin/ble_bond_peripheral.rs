#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use core::cell::{Cell, RefCell};
use core::mem;

use defmt::{info, *};
use embassy_executor::Spawner;
use nrf_softdevice::ble::gatt_server::builder::ServiceBuilder;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};
use nrf_softdevice::ble::gatt_server::{set_sys_attrs, RegisterError, WriteOp};
use nrf_softdevice::ble::security::{IoCapabilities, SecurityHandler};
use nrf_softdevice::ble::{
    gatt_server, peripheral, Connection, EncryptionInfo, IdentityKey, MasterId, SecurityMode, Uuid,
};
use nrf_softdevice::{raw, Softdevice};
use static_cell::StaticCell;

const BATTERY_SERVICE: Uuid = Uuid::new_16(0x180f);
const BATTERY_LEVEL: Uuid = Uuid::new_16(0x2a19);

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
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
}

impl Default for Bonder {
    fn default() -> Self {
        Bonder {
            peer: Cell::new(None),
            sys_attrs: Default::default(),
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

    fn get_key(&self, _conn: &Connection, master_id: MasterId) -> Option<EncryptionInfo> {
        debug!("getting bond for: id: {}", master_id);

        self.peer
            .get()
            .and_then(|peer| (master_id == peer.master_id).then_some(peer.key))
    }

    fn save_sys_attrs(&self, conn: &Connection) {
        debug!("saving system attributes for: {}", conn.peer_address());

        if let Some(peer) = self.peer.get() {
            if peer.peer_id.is_match(conn.peer_address()) {
                let mut sys_attrs = self.sys_attrs.borrow_mut();
                let capacity = sys_attrs.capacity();
                unwrap!(sys_attrs.resize(capacity, 0));
                let len = unwrap!(gatt_server::get_sys_attrs(conn, &mut sys_attrs)) as u16;
                sys_attrs.truncate(usize::from(len));
                // In a real application you would want to signal another task to permanently store sys_attrs for this connection's peer
            }
        }
    }

    fn load_sys_attrs(&self, conn: &Connection) {
        let addr = conn.peer_address();
        debug!("loading system attributes for: {}", addr);

        let attrs = self.sys_attrs.borrow();
        // In a real application you would search all stored peers to find a match
        let attrs = if self.peer.get().map(|peer| peer.peer_id.is_match(addr)).unwrap_or(false) {
            (!attrs.is_empty()).then_some(attrs.as_slice())
        } else {
            None
        };

        unwrap!(set_sys_attrs(conn, attrs));
    }
}

pub struct BatteryService {
    value_handle: u16,
    cccd_handle: u16,
}

impl BatteryService {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let mut service_builder = ServiceBuilder::new(sd, BATTERY_SERVICE)?;

        let attr = Attribute::new(&[0u8]).security(SecurityMode::JustWorks);
        let metadata = Metadata::new(Properties::new().read().notify());
        let characteristic_builder = service_builder.add_characteristic(BATTERY_LEVEL, attr, metadata)?;
        let characteristic_handles = characteristic_builder.build();

        let _service_handle = service_builder.build();

        Ok(BatteryService {
            value_handle: characteristic_handles.value_handle,
            cccd_handle: characteristic_handles.cccd_handle,
        })
    }

    pub fn battery_level_get(&self, sd: &Softdevice) -> Result<u8, gatt_server::GetValueError> {
        let buf = &mut [0u8];
        gatt_server::get_value(sd, self.value_handle, buf)?;
        Ok(buf[0])
    }

    pub fn battery_level_set(&self, sd: &Softdevice, val: u8) -> Result<(), gatt_server::SetValueError> {
        gatt_server::set_value(sd, self.value_handle, &[val])
    }
    pub fn battery_level_notify(&self, conn: &Connection, val: u8) -> Result<(), gatt_server::NotifyValueError> {
        gatt_server::notify_value(conn, self.value_handle, &[val])
    }

    pub fn on_write(&self, handle: u16, data: &[u8]) {
        if handle == self.cccd_handle && !data.is_empty() {
            info!("battery notifications: {}", (data[0] & 0x01) != 0);
        }
    }
}

struct Server {
    bas: BatteryService,
}

impl Server {
    pub fn new(sd: &mut Softdevice) -> Result<Self, RegisterError> {
        let bas = BatteryService::new(sd)?;

        Ok(Self { bas })
    }
}

impl gatt_server::Server for Server {
    type Event = ();

    fn on_write(
        &self,
        _conn: &Connection,
        handle: u16,
        _op: WriteOp,
        _offset: usize,
        data: &[u8],
    ) -> Option<Self::Event> {
        self.bas.on_write(handle, data);
        None
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
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
    let server = unwrap!(Server::new(sd));
    unwrap!(spawner.spawn(softdevice_task(sd)));

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

    static BONDER: StaticCell<Bonder> = StaticCell::new();
    let bonder = BONDER.init(Bonder::default());

    loop {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected { adv_data, scan_data };
        let conn = unwrap!(peripheral::advertise_pairable(sd, adv, &config, bonder).await);

        info!("advertising done!");

        // Run the GATT server on the connection. This returns when the connection gets disconnected.
        let e = gatt_server::run(&conn, &server, |_| {}).await;

        info!("gatt_server run exited with error: {:?}", e);
    }
}
