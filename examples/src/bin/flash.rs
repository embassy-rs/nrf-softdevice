#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;

use defmt::*;
use embassy_executor::Spawner;
use embedded_storage_async::nor_flash::*;
use futures::pin_mut;
use nrf_softdevice::{Flash, Softdevice};

#[embassy_executor::task]
async fn softdevice_task(sd: &'static Softdevice) -> ! {
    sd.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let sd = Softdevice::enable(&Default::default());
    unwrap!(spawner.spawn(softdevice_task(sd)));

    let f = Flash::take(sd);
    pin_mut!(f);

    info!("starting erase");
    unwrap!(f.as_mut().erase(0x80000, 0x81000).await);
    info!("erased!");

    info!("starting write");
    unwrap!(f.as_mut().write(0x80000, &[1, 2, 3, 4]).await);
    info!("write done!");
}
