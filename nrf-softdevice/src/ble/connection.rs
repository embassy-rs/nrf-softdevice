use core::cell::Cell;
use core::cell::UnsafeCell;

#[cfg(feature = "ble-gatt-client")]
use crate::ble::gatt_client;
#[cfg(feature = "ble-gatt-server")]
use crate::ble::gatt_server;
use crate::ble::types::*;
use crate::raw;
use crate::util::*;
use crate::RawError;

const BLE_GAP_DATA_LENGTH_DEFAULT: u8 = 27; //  The stack's default data length. <27-251>

#[derive(defmt::Format)]
pub(crate) struct OutOfConnsError;

#[derive(defmt::Format)]
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

    pub att_mtu_desired: u16,           // Requested ATT_MTU size (in bytes).
    pub att_mtu_effective: u16,         // Effective ATT_MTU size (in bytes).
    pub att_mtu_exchange_pending: bool, // Indicates that an ATT_MTU exchange request is pending (the call to @ref sd_ble_gattc_exchange_mtu_request returned @ref NRF_ERROR_BUSY).
    pub att_mtu_exchange_requested: bool, // Indicates that an ATT_MTU exchange request was made.
    #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
    pub data_length_effective: u8, // Effective data length (in bytes).

    pub rx_phys: u8,
    pub tx_phys: u8,
}

impl ConnectionState {
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
        RawError::convert(ret).dexpect(intern!("sd_ble_gap_disconnect"));

        self.disconnecting = true;
        Ok(())
    }

    pub(crate) fn on_disconnected(&mut self) {
        let conn_handle = self
            .conn_handle
            .dexpect(intern!("bug: on_disconnected when already disconnected"));

        let ibh = index_by_handle(conn_handle);
        let index = ibh.get().dexpect(intern!("conn_handle has no index"));
        ibh.set(None);

        self.conn_handle = None;

        // Signal possible in-progess operations that the connection has disconnected.
        #[cfg(feature = "ble-gatt-client")]
        gatt_client::portal(conn_handle).call(gatt_client::PortalMessage::Disconnected);
        #[cfg(feature = "ble-gatt-server")]
        gatt_server::portal(conn_handle).call(gatt_server::PortalMessage::Disconnected);

        trace!("conn {:u8}: disconnected", index);
    }

    pub(crate) fn set_att_mtu_desired(&mut self, mtu: u16) {
        self.att_mtu_desired = mtu;

        // Begin an ATT MTU exchange if necessary.
        if self.att_mtu_desired > self.att_mtu_effective as u16 {
            let ret = unsafe {
                raw::sd_ble_gattc_exchange_mtu_request(
                    self.conn_handle.unwrap(), //todo
                    self.att_mtu_desired,
                )
            };

            // TODO handle busy
            if let Err(err) = RawError::convert(ret) {
                warn!("sd_ble_gattc_exchange_mtu_request err {:?}", err);
            }
        }
    }
}

pub struct Connection {
    index: u8,
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.with_state(|state| {
            state.refcount = state.refcount.checked_sub(1).dexpect(intern!(
                "bug: dropping a conn which is already at refcount 0"
            ));

            if state.refcount == 0 {
                if state.conn_handle.is_some() {
                    trace!("conn {:u8}: dropped, disconnecting", self.index);
                    // We still leave conn_handle set, because the connection is
                    // not really disconnected until we get GAP_DISCONNECTED event.
                    state.disconnect().dewrap();
                } else {
                    trace!("conn {:u8}: dropped, already disconnected", self.index);
                }
            }
        });
    }
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        self.with_state(|state| {
            state.refcount = state
                .refcount
                .checked_add(1)
                .dexpect(intern!("Too many references to same connection"));
        });

        Self { index: self.index }
    }
}

impl Connection {
    pub fn disconnect(&self) -> Result<(), DisconnectedError> {
        self.with_state(|state| state.disconnect())
    }

    pub(crate) fn new(conn_handle: u16, role: Role) -> Result<Self, OutOfConnsError> {
        let index = find_free_index().ok_or(OutOfConnsError)?;

        let state = unsafe { &mut *STATES[index as usize].get() };

        // Initialize
        *state = Some(ConnectionState {
            refcount: 1,
            conn_handle: Some(conn_handle),
            role,

            disconnecting: false,

            att_mtu_desired: raw::BLE_GATT_ATT_MTU_DEFAULT as _,
            att_mtu_effective: raw::BLE_GATT_ATT_MTU_DEFAULT as _,
            att_mtu_exchange_pending: false,
            att_mtu_exchange_requested: false,

            #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
            data_length_effective: BLE_GAP_DATA_LENGTH_DEFAULT,

            rx_phys: 0,
            tx_phys: 0,
        });

        // Update index_by_handle
        let ibh = index_by_handle(conn_handle);
        deassert!(ibh.get().is_none(), "bug: conn_handle already has index");
        ibh.set(Some(index));

        trace!("conn {:u8}: connected", index);
        return Ok(Self { index });
    }

    pub(crate) fn with_state<T>(&self, f: impl FnOnce(&mut ConnectionState) -> T) -> T {
        with_state(self.index, f)
    }
}

// ConnectionStates by index.
static mut STATES: [UnsafeCell<Option<ConnectionState>>; CONNS_MAX] =
    [UnsafeCell::new(None); CONNS_MAX];

pub(crate) fn with_state_by_conn_handle<T>(
    conn_handle: u16,
    f: impl FnOnce(&mut ConnectionState) -> T,
) -> T {
    let index = index_by_handle(conn_handle).get().dexpect(intern!(
        "bug: with_state_by_conn_handle on conn_handle that has no state"
    ));
    with_state(index, f)
}

pub(crate) fn with_state<T>(index: u8, f: impl FnOnce(&mut ConnectionState) -> T) -> T {
    unsafe {
        let state_opt = &mut *STATES[index as usize].get();
        let (erase, res) = {
            let state = state_opt.as_mut().unwrap();
            let res = f(state);
            let erase = state.refcount == 0 && state.conn_handle.is_none();
            (erase, res)
        };

        if erase {
            trace!("conn {:u8}: index freed", index);
            *state_opt = None
        }

        res
    }
}

fn find_free_index() -> Option<u8> {
    unsafe {
        for (i, s) in STATES.iter().enumerate() {
            let state = &mut *s.get();
            if state.is_none() {
                return Some(i as u8);
            }
        }
        None
    }
}

// conn_handle -> index mapping. Used to make stuff go faster
static mut INDEX_BY_HANDLE: [Cell<Option<u8>>; CONNS_MAX] = [Cell::new(None); CONNS_MAX];

fn index_by_handle(conn_handle: u16) -> &'static Cell<Option<u8>> {
    unsafe { &INDEX_BY_HANDLE[conn_handle as usize] }
}
