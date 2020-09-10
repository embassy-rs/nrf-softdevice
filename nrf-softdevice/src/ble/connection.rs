use core::cell::Cell;

use crate::ble::gatt_client;
use crate::error::Error;
use crate::sd;
use crate::util::*;

#[derive(defmt::Format)]
pub(crate) struct OutOfConnsError;

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

#[derive(defmt::Format, Copy, Clone, Eq, PartialEq)]
pub enum Role {
    Central,
    Peripheral,
}

impl Role {
    pub fn from_raw(raw: u8) -> Self {
        match raw as u32 {
            sd::BLE_GAP_ROLE_CENTRAL => Self::Central,
            sd::BLE_GAP_ROLE_PERIPH => Self::Peripheral,
            _ => depanic!("unknown role {:u8}", raw),
        }
    }
}

// This struct is a bit ugly because it has to be usable with const-refs.
// Hence all the cells.
pub(crate) struct ConnectionState {
    pub(crate) refcount: Cell<u8>,

    /// none = not connected
    pub(crate) conn_handle: Cell<Option<u16>>,

    pub(crate) role: Cell<Role>,
    // TODO gattc portals go here instead of being globals.
}

impl ConnectionState {
    const fn new() -> Self {
        Self {
            refcount: Cell::new(0),
            conn_handle: Cell::new(None),
            role: Cell::new(Role::Peripheral),
        }
    }
}

// Highest ever the softdevice can support.
const CONNS_MAX: usize = 20;

struct ConnectionStates([ConnectionState; CONNS_MAX]);

// TODO is this really safe? should be if all the crate's public types are
// non-Send, so client code can only call this crate from the same thread.
unsafe impl Send for ConnectionStates {}
unsafe impl Sync for ConnectionStates {}

static STATES: ConnectionStates = ConnectionStates([ConnectionState::new(); CONNS_MAX]);

pub struct Connection {
    index: u8,
}

impl Drop for Connection {
    fn drop(&mut self) {
        let state = self.state();
        let new_refcount = state.refcount.get().checked_sub(1).dexpect(intern!(
            "bug: dropping a conn which is already at refcount 0"
        ));
        state.refcount.set(new_refcount);
        trace!(
            "dropped conn index {:u8}, refcount={:u8}",
            self.index,
            new_refcount
        );

        if new_refcount == 0 {
            trace!("all refs dropped, disconnecting");
            let ret = unsafe {
                sd::sd_ble_gap_disconnect(
                    state.conn_handle.get().dewrap(),
                    sd::BLE_HCI_REMOTE_USER_TERMINATED_CONNECTION as u8,
                )
            };
            if let Err(e) = Error::convert(ret) {
                warn!("sd_ble_gap_disconnect err {:?}", e);
            }

            state.conn_handle.set(None)
        }
    }
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        let state = self.state();

        // refcount += 1
        let new_refcount = state
            .refcount
            .get()
            .checked_add(1)
            .dexpect(intern!("Too many references to same connection"));
        state.refcount.set(new_refcount);
        trace!(
            "cloned conn index {:u8}, refcount={:u8}",
            self.index,
            new_refcount
        );

        Self { index: self.index }
    }
}

impl Connection {
    pub async fn discover<T: gatt_client::Client>(&self) -> Result<T, gatt_client::DiscoveryError> {
        let state = self.state();
        // TODO return error if not connected instead of panicking
        let conn_handle = state
            .conn_handle
            .get()
            .dexpect(intern!("Connection is disconnected"));
        gatt_client::discover(conn_handle).await
    }

    pub(crate) fn new() -> Result<Self, OutOfConnsError> {
        for (i, s) in STATES.0.iter().enumerate() {
            if s.refcount.get() == 0 {
                s.refcount.set(1);
                let index = i as u8;
                trace!("allocated conn index {:u8}, refcount=1", index);
                return Ok(Self { index });
            }
        }
        warn!("no free conn index");
        // TODO disconnect the connection, either here or in calling code.
        Err(OutOfConnsError)
    }

    pub(crate) fn state(&self) -> &ConnectionState {
        &STATES.0[self.index as usize]
    }
}
