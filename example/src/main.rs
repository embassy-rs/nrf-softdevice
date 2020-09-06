#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use nrf52840_hal as _;
use panic_probe as _;
use static_executor_cortex_m as _;

use async_flash::Flash;
use core::mem;
use core::sync::atomic::{AtomicUsize, Ordering};
use cortex_m_rt::entry;
use defmt::info;
use nrf_softdevice::Error;
use nrf_softdevice_s140 as sd;

use nrf_softdevice::interrupt;

#[defmt::timestamp]
fn timestamp() -> u64 {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n as u64
}

macro_rules! depanic {
    ($( $i:expr ),*) => {
        {
            defmt::error!($( $i ),*);
            panic!();
        }
    }
}

#[static_executor::task]
async fn softdevice_task() {
    nrf_softdevice::run().await;
}

#[static_executor::task]
async fn interrupt_task() {
    let enabled = interrupt::is_enabled(interrupt::SWI0_EGU0);
    info!("enabled: {:?}", enabled);

    // This would panic with "irq RADIO is reserved for the softdevice"
    // interrupt::set_priority(interrupt::RADIO, interrupt::Priority::Level7);

    // This would panic with "priority level Level1 is reserved for the softdevice"
    // interrupt::set_priority(interrupt::SWI0_EGU0, interrupt::Priority::Level1);

    // This would panic with "irq SWI0_EGU0 has priority Level0 which is reserved for the softdevice. Set another prority before enabling it.""
    // interrupt::enable(interrupt::SWI0_EGU0);

    // If we set a non-reserved priority first, we can enable the interrupt
    interrupt::set_priority(interrupt::SWI0_EGU0, interrupt::Priority::Level7);
    interrupt::enable(interrupt::SWI0_EGU0);

    // Now it's enabled
    let enabled = interrupt::is_enabled(interrupt::SWI0_EGU0);
    info!("enabled: {:?}", enabled);

    // The interrupt will trigger instantly
    info!("before pend");
    interrupt::pend(interrupt::SWI0_EGU0);
    info!("after pend");

    interrupt::free(|_| {
        info!("Hello from critical section!");

        // The interrupt will trigger at end of critical section
        info!("before pend");
        interrupt::pend(interrupt::SWI0_EGU0);
        info!("after pend");

        // This will print true even if we're inside a critical section
        // because it reads a "backup" of the irq enabled registers.
        let enabled = interrupt::is_enabled(interrupt::SWI0_EGU0);
        info!("enabled: {:?}", enabled);

        // You can also enable/disable interrupts inside a critical section
        // They don't take effect until exiting the critical section, so it's safe.
        // (they modify the "backup" register which gets restored on CS exit)
        interrupt::set_priority(interrupt::SWI1_EGU1, interrupt::Priority::Level6);
        interrupt::enable(interrupt::SWI1_EGU1);
        interrupt::pend(interrupt::SWI1_EGU1);

        info!("exiting critical section...");
    });

    info!("exited critical section");
}

#[interrupt]
fn SWI0_EGU0() {
    info!("SWI0_EGU0 triggered!")
}

#[interrupt]
fn SWI1_EGU1() {
    info!("SWI1_EGU1 triggered!")
}

#[static_executor::task]
async fn flash_task() {
    let mut f = unsafe { nrf_softdevice::Flash::new() };
    info!("starting erase");
    match f.erase(0x80000).await {
        Ok(()) => info!("erased!"),
        Err(e) => depanic!("erase failed: {:?}", e),
    }

    info!("starting write");
    match f.write(0x80000, &[1, 2, 3, 4]).await {
        Ok(()) => info!("write done!"),
        Err(e) => depanic!("write failed: {:?}", e),
    }
}

#[static_executor::task]
async fn bluetooth_task() {
    #[rustfmt::skip]
    let adv_data = &mut [
        0x02, 0x01, sd::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x09, 0x18,
        0x0a, 0x09, b'H', b'e', b'l', b'l', b'o', b'R', b'u', b's', b't',
    ];
    #[rustfmt::skip]
    let scan_data = &mut [
        0x03, 0x03, 0x09, 0x18,
    ];

    loop {
        nrf_softdevice::gap::advertise(
            nrf_softdevice::gap::ConnectableAdvertisement::ScannableUndirected {
                adv_data,
                scan_data,
            },
        )
        .await;

        info!("advertising done");
    }
}

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let config = nrf_softdevice::Config {
        clock: Some(sd::nrf_clock_lf_cfg_t {
            source: sd::NRF_CLOCK_LF_SRC_XTAL as u8,
            rc_ctiv: 0,
            rc_temp_ctiv: 0,
            accuracy: 7,
        }),
        conn_gap: Some(sd::ble_gap_conn_cfg_t {
            conn_count: 20,
            event_length: 6,
        }),
        conn_gatt: Some(sd::ble_gatt_conn_cfg_t { att_mtu: 128 }),
        gap_role_count: Some(sd::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 20,
            central_role_count: 0,
            central_sec_count: 0,
            _bitfield_1: sd::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(sd::ble_gap_cfg_device_name_t {
            p_value: b"HelloRust" as *const u8 as _,
            current_len: 9,
            max_len: 9,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: sd::ble_gap_cfg_device_name_t::new_bitfield_1(
                sd::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    };

    unsafe { nrf_softdevice::enable(&config) }

    unsafe {
        softdevice_task.spawn().unwrap();
        interrupt_task.spawn().unwrap();
        flash_task.spawn().unwrap();
        bluetooth_task.spawn().unwrap();

        static_executor::run();
    }
}
