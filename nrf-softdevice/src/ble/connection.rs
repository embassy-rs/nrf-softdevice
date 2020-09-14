use core::cell::Cell;

use crate::ble::gatt_client;
use crate::ble::types::*;
use crate::raw;
use crate::util::*;
use crate::RawError;

#[derive(defmt::Format)]
pub(crate) struct OutOfConnsError;

#[derive(defmt::Format)]
pub struct DisconnectedError;

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

// This struct is a bit ugly because it has to be usable with const-refs.
// Hence all the cells.
pub(crate) struct ConnectionState {
    // This ConnectionState is "free" if refcount == 0 and conn_handle = none
    //
    // When client code drops all instances, refcount becomes 0 and disconnection is initiated.
    // However, disconnection is not complete until the event GAP_DISCONNECTED.
    // so there's a small gap of time where the ConnectionState is not "free" even if refcount=0.
    pub refcount: Cell<u8>,   // number of existing Connection instances
    pub detached: Cell<bool>, // if true, .detach() has been called, so the conn shouldn't be dropped when refcount reaches 0.
    pub conn_handle: Cell<Option<u16>>, // none = not connected

    pub disconnecting: Cell<bool>,
    pub role: Cell<Role>,

    pub gattc_portal: Portal<gatt_client::PortalMessage>,
}

impl ConnectionState {
    const fn new() -> Self {
        Self {
            refcount: Cell::new(0),
            detached: Cell::new(false),
            conn_handle: Cell::new(None),

            disconnecting: Cell::new(false),
            role: Cell::new(Role::whatever()),
            gattc_portal: Portal::new(),
        }
    }

    fn reset(&self) {
        self.detached.set(false);
        self.disconnecting.set(false);
    }

    pub(crate) fn by_conn_handle(conn_handle: u16) -> &'static Self {
        let index = INDEX_BY_HANDLE.0[conn_handle as usize]
            .get()
            .expect("by_conn_handle on not connected conn_handle");
        &STATE_BY_INDEX.0[index as usize]
    }

    pub(crate) fn check_connected(&self) -> Result<u16, DisconnectedError> {
        match self.conn_handle.get() {
            Some(h) => Ok(h),
            None => Err(DisconnectedError),
        }
    }

    pub(crate) fn disconnect(&self) -> Result<(), DisconnectedError> {
        let conn_handle = self.check_connected()?;

        if self.disconnecting.get() {
            return Ok(());
        }

        let ret = unsafe {
            raw::sd_ble_gap_disconnect(
                conn_handle,
                raw::BLE_HCI_REMOTE_USER_TERMINATED_CONNECTION as u8,
            )
        };
        RawError::convert(ret).dexpect(intern!("sd_ble_gap_disconnect"));

        self.disconnecting.set(true);
        Ok(())
    }

    pub(crate) fn on_disconnected(&self) {
        let conn_handle = self
            .conn_handle
            .get()
            .dexpect(intern!("on_disconnected when already disconnected"));

        let index = INDEX_BY_HANDLE.0[conn_handle as usize]
            .get()
            .dexpect(intern!("conn_handle has no index"));

        trace!("conn {:u8}: disconnected", index,);

        INDEX_BY_HANDLE.0[conn_handle as usize].set(None);
        self.conn_handle.set(None);

        // Signal possible in-progess gattc procedures that the connection
        // has disconnected.
        self.gattc_portal
            .call(gatt_client::PortalMessage::Disconnected);
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

        if new_refcount == 0 {
            if state.detached.get() {
                trace!("conn {:u8}: dropped, but is detached", self.index);
            } else if state.conn_handle.get().is_some() {
                trace!("conn {:u8}: dropped, disconnecting", self.index);
                // We still leave conn_handle set, because the connection is
                // not really disconnected until we get GAP_DISCONNECTED event.
                state.disconnect().dewrap();
            } else {
                trace!("conn {:u8}: dropped, already disconnected", self.index);
            }
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

        Self { index: self.index }
    }
}

impl Connection {
    pub fn disconnect(&self) -> Result<(), DisconnectedError> {
        let state = self.state();
        state.disconnect()
    }

    pub fn detach(&self) {
        let state = self.state();
        state.detached.set(true)
    }

    pub(crate) fn new(conn_handle: u16) -> Result<Self, OutOfConnsError> {
        for (i, s) in STATE_BY_INDEX.0.iter().enumerate() {
            if s.refcount.get() == 0 && s.conn_handle.get().is_none() {
                let index = i as u8;

                // Initialize
                s.refcount.set(1);
                s.conn_handle.set(Some(conn_handle));
                s.reset();

                // Update index_by_handle
                deassert!(
                    INDEX_BY_HANDLE.0[conn_handle as usize].get().is_none(),
                    "conn_handle already has index"
                );
                INDEX_BY_HANDLE.0[conn_handle as usize].set(Some(index));

                trace!("conn {:u8}: connected", index);
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
