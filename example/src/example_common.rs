#![macro_use]

use defmt_rtt as _; // global logger
use nrf52840_hal as _;
use panic_probe as _;
use static_executor_cortex_m as _;

pub use defmt::info;

use core::sync::atomic::{AtomicUsize, Ordering};

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

pub trait Dewrap<T> {
    fn dewrap(self) -> T;
}

impl<T, E: defmt::Format> Dewrap<T> for Result<T, E> {
    fn dewrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => depanic!("dewrap failed: {:?}", e),
        }
    }
}
