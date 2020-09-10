use core::cell::Cell;

use crate::ble::gatt_client;
use crate::error::Error;
use crate::sd;
use crate::util::*;

#[derive(defmt::Format)]
pub(crate) struct OutOfConnsError;

#[derive(defmt::Format)]
pub(crate) struct DisconnectedError;

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
    // This ConnectionState is "free" if refcount == 0 and conn_handle = none
    //
    // When client code drops all instances, refcount becomes 0 and disconnection is initiated.
    // However, disconnection is not complete until the event GAP_DISCONNECTED.
    // so there's a small gap of time where the ConnectionState is not "free" even if refcount=0.
    pub refcount: Cell<u8>, // number of existing Connection instances
    pub conn_handle: Cell<Option<u16>>, // none = not connected

    pub role: Cell<Role>,
    // TODO gattc portals go here instead of being globals.
}

impl ConnectionState {
    const fn new() -> Self {
        Self {
            refcount: Cell::new(0),
            conn_handle: Cell::new(None),
            role: Cell::new(Role::Central),
        }
    }

    pub(crate) fn by_conn_handle(conn_handle: u16) -> &'static Self {
        let index = INDEX_BY_HANDLE.0[conn_handle as usize]
            .get()
            .expect("by_conn_handle on not connected conn_handle");
        &STATE_BY_INDEX.0[index as usize]
    }

    pub(crate) fn on_disconnected(&self) {
        let conn_handle = self
            .conn_handle
            .get()
            .dexpect(intern!("on_disconnected when already disconnected"));

        deassert!(
            INDEX_BY_HANDLE.0[conn_handle as usize].get().is_some(),
            "conn_handle has no index"
        );
        INDEX_BY_HANDLE.0[conn_handle as usize].set(None);
        self.conn_handle.set(None)
    }
}

// Highest ever the softdevice can support.
const CONNS_MAX: usize = 20;

// TODO is this really safe? should be if all the crate's public types are
// non-Send, so client code can only call this crate from the same thread.
struct ForceSync<T>(T);
unsafe impl<T> Sync for ForceSync<T> {}

static STATE_BY_INDEX: ForceSync<[ConnectionState; CONNS_MAX]> =
    ForceSync([ConnectionState::new(); CONNS_MAX]);
static INDEX_BY_HANDLE: ForceSync<[Cell<Option<u8>>; CONNS_MAX]> =
    ForceSync([Cell::new(None); CONNS_MAX]);

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

            // We still leave conn_handle set, because the connection is
            // not really disconnected until we get GAP_DISCONNECTED event.
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

    pub(crate) fn new(conn_handle: u16) -> Result<Self, OutOfConnsError> {
        for (i, s) in STATE_BY_INDEX.0.iter().enumerate() {
            if s.refcount.get() == 0 && s.conn_handle.get().is_none() {
                let index = i as u8;

                // Initialize basic fields
                s.refcount.set(1);
                s.conn_handle.set(Some(conn_handle));

                // Update index_by_handle
                deassert!(
                    INDEX_BY_HANDLE.0[conn_handle as usize].get().is_none(),
                    "conn_handle already has index"
                );
                INDEX_BY_HANDLE.0[conn_handle as usize].set(Some(index));

                trace!("allocated conn index {:u8}, refcount=1", index);
                return Ok(Self { index });
            }
        }
        warn!("no free conn index");
        // TODO disconnect the connection, either here or in calling code.
        Err(OutOfConnsError)
    }

    pub(crate) fn state(&self) -> &ConnectionState {
        &STATE_BY_INDEX.0[self.index as usize]
    }
}
