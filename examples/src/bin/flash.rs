#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

#[path = "../example_common.rs"]
mod example_common;
use example_common::*;

use anyfmt::{panic, *};
use cortex_m_rt::entry;
use embassy::executor::{task, Executor};
use embassy::flash::Flash as _;
use embassy::util::Forever;

use nrf_softdevice::{Flash, Softdevice};

static EXECUTOR: Forever<Executor> = Forever::new();

#[task]
async fn softdevice_task(sd: &'static Softdevice) {
    sd.run().await;
}

#[task]
async fn flash_task(sd: &'static Softdevice) {
    let mut f = Flash::take(sd);

    info!("starting erase");
    unwrap!(f.erase(0x80000).await);
    info!("erased!");

    info!("starting write");
    unwrap!(f.write(0x80000, &[1, 2, 3, 4]).await);
    info!("write done!");
}

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let (sdp, p) = take_peripherals();
    let sd = Softdevice::enable(sdp, &Default::default());

    let executor = EXECUTOR.put(Executor::new(cortex_m::asm::sev));
    unwrap!(executor.spawn(softdevice_task(sd)));
    unwrap!(executor.spawn(flash_task(sd)));

    loop {
        executor.run();
        cortex_m::asm::wfe();
    }
}
