//! Link-Layer Control and Adaptation Protocol

use core::marker::PhantomData;
use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};
use core::u16;

use crate::ble::*;
use crate::fmt::{assert, panic, unreachable, *};
use crate::raw;
use crate::util::{get_union_field, Portal};
use crate::{RawError, Softdevice};

fn evt_conn_handle(ble_evt: *const raw::ble_evt_t) -> u16 {
    let evt = unsafe { get_union_field(ble_evt, &(*ble_evt).evt.l2cap_evt) };
    evt.conn_handle
}

pub(crate) fn on_ch_setup_request(ble_evt: *const raw::ble_evt_t) {
    trace!("on_ch_setup_request");
    let conn_handle = evt_conn_handle(ble_evt);
    portal(conn_handle).call(PortalMessage::SetupRequest(ble_evt));
}

pub(crate) fn on_ch_setup_refused(ble_evt: *const raw::ble_evt_t) {
    trace!("on_ch_setup_refused");
    let conn_handle = evt_conn_handle(ble_evt);
    portal(conn_handle).call(PortalMessage::SetupRefused(ble_evt));
}

pub(crate) fn on_ch_setup(ble_evt: *const raw::ble_evt_t) {
    trace!("on_ch_setup");
    let conn_handle = evt_conn_handle(ble_evt);
    portal(conn_handle).call(PortalMessage::SetupDone(ble_evt));
}

pub(crate) fn on_ch_released(ble_evt: *const raw::ble_evt_t) {
    trace!("on_ch_released");
    let _conn_handle = evt_conn_handle(ble_evt);
}

pub(crate) fn on_ch_sdu_buf_released(ble_evt: *const raw::ble_evt_t) {
    trace!("on_ch_sdu_buf_released");
    unsafe {
        let l2cap_evt = get_union_field(ble_evt, &(*ble_evt).evt.l2cap_evt);
        let evt = &l2cap_evt.params.ch_sdu_buf_released;
        let pkt = unwrap!(NonNull::new(evt.sdu_buf.p_data));
        (unwrap!(PACKET_FREE))(pkt)
    }
}

pub(crate) fn on_ch_credit(ble_evt: *const raw::ble_evt_t) {
    trace!("on_ch_credit");
    let _conn_handle = evt_conn_handle(ble_evt);
}

pub(crate) fn on_ch_rx(ble_evt: *const raw::ble_evt_t) {
    trace!("on_ch_rx");
    let conn_handle = evt_conn_handle(ble_evt);
    portal(conn_handle).call(PortalMessage::Received(ble_evt));
}

pub(crate) fn on_ch_tx(ble_evt: *const raw::ble_evt_t) {
    trace!("on_ch_tx");
    unsafe {
        let l2cap_evt = get_union_field(ble_evt, &(*ble_evt).evt.l2cap_evt);
        let evt = &l2cap_evt.params.tx;
        let pkt = unwrap!(NonNull::new(evt.sdu_buf.p_data));
        (unwrap!(PACKET_FREE))(pkt)
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TxError {
    Disconnected,
    Raw(RawError),
}

impl From<DisconnectedError> for TxError {
    fn from(_err: DisconnectedError) -> Self {
        TxError::Disconnected
    }
}

impl From<RawError> for TxError {
    fn from(err: RawError) -> Self {
        TxError::Raw(err)
    }
}
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RxError {
    Disconnected,
    Raw(RawError),
}

impl From<DisconnectedError> for RxError {
    fn from(_err: DisconnectedError) -> Self {
        RxError::Disconnected
    }
}

impl From<RawError> for RxError {
    fn from(err: RawError) -> Self {
        RxError::Raw(err)
    }
}

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SetupError {
    Disconnected,
    Refused,
    Raw(RawError),
}

impl From<DisconnectedError> for SetupError {
    fn from(_err: DisconnectedError) -> Self {
        SetupError::Disconnected
    }
}

impl From<RawError> for SetupError {
    fn from(err: RawError) -> Self {
        SetupError::Raw(err)
    }
}

static PORTALS: [Portal<PortalMessage>; CONNS_MAX] = [Portal::new(); CONNS_MAX];
pub(crate) fn portal(conn_handle: u16) -> &'static Portal<PortalMessage> {
    &PORTALS[conn_handle as usize]
}

pub(crate) enum PortalMessage {
    SetupDone(*const raw::ble_evt_t),
    SetupRefused(*const raw::ble_evt_t),
    SetupRequest(*const raw::ble_evt_t),
    Received(*const raw::ble_evt_t),
    Disconnected,
}

pub trait Packet: Sized {
    const MTU: usize;
    fn allocate() -> Option<NonNull<u8>>;
    fn into_raw_parts(self) -> (NonNull<u8>, usize);
    unsafe fn from_raw_parts(ptr: NonNull<u8>, len: usize) -> Self;
}

pub struct L2cap<P: Packet> {
    _private: PhantomData<*mut P>,
}

static IS_INIT: AtomicBool = AtomicBool::new(false);
static mut PACKET_FREE: Option<unsafe fn(NonNull<u8>)> = None;

impl<P: Packet> L2cap<P> {
    pub fn init(_sd: &Softdevice) -> Self {
        if IS_INIT
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            panic!("L2cap::init() called multiple times.")
        }

        unsafe {
            PACKET_FREE = Some(|ptr| {
                P::from_raw_parts(ptr, 0);
                // create Packet from pointer, will be freed on drop
            })
        }

        Self {
            _private: PhantomData,
        }
    }

    pub async fn setup(
        &self,
        conn: &Connection,
        config: &Config,
    ) -> Result<Channel<P>, SetupError> {
        let sd = unsafe { Softdevice::steal() };

        let conn_handle = conn.with_state(|state| state.check_connected())?;
        let mut cid: u16 = raw::BLE_L2CAP_CID_INVALID as _;
        let params = raw::ble_l2cap_ch_setup_params_t {
            le_psm: config.psm,
            status: 0, // only used when responding
            rx_params: raw::ble_l2cap_ch_rx_params_t {
                rx_mps: sd.l2cap_rx_mps,
                rx_mtu: P::MTU as u16,
                sdu_buf: raw::ble_data_t {
                    len: 0,
                    p_data: ptr::null_mut(),
                },
            },
        };
        let ret = unsafe { raw::sd_ble_l2cap_ch_setup(conn_handle, &mut cid, &params) };
        if let Err(err) = RawError::convert(ret) {
            warn!("sd_ble_l2cap_ch_setup err {:?}", err);
            return Err(err.into());
        }
        info!("cid {:?}", cid);

        portal(conn_handle)
            .wait_once(|msg| match msg {
                PortalMessage::Disconnected => Err(SetupError::Disconnected),
                PortalMessage::SetupDone(ble_evt) => unsafe {
                    let l2cap_evt = get_union_field(ble_evt, &(*ble_evt).evt.l2cap_evt);
                    let _evt = &l2cap_evt.params.ch_setup;

                    // default is 1
                    if config.credits != 1 {
                        let ret = unsafe {
                            raw::sd_ble_l2cap_ch_flow_control(
                                conn_handle,
                                cid,
                                config.credits,
                                ptr::null_mut(),
                            )
                        };
                        if let Err(err) = RawError::convert(ret) {
                            warn!("sd_ble_l2cap_ch_flow_control err {:?}", err);
                            return Err(err.into());
                        }
                    }

                    Ok(Channel {
                        conn: conn.clone(),
                        cid,
                        _private: PhantomData,
                    })
                },
                PortalMessage::SetupRefused(ble_evt) => unsafe {
                    let l2cap_evt = get_union_field(ble_evt, &(*ble_evt).evt.l2cap_evt);
                    let _evt = &l2cap_evt.params.ch_setup_refused;
                    Err(SetupError::Refused)
                },
                _ => unreachable!(),
            })
            .await
    }

    pub async fn listen(
        &self,
        conn: &Connection,
        config: &Config,
    ) -> Result<Channel<P>, SetupError> {
        let sd = unsafe { Softdevice::steal() };
        let conn_handle = conn.with_state(|state| state.check_connected())?;

        portal(conn_handle)
            .wait_many(|msg| match msg {
                PortalMessage::Disconnected => Some(Err(SetupError::Disconnected)),
                PortalMessage::SetupRequest(ble_evt) => unsafe {
                    let l2cap_evt = get_union_field(ble_evt, &(*ble_evt).evt.l2cap_evt);
                    let evt = &l2cap_evt.params.ch_setup_request;

                    let mut cid: u16 = l2cap_evt.local_cid;
                    if evt.le_psm == config.psm {
                        let params = raw::ble_l2cap_ch_setup_params_t {
                            le_psm: evt.le_psm,
                            status: raw::BLE_L2CAP_CH_STATUS_CODE_SUCCESS as _,
                            rx_params: raw::ble_l2cap_ch_rx_params_t {
                                rx_mps: sd.l2cap_rx_mps,
                                rx_mtu: P::MTU as u16,
                                sdu_buf: raw::ble_data_t {
                                    len: 0,
                                    p_data: ptr::null_mut(),
                                },
                            },
                        };

                        let ret = raw::sd_ble_l2cap_ch_setup(conn_handle, &mut cid, &params);
                        if let Err(err) = RawError::convert(ret) {
                            warn!("sd_ble_l2cap_ch_setup err {:?}", err);
                            return Some(Err(err.into()));
                        }

                        // default is 1
                        if config.credits != 1 {
                            let ret = unsafe {
                                raw::sd_ble_l2cap_ch_flow_control(
                                    conn_handle,
                                    cid,
                                    config.credits,
                                    ptr::null_mut(),
                                )
                            };
                            if let Err(err) = RawError::convert(ret) {
                                warn!("sd_ble_l2cap_ch_flow_control err {:?}", err);
                                return Some(Err(err.into()));
                            }
                        }

                        Some(Ok(Channel {
                            _private: PhantomData,
                            cid,
                            conn: conn.clone(),
                        }))
                    } else {
                        let params = raw::ble_l2cap_ch_setup_params_t {
                            le_psm: evt.le_psm,
                            status: raw::BLE_L2CAP_CH_STATUS_CODE_LE_PSM_NOT_SUPPORTED as _,
                            rx_params: mem::zeroed(),
                        };

                        let ret = raw::sd_ble_l2cap_ch_setup(conn_handle, &mut cid, &params);
                        if let Err(err) = RawError::convert(ret) {
                            warn!("sd_ble_l2cap_ch_setup err {:?}", err);
                        }

                        None
                    }
                },
                _ => unreachable!(),
            })
            .await
    }
}

pub struct Config {
    pub psm: u16,
    pub credits: u16,
}

pub struct Channel<P: Packet> {
    _private: PhantomData<*mut P>,
    conn: Connection,
    cid: u16,
}

impl<P: Packet> Clone for Channel<P> {
    fn clone(&self) -> Self {
        Self {
            _private: PhantomData,
            conn: self.conn.clone(),
            cid: self.cid,
        }
    }
}

impl<P: Packet> Channel<P> {
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn tx(&self, sdu: P) -> Result<(), TxError> {
        let conn_handle = self.conn.with_state(|s| s.check_connected())?;

        let (ptr, len) = sdu.into_raw_parts();
        assert!(len <= P::MTU);
        let data = raw::ble_data_t {
            p_data: ptr.as_ptr(),
            len: len as u16,
        };

        let ret = unsafe { raw::sd_ble_l2cap_ch_tx(conn_handle, self.cid, &data) };
        if let Err(err) = RawError::convert(ret) {
            warn!("sd_ble_l2cap_ch_tx err {:?}", err);
            // The SD didn't take ownership of the buffer, so it's on us to free it.
            // Reconstruct the P and let it get dropped.
            unsafe { P::from_raw_parts(ptr, len) };
            return Err(err.into());
        }

        Ok(())
    }

    pub async fn rx(&self) -> Result<P, RxError> {
        let conn_handle = self.conn.with_state(|s| s.check_connected())?;

        let ptr = unwrap!(P::allocate());
        let data = raw::ble_data_t {
            p_data: ptr.as_ptr(),
            len: P::MTU as u16,
        };

        let ret = unsafe { raw::sd_ble_l2cap_ch_rx(conn_handle, self.cid, &data) };
        if let Err(err) = RawError::convert(ret) {
            warn!("sd_ble_l2cap_ch_rx err {:?}", err);
            // The SD didn't take ownership of the buffer, so it's on us to free it.
            // Reconstruct the P and let it get dropped.
            unsafe { P::from_raw_parts(ptr, 0) };
            return Err(err.into());
        }

        portal(conn_handle)
            .wait_once(|msg| match msg {
                PortalMessage::Disconnected => Err(RxError::Disconnected),
                PortalMessage::Received(ble_evt) => unsafe {
                    let l2cap_evt = get_union_field(ble_evt, &(*ble_evt).evt.l2cap_evt);
                    let evt = &l2cap_evt.params.rx;

                    let ptr = unwrap!(NonNull::new(evt.sdu_buf.p_data));
                    let len = evt.sdu_len;
                    let pkt = Packet::from_raw_parts(ptr, len as usize);
                    Ok(pkt)
                },
                _ => unreachable!(),
            })
            .await
    }
}
