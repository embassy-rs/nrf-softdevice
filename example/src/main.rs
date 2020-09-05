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
use nrf_softdevice as sd;

use sd::interrupt;

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
    sd::run().await;
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
    let mut f = unsafe { sd::Flash::new() };
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

#[entry]
fn main() -> ! {
    info!("Hello World!");

    info!("enabling softdevice");
    unsafe { sd::enable() }
    info!("softdevice enabled");

    unsafe {
        softdevice_task.spawn().unwrap();
        interrupt_task.spawn().unwrap();
        flash_task.spawn().unwrap();

        static_executor::run();
    }
}
