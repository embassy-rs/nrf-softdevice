use core::mem;
use core::ptr;

use crate::ble::*;
use crate::raw;
use crate::util::{panic, *};
use crate::RawError;

#[rustfmt::skip]
pub(crate) unsafe fn on_evt(ble_evt: *const raw::ble_evt_t) {
    let bounded = BoundedLifetime;
    let evt = bounded.deref(ble_evt);
    //defmt::trace!("ble evt {:istr}", evt_str(evt.header.evt_id as u32));
    match evt.header.evt_id as u32 {
        raw::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_REQUEST => on_user_mem_request(ble_evt, get_union_field(ble_evt, &evt.evt.common_evt)),
        raw::BLE_COMMON_EVTS_BLE_EVT_USER_MEM_RELEASE => on_user_mem_release(ble_evt, get_union_field(ble_evt, &evt.evt.common_evt)),

        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONNECTED => on_connected(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => on_disconnected(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE => on_conn_param_update(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_PARAMS_REQUEST => on_sec_params_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-peripheral")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_INFO_REQUEST => peripheral::on_sec_info_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_PASSKEY_DISPLAY => on_passkey_display(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_KEY_PRESSED => on_key_pressed(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_KEY_REQUEST => on_auth_key_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_LESC_DHKEY_REQUEST => on_lesc_dhkey_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_STATUS => on_auth_status(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_SEC_UPDATE => on_conn_sec_update(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => on_timeout(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_RSSI_CHANGED => on_rssi_changed(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-central")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_REPORT => central::on_adv_report(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_REQUEST => on_sec_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-central")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE_REQUEST => central::on_conn_param_update_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-peripheral")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_SCAN_REQ_REPORT => peripheral::on_scan_req_report(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE_REQUEST => on_phy_update_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE => on_phy_update(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(any(feature="s113", feature="s132", feature="s140"))]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE_REQUEST => on_data_length_update_request(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(any(feature="s113", feature="s132", feature="s140"))]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE => on_data_length_update(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-central")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_QOS_CHANNEL_SURVEY_REPORT => central::on_qos_channel_survey_report(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),
        #[cfg(feature="ble-peripheral")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_SET_TERMINATED => peripheral::on_adv_set_terminated(ble_evt, get_union_field(ble_evt, &evt.evt.gap_evt)),

        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REQUEST => l2cap::on_ch_setup_request(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP_REFUSED => l2cap::on_ch_setup_refused(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SETUP => l2cap::on_ch_setup(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RELEASED => l2cap::on_ch_released(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_SDU_BUF_RELEASED => l2cap::on_ch_sdu_buf_released(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_CREDIT => l2cap::on_ch_credit(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_RX => l2cap::on_ch_rx(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),
        #[cfg(feature="ble-l2cap")]
        raw::BLE_L2CAP_EVTS_BLE_L2CAP_EVT_CH_TX => l2cap::on_ch_tx(ble_evt, get_union_field(ble_evt, &evt.evt.l2cap_evt)),

        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_PRIM_SRVC_DISC_RSP => gatt_client::on_prim_srvc_disc_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_REL_DISC_RSP => gatt_client::on_rel_disc_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_DISC_RSP => gatt_client::on_char_disc_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_DESC_DISC_RSP => gatt_client::on_desc_disc_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_ATTR_INFO_DISC_RSP => gatt_client::on_attr_info_disc_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VAL_BY_UUID_READ_RSP => gatt_client::on_char_val_by_uuid_read_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_READ_RSP => gatt_client::on_read_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_CHAR_VALS_READ_RSP => gatt_client::on_char_vals_read_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_RSP => gatt_client::on_write_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_HVX => gatt_client::on_hvx(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_EXCHANGE_MTU_RSP => gatt_client::on_exchange_mtu_rsp(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_TIMEOUT => gatt_client::on_timeout(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),
        #[cfg(feature="ble-gatt-client")]
        raw::BLE_GATTC_EVTS_BLE_GATTC_EVT_WRITE_CMD_TX_COMPLETE => gatt_client::on_write_cmd_tx_complete(ble_evt, get_union_field(ble_evt, &evt.evt.gattc_evt)),

        #[cfg(feature="ble-gatt-server")]
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_WRITE => gatt_server::on_write(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        #[cfg(feature="ble-gatt-server")]
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_RW_AUTHORIZE_REQUEST => gatt_server::on_rw_authorize_request(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        #[cfg(feature="ble-gatt-server")]
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_SYS_ATTR_MISSING => gatt_server::on_sys_attr_missing(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        #[cfg(feature="ble-gatt-server")]
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVC => gatt_server::on_hvc(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        #[cfg(feature="ble-gatt-server")]
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_SC_CONFIRM => gatt_server::on_sc_confirm(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        #[cfg(feature="ble-gatt-server")]
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_EXCHANGE_MTU_REQUEST => gatt_server::on_exchange_mtu_request(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        #[cfg(feature="ble-gatt-server")]
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_TIMEOUT => gatt_server::on_timeout(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),
        #[cfg(feature="ble-gatt-server")]
        raw::BLE_GATTS_EVTS_BLE_GATTS_EVT_HVN_TX_COMPLETE => gatt_server::on_hvn_tx_complete(ble_evt, get_union_field(ble_evt, &evt.evt.gatts_evt)),

        x => panic!("Unknown ble evt {:u32}", x),
    }
}

fn on_user_mem_request(_ble_evt: *const raw::ble_evt_t, _common_evt: &raw::ble_common_evt_t) {
    trace!("on_user_mem_request");
}
fn on_user_mem_release(_ble_evt: *const raw::ble_evt_t, _common_evt: &raw::ble_common_evt_t) {
    trace!("on_user_mem_release");
}

pub(crate) unsafe fn on_connected(_ble_evt: *const raw::ble_evt_t, gap_evt: &raw::ble_gap_evt_t) {
    trace!("on_connected conn_handle={:u16}", gap_evt.conn_handle);

    let params = &gap_evt.params.connected;
    let conn_handle = gap_evt.conn_handle;
    let role = Role::from_raw(params.role);

    let res = match Connection::new(conn_handle, role) {
        Ok(conn) => {
            #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
            do_data_length_update(conn_handle, ptr::null());

            Ok(conn)
        }
        Err(_) => {
            raw::sd_ble_gap_disconnect(
                conn_handle,
                raw::BLE_HCI_REMOTE_USER_TERMINATED_CONNECTION as _,
            );
            Err(RawError::Busy)
        }
    };

    match role {
        #[cfg(feature = "ble-central")]
        Role::Central => central::CONNECT_PORTAL.call(res.map_err(|e| e.into())),
        #[cfg(feature = "ble-peripheral")]
        Role::Peripheral => peripheral::ADV_PORTAL.call(res.map_err(|e| e.into())),
    }
}

pub(crate) unsafe fn on_disconnected(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    trace!("on_disconnected conn_handle={:u16}", gap_evt.conn_handle);
    let conn_handle = gap_evt.conn_handle;
    connection::with_state_by_conn_handle(conn_handle, |state| state.on_disconnected());
}

pub(crate) unsafe fn on_conn_param_update(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    let conn_params = gap_evt.params.conn_param_update.conn_params;

    trace!(
        "on_conn_param_update conn_handle={:u16} conn_sup_timeout={:u16} max_conn_interval={:u16} min_conn_interval={:u16} slave_latency={:u16}",
        gap_evt.conn_handle,
        conn_params.conn_sup_timeout,
        conn_params.max_conn_interval,
        conn_params.min_conn_interval,
        conn_params.slave_latency,
    );
}

pub(crate) unsafe fn on_sec_params_request(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    trace!(
        "on_sec_params_request conn_handle={:u16}",
        gap_evt.conn_handle
    );
}

pub(crate) unsafe fn on_passkey_display(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    trace!("on_passkey_display conn_handle={:u16}", gap_evt.conn_handle);
}

pub(crate) unsafe fn on_key_pressed(_ble_evt: *const raw::ble_evt_t, gap_evt: &raw::ble_gap_evt_t) {
    trace!("on_key_pressed conn_handle={:u16}", gap_evt.conn_handle);
}
pub(crate) unsafe fn on_auth_key_request(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    trace!(
        "on_auth_key_request conn_handle={:u16}",
        gap_evt.conn_handle
    );
}

pub(crate) unsafe fn on_lesc_dhkey_request(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    trace!(
        "on_lesc_dhkey_request conn_handle={:u16}",
        gap_evt.conn_handle
    );
}

pub(crate) unsafe fn on_auth_status(_ble_evt: *const raw::ble_evt_t, gap_evt: &raw::ble_gap_evt_t) {
    trace!("on_auth_status conn_handle={:u16}", gap_evt.conn_handle);
}

pub(crate) unsafe fn on_conn_sec_update(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    trace!("on_conn_sec_update conn_handle={:u16}", gap_evt.conn_handle);
}

pub(crate) unsafe fn on_timeout(ble_evt: *const raw::ble_evt_t, gap_evt: &raw::ble_gap_evt_t) {
    trace!("on_timeout conn_handle={:u16}", gap_evt.conn_handle);

    let params = &gap_evt.params.timeout;
    match params.src as u32 {
        #[cfg(feature = "ble-central")]
        raw::BLE_GAP_TIMEOUT_SRC_CONN => {
            central::CONNECT_PORTAL.call(Err(central::ConnectError::Timeout))
        }
        x => panic!("unknown timeout src {:u32}", x),
    }
}

pub(crate) unsafe fn on_rssi_changed(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    trace!("on_rssi_changed conn_handle={:u16}", gap_evt.conn_handle);
}

pub(crate) unsafe fn on_sec_request(_ble_evt: *const raw::ble_evt_t, gap_evt: &raw::ble_gap_evt_t) {
    trace!("on_sec_request conn_handle={:u16}", gap_evt.conn_handle);
}

/// Called when a phy update has been requested by peer
pub(crate) unsafe fn on_phy_update_request(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    let peer_preferred_phys = gap_evt.params.phy_update_request.peer_preferred_phys;
    let conn_handle = gap_evt.conn_handle;

    trace!(
        "on_phy_update_request conn_handle={:u16} rx_phys={:u8} tx_phys={:u8}",
        conn_handle,
        peer_preferred_phys.rx_phys,
        peer_preferred_phys.tx_phys
    );

    let phys = connection::with_state_by_conn_handle(conn_handle, |state| raw::ble_gap_phys_t {
        rx_phys: state.rx_phys,
        tx_phys: state.tx_phys,
    });

    let ret = raw::sd_ble_gap_phy_update(conn_handle, &phys as *const raw::ble_gap_phys_t);

    if let Err(err) = RawError::convert(ret) {
        warn!("sd_ble_gap_phy_update err {:?}", err);
    }
}

/// Called when a phy update was completed with status phy_update.status
pub(crate) unsafe fn on_phy_update(_ble_evt: *const raw::ble_evt_t, gap_evt: &raw::ble_gap_evt_t) {
    let phy_update = gap_evt.params.phy_update;

    trace!(
        "on_phy_update conn_handle={:u16} status={:u8} rx_phy={:u8} tx_phy={:u8}",
        gap_evt.conn_handle,
        phy_update.status,
        phy_update.rx_phy,
        phy_update.tx_phy
    );
}

/// Called when a data length update has been requested by peer
#[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
pub(crate) unsafe fn on_data_length_update_request(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    let peer_params = gap_evt.params.data_length_update_request.peer_params;

    trace!(
        "on_data_length_update_request conn_handle={:u16} max_rx_octets={:u16} max_rx_time_us={:u16} max_tx_octets={:u16} max_tx_time_us={:u16}",
        gap_evt.conn_handle,
        peer_params.max_rx_octets,
        peer_params.max_rx_time_us,
        peer_params.max_tx_octets,
        peer_params.max_tx_time_us,
    );

    let conn_handle = gap_evt.conn_handle;
    do_data_length_update(conn_handle, ptr::null());
}

/// Called when a data length update was completed sucessfully
#[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
pub(crate) unsafe fn on_data_length_update(
    _ble_evt: *const raw::ble_evt_t,
    gap_evt: &raw::ble_gap_evt_t,
) {
    let effective_params = gap_evt.params.data_length_update.effective_params;

    connection::with_state_by_conn_handle(gap_evt.conn_handle, |state| {
        state.data_length_effective = effective_params.max_tx_octets as u8;
    });

    trace!(
        "on_data_length_update conn_handle={:u16} max_rx_octets={:u16} max_rx_time_us={:u16} max_tx_octets={:u16} max_tx_time_us={:u16}",
        gap_evt.conn_handle,
        effective_params.max_rx_octets,
        effective_params.max_rx_time_us,
        effective_params.max_tx_octets,
        effective_params.max_tx_time_us,
    );
}

#[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
unsafe fn do_data_length_update(
    conn_handle: u16,
    params: *const raw::ble_gap_data_length_params_t,
) {
    let mut dl_limitation = mem::zeroed();
    let ret = raw::sd_ble_gap_data_length_update(conn_handle, params, &mut dl_limitation);
    if let Err(err) = RawError::convert(ret) {
        warn!("sd_ble_gap_data_length_update err {:?}", err);

        if dl_limitation.tx_payload_limited_octets != 0
            || dl_limitation.rx_payload_limited_octets != 0
        {
            warn!(
                "The requested TX/RX packet length is too long by {:u16}/{:u16} octets.",
                dl_limitation.tx_payload_limited_octets, dl_limitation.rx_payload_limited_octets
            );
        }

        if dl_limitation.tx_rx_time_limited_us != 0 {
            warn!(
                "The requested combination of TX and RX packet lengths is too long by {:u16} us",
                dl_limitation.tx_rx_time_limited_us
            );
        }
    }
}
