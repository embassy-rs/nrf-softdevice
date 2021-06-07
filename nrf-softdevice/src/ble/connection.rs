use core::cell::Cell;
use core::cell::UnsafeCell;

use crate::ble::types::*;
use crate::ble::*;
use crate::raw;
use crate::RawError;

const BLE_GAP_DATA_LENGTH_DEFAULT: u8 = 27; //  The stack's default data length. <27-251>

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub(crate) struct OutOfConnsError;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DisconnectedError;

// Highest ever the softdevice can support.
pub(crate) const CONNS_MAX: usize = 20;

// We could make the public Connection type simply hold the softdevice's conn_handle.
// However, that would allow for bugs like:
// - Connection is established with conn_handle=5
// - Client code stores a Connection instance with conn_handle=5
// - Connection gets disconnected
// - A new, unrelated connection is established with same conn_handle=5
// - Client code uses the Connection instance from before, mistakenly doing an operation in an unrelated connection.
//
// To avoid this, the public Connection struct has an "index" into a private ConnectionState array.
// It is refcounted, so an index will never be reused until client code has dropped all Connection instances.

pub(crate) struct ConnectionState {
    // Every Connection instance counts as one ref.
    //
    // When client code drops all instances, refcount becomes 0 and disconnection is initiated.
    // However, disconnection is not complete until the event GAP_DISCONNECTED.
    // so there's a small gap of time where the ConnectionState is not "free" even if refcount=0.
    pub refcount: u8,
    pub conn_handle: Option<u16>,

    pub disconnecting: bool,
    pub role: Role,
    pub peer_address: Address,

    pub att_mtu: u16, // Effective ATT_MTU size (in bytes).
    #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
    pub data_length_effective: u8, // Effective data length (in bytes).
}

impl ConnectionState {
    const fn dummy() -> Self {
        // The returned value should have bit pattern 0, so that STATES
        // can go into .bss instead of .data, which saves flash space.
        Self {
            refcount: 0,
            conn_handle: None,
            #[cfg(feature = "ble-central")]
            role: Role::Central,
            #[cfg(not(feature = "ble-central"))]
            role: Role::Peripheral,
            peer_address: Address::new(AddressType::Public, [0; 6]),
            disconnecting: false,
            att_mtu: 0,
            #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
            data_length_effective: 0,
        }
    }
    pub(crate) fn check_connected(&mut self) -> Result<u16, DisconnectedError> {
        self.conn_handle.ok_or(DisconnectedError)
    }

    pub(crate) fn disconnect(&mut self) -> Result<(), DisconnectedError> {
        let conn_handle = self.check_connected()?;

        if self.disconnecting {
            return Ok(());
        }

        let ret = unsafe {
            raw::sd_ble_gap_disconnect(
                conn_handle,
                raw::BLE_HCI_REMOTE_USER_TERMINATED_CONNECTION as u8,
            )
        };
        unwrap!(RawError::convert(ret), "sd_ble_gap_disconnect");

        self.disconnecting = true;
        Ok(())
    }

    pub(crate) fn on_disconnected(&mut self, ble_evt: *const raw::ble_evt_t) {
        let conn_handle = unwrap!(
            self.conn_handle,
            "bug: on_disconnected when already disconnected"
        );

        let ibh = index_by_handle(conn_handle);
        let _index = unwrap!(ibh.get(), "bug: conn_handle has no index");
        ibh.set(None);

        self.conn_handle = None;

        // Signal possible in-progess operations that the connection has disconnected.
        #[cfg(feature = "ble-gatt-client")]
        gatt_client::portal(conn_handle).call(ble_evt);
        #[cfg(feature = "ble-gatt-server")]
        gatt_server::portal(conn_handle).call(ble_evt);
        #[cfg(feature = "ble-l2cap")]
        l2cap::portal(conn_handle).call(ble_evt);

        trace!("conn {:?}: disconnected", _index);
    }
}

pub struct Connection {
    index: u8,
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.with_state(|state| {
            state.refcount = unwrap!(
                state.refcount.checked_sub(1),
                "bug: dropping a conn which is already at refcount 0"
            );

            if state.refcount == 0 {
                if state.conn_handle.is_some() {
                    trace!("conn {:?}: dropped, disconnecting", self.index);
                    // We still leave conn_handle set, because the connection is
                    // not really disconnected until we get GAP_DISCONNECTED event.
                    unwrap!(state.disconnect());
                } else {
                    trace!("conn {:?}: dropped, already disconnected", self.index);
                }
            }
        });
    }
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        self.with_state(|state| {
            state.refcount = unwrap!(
                state.refcount.checked_add(1),
                "Too many references to same connection"
            );
        });

        Self { index: self.index }
    }
}

impl Connection {
    pub fn role(&self) -> Role {
        self.with_state(|state| state.role)
    }

    pub fn peer_address(&self) -> Address {
        self.with_state(|state| state.peer_address)
    }

    pub fn disconnect(&self) -> Result<(), DisconnectedError> {
        self.with_state(|state| state.disconnect())
    }

    pub fn handle(&self) -> Option<u16> {
        self.with_state(|state| state.conn_handle)
    }

    pub(crate) fn new(
        conn_handle: u16,
        role: Role,
        peer_address: Address,
    ) -> Result<Self, OutOfConnsError> {
        allocate_index(|index, state| {
            // Initialize
            *state = ConnectionState {
                refcount: 1,
                conn_handle: Some(conn_handle),
                role,
                peer_address,

                disconnecting: false,

                att_mtu: raw::BLE_GATT_ATT_MTU_DEFAULT as _,

                #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
                data_length_effective: BLE_GAP_DATA_LENGTH_DEFAULT,
            };

            // Update index_by_handle
            let ibh = index_by_handle(conn_handle);
            assert!(ibh.get().is_none(), "bug: conn_handle already has index");
            ibh.set(Some(index));

            trace!("conn {:?}: connected", index);
            Self { index }
        })
    }

    pub(crate) fn with_state<T>(&self, f: impl FnOnce(&mut ConnectionState) -> T) -> T {
        with_state(self.index, f)
    }
}

// ConnectionStates by index.
const DUMMY_STATE: UnsafeCell<ConnectionState> = UnsafeCell::new(ConnectionState::dummy());
static mut STATES: [UnsafeCell<ConnectionState>; CONNS_MAX] = [DUMMY_STATE; CONNS_MAX];

pub(crate) fn with_state_by_conn_handle<T>(
    conn_handle: u16,
    f: impl FnOnce(&mut ConnectionState) -> T,
) -> T {
    let index = unwrap!(
        index_by_handle(conn_handle).get(),
        "bug: with_state_by_conn_handle on conn_handle that has no state"
    );
    with_state(index, f)
}

pub(crate) fn with_state<T>(index: u8, f: impl FnOnce(&mut ConnectionState) -> T) -> T {
    let state = unsafe { &mut *STATES[index as usize].get() };
    f(state)
}

fn allocate_index<T>(f: impl FnOnce(u8, &mut ConnectionState) -> T) -> Result<T, OutOfConnsError> {
    unsafe {
        for (i, s) in STATES.iter().enumerate() {
            let state = &mut *s.get();
            if state.refcount == 0 && state.conn_handle.is_none() {
                return Ok(f(i as u8, state));
            }
        }
        Err(OutOfConnsError)
    }
}

// conn_handle -> index mapping. Used to make stuff go faster
const INDEX_NONE: Cell<Option<u8>> = Cell::new(None);
static mut INDEX_BY_HANDLE: [Cell<Option<u8>>; CONNS_MAX] = [INDEX_NONE; CONNS_MAX];

fn index_by_handle(conn_handle: u16) -> &'static Cell<Option<u8>> {
    unsafe { &INDEX_BY_HANDLE[conn_handle as usize] }
}
