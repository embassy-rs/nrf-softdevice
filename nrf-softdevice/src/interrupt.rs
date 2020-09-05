use bare_metal::CriticalSection;
use core::sync::atomic::{compiler_fence, AtomicBool, Ordering};
use cortex_m::interrupt::InterruptNumber;

use crate::pac::{Interrupt, NVIC, NVIC_PRIO_BITS};

const RESERVED_IRQS: [u32; 2] = [
    (1 << (Interrupt::POWER_CLOCK as u8))
        | (1 << (Interrupt::RADIO as u8))
        | (1 << (Interrupt::RTC0 as u8))
        | (1 << (Interrupt::TIMER0 as u8))
        | (1 << (Interrupt::RNG as u8))
        | (1 << (Interrupt::ECB as u8))
        | (1 << (Interrupt::CCM_AAR as u8))
        | (1 << (Interrupt::TEMP as u8))
        | (1 << (Interrupt::SWI5_EGU5 as u8)),
    0,
];

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum Priority {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
    Level5 = 5,
    Level6 = 6,
    Level7 = 7,
}

impl Priority {
    #[inline]
    fn to_nvic(self) -> u8 {
        (self as u8) << (8 - NVIC_PRIO_BITS)
    }

    #[inline]
    fn from_nvic(priority: u8) -> Self {
        match priority >> (8 - NVIC_PRIO_BITS) {
            0 => Self::Level0,
            1 => Self::Level1,
            2 => Self::Level2,
            3 => Self::Level3,
            4 => Self::Level4,
            5 => Self::Level5,
            6 => Self::Level6,
            7 => Self::Level7,
            _ => unreachable!(),
        }
    }
}

static mut CS_FLAG: AtomicBool = AtomicBool::new(false);
static mut CS_MASK: [u32; 2] = [0; 2];

#[inline]
pub(crate) unsafe fn raw_free<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
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

/// Execute closure `f` in an interrupt-free context.
///
/// This as also known as a "critical section".
#[inline]
pub fn free<F, R>(f: F) -> R
where
    F: FnOnce(&CriticalSection) -> R,
{
    unsafe {
        // TODO: assert that we're in privileged level
        // Needed because disabling irqs in non-privileged level is a noop, which would break safety.

        let nvic = &*NVIC::ptr();

        let nested_cs = CS_FLAG.load(Ordering::SeqCst);

        if !nested_cs {
            raw_free(|| {
                CS_FLAG.store(true, Ordering::Relaxed);

                // Store the state of irqs.
                CS_MASK[0] = nvic.icer[0].read();
                CS_MASK[1] = nvic.icer[1].read();

                // Disable only not-reserved irqs.
                nvic.icer[0].write(!RESERVED_IRQS[0]);
                nvic.icer[1].write(!RESERVED_IRQS[1]);
            });
        }

        let r = f(&CriticalSection::new());

        if !nested_cs {
            raw_free(|| {
                CS_FLAG.store(false, Ordering::Relaxed);
                // restore only non-reserved irqs.
                nvic.iser[0].write(CS_MASK[0] & !RESERVED_IRQS[0]);
                nvic.iser[1].write(CS_MASK[1] & !RESERVED_IRQS[1]);
            });
        }

        r
    }
}

#[inline]
fn is_app_accessible_irq(irq: Interrupt) -> bool {
    match irq {
        Interrupt::POWER_CLOCK
        | Interrupt::RADIO
        | Interrupt::RTC0
        | Interrupt::TIMER0
        | Interrupt::RNG
        | Interrupt::ECB
        | Interrupt::CCM_AAR
        | Interrupt::TEMP
        | Interrupt::SWI5_EGU5 => false,
        _ => true,
    }
}

#[inline]
fn is_app_accessible_priority(priority: Priority) -> bool {
    match priority {
        Priority::Level0 | Priority::Level1 | Priority::Level4 => false,
        _ => true,
    }
}

#[inline]
pub fn unmask(irq: Interrupt) {
    assert!(is_app_accessible_irq(irq));
    assert!(is_app_accessible_priority(get_priority(irq)));

    unsafe {
        if CS_FLAG.load(Ordering::SeqCst) {
            let nr = irq.number();
            CS_MASK[usize::from(nr / 32)] |= 1 << (nr % 32);
        } else {
            NVIC::unmask(irq);
        }
    }
}

#[inline]
pub fn mask(irq: Interrupt) {
    assert!(is_app_accessible_irq(irq));

    unsafe {
        if CS_FLAG.load(Ordering::SeqCst) {
            let nr = irq.number();
            CS_MASK[usize::from(nr / 32)] &= !(1 << (nr % 32));
        } else {
            NVIC::mask(irq);
        }
    }
}

#[inline]
pub fn is_active(irq: Interrupt) -> bool {
    assert!(is_app_accessible_irq(irq));
    NVIC::is_active(irq)
}

#[inline]
pub fn is_enabled(irq: Interrupt) -> bool {
    assert!(is_app_accessible_irq(irq));
    NVIC::is_enabled(irq)
}

#[inline]
pub fn is_pending(irq: Interrupt) -> bool {
    assert!(is_app_accessible_irq(irq));
    NVIC::is_pending(irq)
}

#[inline]
pub fn pend(irq: Interrupt) {
    assert!(is_app_accessible_irq(irq));
    NVIC::pend(irq)
}

#[inline]
pub fn unpend(irq: Interrupt) {
    assert!(is_app_accessible_irq(irq));
    NVIC::unpend(irq)
}

#[inline]
pub fn get_priority(irq: Interrupt) -> Priority {
    Priority::from_nvic(NVIC::get_priority(irq))
}

#[inline]
pub fn set_priority(irq: Interrupt, prio: Priority) {
    assert!(is_app_accessible_irq(irq));
    assert!(is_app_accessible_priority(prio));
    unsafe {
        cortex_m::peripheral::Peripherals::steal()
            .NVIC
            .set_priority(irq, prio.to_nvic())
    }
}
