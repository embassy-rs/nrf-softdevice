use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};

use cortex_m::peripheral::NVIC;

use crate::{raw, Interrupt, RawError, SocEvent};

unsafe extern "C" fn fault_handler(id: u32, pc: u32, info: u32) {
    match (id, info) {
        (raw::NRF_FAULT_ID_SD_ASSERT, _) => panic!(
            "Softdevice assertion failed: an assertion inside the softdevice's code has failed. Most common cause is disabling interrupts for too long. Make sure you're using nrf_softdevice::interrupt::free instead of cortex_m::interrupt::free, which disables non-softdevice interrupts only. PC={:x}",
            pc
        ),
        (raw::NRF_FAULT_ID_APP_MEMACC, 0) => panic!(
            "Softdevice memory access violation. Your program accessed RAM reserved to the softdevice. PC={:x}",
            pc
        ),
        (raw::NRF_FAULT_ID_APP_MEMACC, _) => panic!(
            "Softdevice memory access violation. Your program accessed registers for a peripheral reserved to the softdevice. PC={:x} PREGION={:?}",
            pc, info
        ),
        _ => panic!(
            "Softdevice unknown fault id={:?} pc={:x} info={:?}",
            id, pc, info
        ),
    }
}

/// Singleton instance of the enabled softdevice.
///
/// The `Softdevice` instance can be obtaind by enabling it with [`Softdevice::enable`]. Once
/// enabled, it can be used to establish Bluetooth connections with [`ble::central`] and [`ble::peripheral`].
///
/// Disabling the softdevice is not supported due to the complexity of a safe implementation. Consider resetting the CPU instead.
pub struct Softdevice {
    // Prevent Send, Sync
    _private: PhantomData<*mut ()>,
    #[cfg(feature = "ble-gatt")]
    #[allow(unused)]
    pub(crate) att_mtu: u16,
    #[cfg(feature = "ble-l2cap")]
    pub(crate) l2cap_rx_mps: u16,
}

/// Softdevice configuration.
///
/// Fields set to None will use a default configuration.
#[derive(Default)]
pub struct Config {
    pub clock: Option<raw::nrf_clock_lf_cfg_t>,
    pub conn_gap: Option<raw::ble_gap_conn_cfg_t>,
    pub conn_gattc: Option<raw::ble_gattc_conn_cfg_t>,
    pub conn_gatts: Option<raw::ble_gatts_conn_cfg_t>,
    pub conn_gatt: Option<raw::ble_gatt_conn_cfg_t>,
    #[cfg(feature = "ble-l2cap")]
    pub conn_l2cap: Option<raw::ble_l2cap_conn_cfg_t>,
    pub common_vs_uuid: Option<raw::ble_common_cfg_vs_uuid_t>,
    pub gap_role_count: Option<raw::ble_gap_cfg_role_count_t>,
    pub gap_device_name: Option<raw::ble_gap_cfg_device_name_t>,
    pub gap_ppcp_incl: Option<raw::ble_gap_cfg_ppcp_incl_cfg_t>,
    pub gap_car_incl: Option<raw::ble_gap_cfg_car_incl_cfg_t>,
    pub gatts_service_changed: Option<raw::ble_gatts_cfg_service_changed_t>,
    pub gatts_attr_tab_size: Option<raw::ble_gatts_cfg_attr_tab_size_t>,
}

const APP_CONN_CFG_TAG: u8 = 1;

fn get_app_ram_base() -> u32 {
    extern "C" {
        static mut __sdata: u32;
    }

    ptr::addr_of!(__sdata) as u32
}

fn cfg_set(id: u32, cfg: &raw::ble_cfg_t) {
    let app_ram_base = get_app_ram_base();
    let ret = unsafe { raw::sd_ble_cfg_set(id, cfg, app_ram_base) };
    match RawError::convert(ret) {
        Ok(()) => {}
        Err(RawError::NoMem) => {}
        Err(err) => panic!("sd_ble_cfg_set {:?} err {:?}", id, err),
    }
}

static ENABLED: AtomicBool = AtomicBool::new(false);
static mut SOFTDEVICE: MaybeUninit<Softdevice> = MaybeUninit::uninit();

impl Softdevice {
    /// Enable the softdevice.
    ///
    /// # Panics
    /// - Panics if the requested configuration requires more memory than reserved for the softdevice. In that case, you can give more memory to the softdevice by editing the RAM start address in `memory.x`. The required start address is logged prior to panic.
    /// - Panics if the requested configuration has too high memory requirements for the softdevice. The softdevice supports a maximum dynamic memory size of 64kb.
    /// - Panics if called multiple times. Must be called at most once.
    pub fn enable(config: &Config) -> &'static mut Softdevice {
        if ENABLED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            panic!("nrf_softdevice::enable() called multiple times.")
        }

        let p_clock_lf_cfg = config.clock.as_ref().map(|x| x as _).unwrap_or(ptr::null());
        let ret = unsafe { raw::sd_softdevice_enable(p_clock_lf_cfg, Some(fault_handler)) };
        match RawError::convert(ret) {
            Ok(()) => {}
            Err(err) => panic!("sd_softdevice_enable err {:?}", err),
        }

        let app_ram_base = get_app_ram_base();

        // Set at least one GAP config so conn_cfg_tag 1 (APP_CONN_CFG_TAG) is usable.
        // If you set none, it seems the softdevice won't let you use it, requiring a conn_cfg_tag of 0 (raw::BLE_CONN_CFG_TAG_DEFAULT) instead.
        let val = config.conn_gap.unwrap_or(raw::ble_gap_conn_cfg_t {
            conn_count: raw::BLE_GAP_CONN_COUNT_DEFAULT as u8,
            event_length: raw::BLE_GAP_EVENT_LENGTH_DEFAULT as u16,
        });
        cfg_set(
            raw::BLE_CONN_CFGS_BLE_CONN_CFG_GAP,
            &raw::ble_cfg_t {
                conn_cfg: raw::ble_conn_cfg_t {
                    conn_cfg_tag: APP_CONN_CFG_TAG,
                    params: raw::ble_conn_cfg_t__bindgen_ty_1 { gap_conn_cfg: val },
                },
            },
        );

        if let Some(val) = config.conn_gatt {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATT,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 { gatt_conn_cfg: val },
                    },
                },
            );
        }

        if let Some(val) = config.conn_gattc {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATTC,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 { gattc_conn_cfg: val },
                    },
                },
            );
        }

        if let Some(val) = config.conn_gatts {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_GATTS,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 { gatts_conn_cfg: val },
                    },
                },
            );
        }

        #[cfg(feature = "ble-l2cap")]
        if let Some(val) = config.conn_l2cap {
            cfg_set(
                raw::BLE_CONN_CFGS_BLE_CONN_CFG_L2CAP,
                &raw::ble_cfg_t {
                    conn_cfg: raw::ble_conn_cfg_t {
                        conn_cfg_tag: APP_CONN_CFG_TAG,
                        params: raw::ble_conn_cfg_t__bindgen_ty_1 { l2cap_conn_cfg: val },
                    },
                },
            );
        }

        if let Some(val) = config.common_vs_uuid {
            cfg_set(
                raw::BLE_COMMON_CFGS_BLE_COMMON_CFG_VS_UUID,
                &raw::ble_cfg_t {
                    common_cfg: raw::ble_common_cfg_t { vs_uuid_cfg: val },
                },
            );
        }

        if let Some(val) = config.gap_role_count {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_ROLE_COUNT,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t { role_count_cfg: val },
                },
            );
        }

        if let Some(val) = config.gap_device_name {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_DEVICE_NAME,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t { device_name_cfg: val },
                },
            );
        }

        if let Some(val) = config.gap_ppcp_incl {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_PPCP_INCL_CONFIG,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t { ppcp_include_cfg: val },
                },
            );
        }

        if let Some(val) = config.gap_car_incl {
            cfg_set(
                raw::BLE_GAP_CFGS_BLE_GAP_CFG_CAR_INCL_CONFIG,
                &raw::ble_cfg_t {
                    gap_cfg: raw::ble_gap_cfg_t { car_include_cfg: val },
                },
            );
        }
        if let Some(val) = config.gatts_service_changed {
            cfg_set(
                raw::BLE_GATTS_CFGS_BLE_GATTS_CFG_SERVICE_CHANGED,
                &raw::ble_cfg_t {
                    gatts_cfg: raw::ble_gatts_cfg_t { service_changed: val },
                },
            );
        }
        if let Some(val) = config.gatts_attr_tab_size {
            cfg_set(
                raw::BLE_GATTS_CFGS_BLE_GATTS_CFG_ATTR_TAB_SIZE,
                &raw::ble_cfg_t {
                    gatts_cfg: raw::ble_gatts_cfg_t { attr_tab_size: val },
                },
            );
        }

        let mut wanted_app_ram_base = app_ram_base;
        let ret = unsafe { raw::sd_ble_enable(&mut wanted_app_ram_base as _) };
        info!("softdevice RAM: {:?} bytes", wanted_app_ram_base - 0x20000000);
        match RawError::convert(ret) {
            Ok(()) => {}
            Err(RawError::NoMem) => {
                if wanted_app_ram_base <= app_ram_base {
                    panic!("selected configuration has too high RAM requirements.")
                } else {
                    panic!(
                        "too little RAM for softdevice. Change your app's RAM start address to {:x}",
                        wanted_app_ram_base
                    );
                }
            }
            Err(err) => panic!("sd_ble_enable err {:?}", err),
        }

        if wanted_app_ram_base < app_ram_base {
            warn!("You're giving more RAM to the softdevice than needed. You can change your app's RAM start address to {:x}", wanted_app_ram_base);
        }

        unsafe {
            NVIC::unmask(Interrupt::SWI2_EGU2);
        }

        #[cfg(feature = "ble-gatt")]
        let att_mtu = config
            .conn_gatt
            .map(|x| x.att_mtu)
            .unwrap_or(raw::BLE_GATT_ATT_MTU_DEFAULT as u16);

        #[cfg(feature = "ble-l2cap")]
        let l2cap_rx_mps = config
            .conn_l2cap
            .map(|x| x.rx_mps)
            .unwrap_or(raw::BLE_L2CAP_MPS_MIN as u16);

        let sd = Softdevice {
            _private: PhantomData,

            #[cfg(feature = "ble-gatt")]
            att_mtu,

            #[cfg(feature = "ble-l2cap")]
            l2cap_rx_mps,
        };

        unsafe {
            let p = (&mut *(&raw mut SOFTDEVICE)).as_mut_ptr();
            p.write(sd);
            &mut *p
        }
    }

    /// Return an instance to the softdevice without checking whether
    /// it is enabled or not. This is only safe if the softdevice is enabled
    /// (a call to [`enable`] has returned without error) and no `&mut` references
    /// to the softdevice are active
    pub unsafe fn steal() -> &'static Softdevice {
        &*(&*(&raw const SOFTDEVICE)).as_ptr()
    }

    /// Runs the softdevice event handling loop.
    ///
    /// It must be called in its own async task after enabling the softdevice
    /// and before doing any operation. Failure to doing so will cause async operations to never finish.
    pub async fn run(&self) -> ! {
        self.run_with_callback(|_| ()).await
    }

    /// Runs the softdevice event handling loop with a callback for [`SocEvent`]s.
    ///
    /// It must be called under the same conditions as [`Softdevice::run()`]. This
    /// version allows the application to provide a callback to receive SoC events
    /// from the softdevice (other than flash events which are handled by [`Flash`](crate::flash::Flash)).
    pub async fn run_with_callback<F: FnMut(SocEvent)>(&self, f: F) -> ! {
        embassy_futures::join::join(self.run_ble(), crate::events::run_soc(f)).await;
        // Should never get here
        loop {}
    }

    /// Runs the softdevice soc event handler only.
    ///
    /// It must be called under the same conditions as [`Softdevice::run()`].
    pub async fn run_soc(&self) -> ! {
        crate::events::run_soc(|_| ()).await
    }

    /// Runs the softdevice ble event handler only.
    ///
    /// It must be called under the same conditions as [`Softdevice::run()`].
    pub async fn run_ble(&self) -> ! {
        crate::events::run_ble().await
    }
}
