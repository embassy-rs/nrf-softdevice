#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use cortex_m_rt::entry;
use defmt::*;
use embassy_executor::executor::Executor;
use embassy_util::Forever;
use embedded_storage_async::nor_flash::*;
use futures::pin_mut;
use nrf_softdevice::{Flash, Softdevice};

static EXECUTOR: Forever<Executor> = Forever::new();

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

#[embassy_executor::task]
async fn flash_task(sd: &'static Softdevice) {
    let f = Flash::take(sd);
    pin_mut!(f);

    info!("starting erase");
    unwrap!(f.as_mut().erase(0x80000, 0x81000).await);
    info!("erased!");

    info!("starting write");
    unwrap!(f.as_mut().write(0x80000, &[1, 2, 3, 4]).await);
    info!("write done!");
}

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let sd = Softdevice::enable(&Default::default());

    let executor = EXECUTOR.put(Executor::new());
    executor.run(move |spawner| {
        unwrap!(spawner.spawn(softdevice_task(sd)));
        unwrap!(spawner.spawn(flash_task(sd)));
    });
}
