#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use async_flash::Flash;
use cortex_m_rt::entry;

#[static_executor::task]
async fn softdevice_task() {
    nrf_softdevice::run().await;
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

#[entry]
fn main() -> ! {
    info!("Hello World!");

    unsafe {
        nrf_softdevice::enable(&Default::default());

        softdevice_task.spawn().dewrap();
        flash_task.spawn().dewrap();

        static_executor::run();
    }
}
