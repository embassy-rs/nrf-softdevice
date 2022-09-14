use crate::ble::*;
use crate::util::get_union_field;
use crate::{raw, RawError};

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
                raw::sd_ble_gap_disconnect(gap_evt.conn_handle, raw::BLE_HCI_REMOTE_USER_TERMINATED_CONNECTION as _);
            }
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_DISCONNECTED => {
            trace!("on_disconnected conn_handle={:?}", gap_evt.conn_handle);
            connection::with_state_by_conn_handle(gap_evt.conn_handle, |state| state.on_disconnected(ble_evt));
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
        #[cfg(feature = "ble-rssi")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_RSSI_CHANGED => {
            let new_rssi = gap_evt.params.rssi_changed.rssi;
            connection::with_state_by_conn_handle(gap_evt.conn_handle, |state| {
                state.rssi = match state.rssi {
                    None => Some(new_rssi),
                    Some(old_rssi) => Some((((old_rssi as i16) * 7 + (new_rssi as i16)) / 8) as i8),
                };
            });
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_PARAMS_REQUEST => {
            let params = &gap_evt.params.sec_params_request;
            let peer_params = params.peer_params;
            trace!("ble evt sec params request conn={:x}, bond={:?}, io_caps={:?}, keypress={:?}, lesc={:?}, mitm={:?}, oob={:?}, key_size={}..={}",
                    gap_evt.conn_handle, peer_params.bond(), peer_params.io_caps(), peer_params.keypress(), peer_params.lesc(), peer_params.mitm(), peer_params.oob(),
                    peer_params.min_key_size, peer_params.max_key_size);

            let (sec_params, keyset) = connection::with_state_by_conn_handle(gap_evt.conn_handle, |state| {
                let mut sec_params: raw::ble_gap_sec_params_t = core::mem::zeroed();

                sec_params.min_key_size = 7;
                sec_params.max_key_size = 16;

                sec_params.kdist_own.set_enc(1);
                sec_params.kdist_own.set_id(1);
                sec_params.kdist_peer.set_enc(1);
                sec_params.kdist_peer.set_id(1);
                sec_params.set_io_caps(raw::BLE_GAP_IO_CAPS_NONE as u8);

                #[cfg(feature = "ble-sec")]
                if let Some(handler) = state.security.handler {
                    sec_params.set_io_caps(handler.io_capabilities().to_io_caps());
                    if let Some(conn) = Connection::from_handle(gap_evt.conn_handle) {
                        sec_params.set_bond(handler.can_bond(&conn) as u8);
                        sec_params.set_oob(handler.can_recv_out_of_band(&conn) as u8);
                    }
                }

                (sec_params, state.keyset())
            });

            let ret = raw::sd_ble_gap_sec_params_reply(
                gap_evt.conn_handle,
                raw::BLE_GAP_SEC_STATUS_SUCCESS as u8,
                &sec_params,
                &keyset,
            );

            if let Err(_err) = RawError::convert(ret) {
                warn!("sd_ble_gap_sec_params_reply err {:?}", _err);
            }
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_PASSKEY_DISPLAY => {
            let params = &gap_evt.params.passkey_display;
            debug_assert_eq!(params.match_request(), 0);
            trace!(
                "on_passkey_display passkey={}",
                core::str::from_utf8_unchecked(&params.passkey)
            );
            #[cfg(feature = "ble-sec")]
            connection::with_state_by_conn_handle(gap_evt.conn_handle, |state| {
                if let Some(handler) = state.security.handler {
                    handler.display_passkey(&params.passkey)
                }
            });
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_KEY_REQUEST => {
            let params = &gap_evt.params.auth_key_request;
            trace!("on_auth_key_request key_type={}", params.key_type);

            #[cfg(not(feature = "ble-sec"))]
            let handled = false;
            #[cfg(feature = "ble-sec")]
            let handled = connection::with_state_by_conn_handle(gap_evt.conn_handle, |state| {
                state
                    .security
                    .handler
                    .and_then(|handler| match u32::from(params.key_type) {
                        raw::BLE_GAP_AUTH_KEY_TYPE_PASSKEY => Connection::from_handle(gap_evt.conn_handle)
                            .map(|conn| handler.enter_passkey(PasskeyReply::new(conn))),
                        raw::BLE_GAP_AUTH_KEY_TYPE_OOB => Connection::from_handle(gap_evt.conn_handle)
                            .map(|conn| handler.recv_out_of_band(OutOfBandReply::new(conn))),
                        _ => None,
                    })
            })
            .is_some();

            if !handled {
                let ret = raw::sd_ble_gap_auth_key_reply(
                    gap_evt.conn_handle,
                    raw::BLE_GAP_AUTH_KEY_TYPE_NONE as u8,
                    core::ptr::null(),
                );

                if let Err(_err) = RawError::convert(ret) {
                    warn!("sd_ble_gap_auth_key_reply err {:?}", _err);
                }
            }
        }
        #[cfg(feature = "ble-peripheral")]
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_SEC_INFO_REQUEST => {
            let params = &gap_evt.params.sec_info_request;
            trace!("ble evt sec info request: enc_info={}, id_info={}, sign_info={}, master_id: {{ ediv: {:x}, rand: {:?} }}, peer_addr: {{ addr: {:?}, addr_id_peer: {}, addr_type: {} }}",
                params.enc_info(), params.id_info(), params.sign_info(), params.master_id.ediv, params.master_id.rand,
                params.peer_addr.addr, params.peer_addr.addr_id_peer(), params.peer_addr.addr_type());

            #[cfg(feature = "ble-sec")]
            let key = Connection::from_handle(gap_evt.conn_handle).and_then(|conn| {
                conn.security_handler()
                    .and_then(|x| x.get_key(&conn, MasterId::from_raw(params.master_id)))
            });

            #[cfg(not(feature = "ble-sec"))]
            let key_ptr = core::ptr::null();
            #[cfg(feature = "ble-sec")]
            let key_ptr = key
                .as_ref()
                .map(|x| x.as_raw() as *const _)
                .unwrap_or(core::ptr::null());

            let ret =
                raw::sd_ble_gap_sec_info_reply(gap_evt.conn_handle, key_ptr, core::ptr::null(), core::ptr::null());

            if let Err(_err) = RawError::convert(ret) {
                warn!("sd_ble_gap_sec_info_reply err {:?}", _err);
            }
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_CONN_SEC_UPDATE => {
            let params = &gap_evt.params.conn_sec_update;
            trace!("ble evt conn sec update");
            if let Some(conn) = Connection::from_handle(gap_evt.conn_handle) {
                conn.with_state(|state| {
                    state.security_mode = SecurityMode::try_from_raw(params.conn_sec.sec_mode).unwrap_or_default();
                    #[cfg(feature = "ble-sec")]
                    if let Some(handler) = state.security.handler {
                        handler.on_security_update(&conn, state.security_mode);
                    }
                });
            }
        }
        raw::BLE_GAP_EVTS_BLE_GAP_EVT_AUTH_STATUS => {
            let params = &gap_evt.params.auth_status;
            trace!(
                "ble evt auth status: bonded={}, error_src={}, lesc={}, kdist_own={}, kdist_peer={}",
                params.bonded(),
                params.error_src(),
                params.lesc(),
                params.kdist_own._bitfield_1.get(0, 8),
                params.kdist_peer._bitfield_1.get(0, 8)
            );
            #[cfg(feature = "ble-sec")]
            if u32::from(params.auth_status) == raw::BLE_GAP_SEC_STATUS_SUCCESS && params.bonded() != 0 {
                if let Some(conn) = Connection::from_handle(gap_evt.conn_handle) {
                    conn.with_state(|state| {
                        if let Some(handler) = state.security.handler {
                            let peer_id = if params.kdist_peer.id() != 0 {
                                IdentityKey::from_raw(state.security.peer_id)
                            } else {
                                debug!("Peer identity key not distributed; falling back to address");
                                IdentityKey::from_addr(conn.peer_address())
                            };

                            handler.on_bonded(
                                &conn,
                                MasterId::from_raw(state.security.own_enc_key.master_id),
                                EncryptionInfo::from_raw(state.security.own_enc_key.enc_info),
                                peer_id,
                            );
                        }
                    });
                }
            }
        }
        // BLE_GAP_EVTS_BLE_GAP_EVT_KEY_PRESSED (LESC central pairing)
        // BLE_GAP_EVTS_BLE_GAP_EVT_LESC_DHKEY_REQUEST (LESC key calculation)
        // BLE_GAP_EVTS_BLE_GAP_EVT_SEC_REQUEST (Peripheral-initiated security request)
        // BLE_GAP_EVTS_BLE_GAP_EVT_RSSI_CHANGED
        // BLE_GAP_EVTS_BLE_GAP_EVT_SCAN_REQ_REPORT
        // BLE_GAP_EVTS_BLE_GAP_EVT_QOS_CHANNEL_SURVEY_REPORT
        _ => {}
    }
}

#[cfg(any(feature = "s113", feature = "s132", feature = "s140"))]
pub(crate) unsafe fn do_data_length_update(conn_handle: u16, params: *const raw::ble_gap_data_length_params_t) {
    let mut dl_limitation = core::mem::zeroed();
    let ret = raw::sd_ble_gap_data_length_update(conn_handle, params, &mut dl_limitation);
    if let Err(_err) = RawError::convert(ret) {
        warn!("sd_ble_gap_data_length_update err {:?}", _err);

        if dl_limitation.tx_payload_limited_octets != 0 || dl_limitation.rx_payload_limited_octets != 0 {
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
