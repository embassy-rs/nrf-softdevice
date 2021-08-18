use crate::ble::*;
use crate::raw;
use crate::util::get_union_field;
use crate::RawError;

pub(crate) unsafe fn on_evt(ble_evt: *const raw::ble_evt_t) {
    let gap_evt = get_union_field(ble_evt, &(*ble_evt).evt.gap_evt);
    match (*ble_evt).header.evt_id as u32 {
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONNECTED => {
            let params = &gap_evt.params.connected;

            debug!(
                "conn_params conn_sup_timeout={:?} max_conn_interval={:?} min_conn_interval={:?} slave_latency={:?}",
                params.conn_params.conn_sup_timeout,
                params.conn_params.max_conn_interval,
                params.conn_params.min_conn_interval,
                params.conn_params.slave_latency,
            );

            let handled = match Role::from_raw(params.role) {
                #[cfg(feature = "ble-central")]
                Role::Central => central::CONNECT_PORTAL.call(ble_evt),
                #[cfg(feature = "ble-peripheral")]
                Role::Peripheral => peripheral::ADV_PORTAL.call(ble_evt),
            };
            if !handled {
                raw::sd_ble_gap_disconnect(
                    gap_evt.conn_handle,
                    raw::BLE_HCI_REMOTE_USER_TERMINATED_CONNECTION as _,
                );
            }
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => {
            trace!("on_disconnected conn_handle={:?}", gap_evt.conn_handle);
            connection::with_state_by_conn_handle(gap_evt.conn_handle, |state| {
                state.on_disconnected(ble_evt)
            });
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE => {
            let conn_params = gap_evt.params.conn_param_update.conn_params;

            debug!(
                "on_conn_param_update conn_handle={:?} conn_sup_timeout={:?} max_conn_interval={:?} min_conn_interval={:?} slave_latency={:?}",
                gap_evt.conn_handle,
                conn_params.conn_sup_timeout,
                conn_params.max_conn_interval,
                conn_params.min_conn_interval,
                conn_params.slave_latency,
            );

            connection::with_state_by_conn_handle(gap_evt.conn_handle, |state| {
                state.conn_params = conn_params;
            });
        }
        #[cfg(feature = "ble-central")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_PARAM_UPDATE_REQUEST => {
            let conn_handle = gap_evt.conn_handle;
            let conn_params = gap_evt.params.conn_param_update_request.conn_params;
            debug!(
                "on_conn_param_update_request conn_handle={:?} conn_sup_timeout={:?} max_conn_interval={:?} min_conn_interval={:?} slave_latency={:?}",
                gap_evt.conn_handle,
                conn_params.conn_sup_timeout,
                conn_params.max_conn_interval,
                conn_params.min_conn_interval,
                conn_params.slave_latency,
            );

            let ret = raw::sd_ble_gap_conn_param_update(conn_handle, &conn_params);
            if let Err(err) = RawError::convert(ret) {
                warn!("sd_ble_gap_conn_param_update err {:?}", err);
                return;
            }
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_TIMEOUT => {
            trace!("on_timeout conn_handle={:?}", gap_evt.conn_handle);

            let params = &gap_evt.params.timeout;
            match params.src as u32 {
                #[cfg(feature = "ble-central")]
                raw::BLE_GAP_TIMEOUT_SRC_CONN => central::CONNECT_PORTAL.call(ble_evt),
                #[cfg(feature = "ble-central")]
                raw::BLE_GAP_TIMEOUT_SRC_SCAN => central::SCAN_PORTAL.call(ble_evt),
                x => panic!("unknown timeout src {:?}", x),
            };
        }
        #[cfg(feature = "ble-peripheral")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_SET_TERMINATED => {
            trace!("adv_set_termnated");
            peripheral::ADV_PORTAL.call(ble_evt);
        }
        #[cfg(feature = "ble-central")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_ADV_REPORT => {
            trace!("central on_adv_report");
            central::SCAN_PORTAL.call(ble_evt);
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE_REQUEST => {
            let peer_preferred_phys = gap_evt.params.phy_update_request.peer_preferred_phys;
            let conn_handle = gap_evt.conn_handle;

            trace!(
                "on_phy_update_request conn_handle={:?} rx_phys={:?} tx_phys={:?}",
                conn_handle,
                peer_preferred_phys.rx_phys,
                peer_preferred_phys.tx_phys
            );

            let phys = raw::ble_gap_phys_t {
                rx_phys: peer_preferred_phys.rx_phys,
                tx_phys: peer_preferred_phys.tx_phys,
            };

            let ret = raw::sd_ble_gap_phy_update(conn_handle, &phys as *const raw::ble_gap_phys_t);

            if let Err(_err) = RawError::convert(ret) {
                warn!("sd_ble_gap_phy_update err {:?}", _err);
            }
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_PHY_UPDATE => {
            let _phy_update = gap_evt.params.phy_update;

            trace!(
                "on_phy_update conn_handle={:?} status={:?} rx_phy={:?} tx_phy={:?}",
                gap_evt.conn_handle,
                _phy_update.status,
                _phy_update.rx_phy,
                _phy_update.tx_phy
            );
        }
        #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE_REQUEST => {
            let _peer_params = gap_evt.params.data_length_update_request.peer_params;

            trace!(
                "on_data_length_update_request conn_handle={:?} max_rx_octets={:?} max_rx_time_us={:?} max_tx_octets={:?} max_tx_time_us={:?}",
                gap_evt.conn_handle,
                _peer_params.max_rx_octets,
                _peer_params.max_rx_time_us,
                _peer_params.max_tx_octets,
                _peer_params.max_tx_time_us,
            );

            let conn_handle = gap_evt.conn_handle;
            do_data_length_update(conn_handle, core::ptr::null());
        }
        #[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_DATA_LENGTH_UPDATE => {
            let effective_params = gap_evt.params.data_length_update.effective_params;

            connection::with_state_by_conn_handle(gap_evt.conn_handle, |state| {
                state.data_length_effective = effective_params.max_tx_octets as u8;
            });

            debug!(
                "on_data_length_update conn_handle={:?} max_rx_octets={:?} max_rx_time_us={:?} max_tx_octets={:?} max_tx_time_us={:?}",
                gap_evt.conn_handle,
                effective_params.max_rx_octets,
                effective_params.max_rx_time_us,
                effective_params.max_tx_octets,
                effective_params.max_tx_time_us,
            );
        }
        _ => {}
    }
}

#[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
pub(crate) unsafe fn do_data_length_update(
    conn_handle: u16,
    params: *const raw::ble_gap_data_length_params_t,
) {
    let mut dl_limitation = core::mem::zeroed();
    let ret = raw::sd_ble_gap_data_length_update(conn_handle, params, &mut dl_limitation);
    if let Err(_err) = RawError::convert(ret) {
        warn!("sd_ble_gap_data_length_update err {:?}", _err);

        if dl_limitation.tx_payload_limited_octets != 0
            || dl_limitation.rx_payload_limited_octets != 0
        {
            warn!(
                "The requested TX/RX packet length is too long by {:?}/{:?} octets.",
                dl_limitation.tx_payload_limited_octets, dl_limitation.rx_payload_limited_octets
            );
        }

        if dl_limitation.tx_rx_time_limited_us != 0 {
            warn!(
                "The requested combination of TX and RX packet lengths is too long by {:?} us",
                dl_limitation.tx_rx_time_limited_us
            );
        }
    }
}
