#![macro_use]

use nrf_softdevice_defmt_rtt as _; // global logger
use panic_probe as _;

use embassy_nrf as _;

use alloc_cortex_m::CortexMHeap;
use core::alloc::Layout;
use core::sync::atomic::{AtomicUsize, Ordering};
use defmt::panic;

// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// define what happens in an Out Of Memory (OOM) condition
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    panic!("Alloc error");
}

defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        // NOTE(no-CAS) `timestamps` runs with interrupts disabled
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}
