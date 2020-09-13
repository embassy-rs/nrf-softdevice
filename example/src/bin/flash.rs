#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use async_flash::Flash;
use cortex_m_rt::entry;
use nrf_softdevice::Softdevice;

#[static_executor::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

#[static_executor::task]
async fn flash_task(sd: &'static Softdevice) {
    let mut f = sd.take_flash();

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

    let sd = Softdevice::enable(&Default::default());

    unsafe {
        softdevice_task.spawn(sd).dewrap();
        flash_task.spawn(sd).dewrap();

        static_executor::run();
    }
}
