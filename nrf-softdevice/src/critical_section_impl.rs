use core::arch::asm;
use core::sync::atomic::{compiler_fence, AtomicBool, Ordering};

use crate::pac::{Interrupt, NVIC};

#[cfg(any(feature = "nrf52810", feature = "nrf52811"))]
const RESERVED_IRQS: u32 = (1 << (Interrupt::POWER_CLOCK as u8))
    | (1 << (Interrupt::RADIO as u8))
    | (1 << (Interrupt::RTC0 as u8))
    | (1 << (Interrupt::TIMER0 as u8))
    | (1 << (Interrupt::RNG as u8))
    | (1 << (Interrupt::ECB as u8))
    | (1 << (Interrupt::CCM_AAR as u8))
    | (1 << (Interrupt::TEMP as u8))
    | (1 << (Interrupt::SWI5 as u8));

#[cfg(not(any(feature = "nrf52810", feature = "nrf52811")))]
const RESERVED_IRQS: u32 = (1 << (Interrupt::POWER_CLOCK as u8))
    | (1 << (Interrupt::RADIO as u8))
    | (1 << (Interrupt::RTC0 as u8))
    | (1 << (Interrupt::TIMER0 as u8))
    | (1 << (Interrupt::RNG as u8))
    | (1 << (Interrupt::ECB as u8))
    | (1 << (Interrupt::CCM_AAR as u8))
    | (1 << (Interrupt::TEMP as u8))
    | (1 << (Interrupt::SWI5_EGU5 as u8));

static CS_FLAG: AtomicBool = AtomicBool::new(false);
static mut CS_MASK: u32 = 0;

#[inline]
unsafe fn raw_critical_section<R>(f: impl FnOnce() -> R) -> R {
    // TODO: assert that we're in privileged level
    // Needed because disabling irqs in non-privileged level is a noop, which would break safety.

    let primask: u32;
    asm!("mrs {}, PRIMASK", out(reg) primask);

    asm!("cpsid i");

    // Prevent compiler from reordering operations inside/outside the critical section.
    compiler_fence(Ordering::SeqCst);

    let r = f();

    compiler_fence(Ordering::SeqCst);

    if primask & 1 == 0 {
        asm!("cpsie i");
    }

    r
}

struct CriticalSection;
critical_section_1::set_impl!(CriticalSection);
critical_section_02::custom_impl!(CriticalSection);

unsafe impl critical_section_1::Impl for CriticalSection {
    unsafe fn acquire() -> bool {
        let nvic = &*NVIC::PTR;
        let nested_cs = CS_FLAG.load(Ordering::SeqCst);

        if !nested_cs {
            raw_critical_section(|| {
                CS_FLAG.store(true, Ordering::Relaxed);

                // Store the state of irqs.
                CS_MASK = nvic.icer[0].read();

                // Disable only not-reserved irqs.
                nvic.icer[0].write(!RESERVED_IRQS);
            });
        }

        compiler_fence(Ordering::SeqCst);

        nested_cs
    }

    unsafe fn release(nested_cs: bool) {
        compiler_fence(Ordering::SeqCst);

        let nvic = &*NVIC::PTR;
        if !nested_cs {
            raw_critical_section(|| {
                CS_FLAG.store(false, Ordering::Relaxed);
                // restore only non-reserved irqs.
                nvic.iser[0].write(CS_MASK & !RESERVED_IRQS);
            });
        }
    }
}

unsafe impl critical_section_02::Impl for CriticalSection {
    unsafe fn acquire() -> u8 {
        <Self as critical_section_1::Impl>::acquire() as _
    }

    unsafe fn release(token: u8) {
        <Self as critical_section_1::Impl>::release(token != 0)
    }
}
