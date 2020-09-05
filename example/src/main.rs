#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use nrf52840_hal as _;
use panic_probe as _;
use static_executor_cortex_m as _;

use async_flash::Flash;
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
    let mut adv_handle: u8 = sd::BLE_GAP_ADV_SET_HANDLE_NOT_SET as u8;

    let mut adv_params: sd::ble_gap_adv_params_t = unsafe { core::mem::zeroed() };
    adv_params.properties.type_ = sd::BLE_GAP_ADV_TYPE_CONNECTABLE_SCANNABLE_UNDIRECTED as u8;
    adv_params.primary_phy = sd::BLE_GAP_PHY_1MBPS as u8;
    adv_params.secondary_phy = sd::BLE_GAP_PHY_1MBPS as u8;
    adv_params.duration = sd::BLE_GAP_ADV_TIMEOUT_GENERAL_UNLIMITED as u16;
    adv_params.interval = 100;

    #[rustfmt::skip]
    let adv = &mut [
        0x02, 0x01, sd::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
        0x03, 0x03, 0x09, 0x18,
        0x06, 0x09, b'H', b'e', b'l', b'l', b'o',
    ];
    #[rustfmt::skip]
    let sr = &mut [
        0x03, 0x03, 0x09, 0x18,
    ];

    let adv_data = sd::ble_gap_adv_data_t {
        adv_data: sd::ble_data_t {
            p_data: adv.as_mut_ptr(),
            len: adv.len() as u16,
        },
        scan_rsp_data: sd::ble_data_t {
            p_data: sr.as_mut_ptr(),
            len: sr.len() as u16,
        },
    };

    let ret = unsafe {
        sd::sd_ble_gap_adv_set_configure(&mut adv_handle as _, &adv_data as _, &adv_params as _)
    };

    match Error::convert(ret) {
        Ok(()) => info!("advertising configured!"),
        Err(err) => depanic!("sd_ble_gap_adv_set_configure err {:?}", err),
    }

    let ret = unsafe { sd::sd_ble_gap_adv_start(adv_handle, sd::BLE_CONN_CFG_TAG_DEFAULT as u8) };
    match Error::convert(ret) {
        Ok(()) => info!("advertising started!"),
        Err(err) => depanic!("sd_ble_gap_adv_start err {:?}", err),
    }

    // The structs above need to be kept alive for the entire duration of the advertising procedure.
    // For now just wait here forever.

    futures::future::pending::<()>().await;
}

#[entry]
fn main() -> ! {
    info!("Hello World!");

    info!("enabling softdevice");
    unsafe { nrf_softdevice::enable() }
    info!("softdevice enabled");

    unsafe {
        softdevice_task.spawn().unwrap();
        interrupt_task.spawn().unwrap();
        flash_task.spawn().unwrap();
        bluetooth_task.spawn().unwrap();

        static_executor::run();
    }
}
