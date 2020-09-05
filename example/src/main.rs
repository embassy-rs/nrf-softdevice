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
        flash_task.spawn().unwrap();

        static_executor::run();
    }
}
