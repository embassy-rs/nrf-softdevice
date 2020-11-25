//! Safe interrupt management
//!
//! This module implements functions to manage interrupts that panic when trying
//! to use softdevice-reserved interrupts and priority levels. Therefore,
//! they're safe to use in all situations.
//!
//! You must NOT use any other crate to manage interrupts, such as `cortex-m`'s `NVIC`.

use crate::pac::{NVIC, NVIC_PRIO_BITS};
use crate::util::{assert, unreachable, *};
use core::sync::atomic::{compiler_fence, AtomicBool, Ordering};

// Re-exports
pub use crate::pac::Interrupt;
pub use crate::pac::Interrupt::*; // needed for cortex-m-rt #[interrupt]
pub use cortex_m::interrupt::{CriticalSection, Mutex, Nr};

#[cfg(any(feature = "nrf52810", feature = "nrf52811"))]
const RESERVED_IRQS: [u32; 2] = [
    (1 << (Interrupt::POWER_CLOCK as u8))
        | (1 << (Interrupt::RADIO as u8))
        | (1 << (Interrupt::RTC0 as u8))
        | (1 << (Interrupt::TIMER0 as u8))
        | (1 << (Interrupt::RNG as u8))
        | (1 << (Interrupt::ECB as u8))
        | (1 << (Interrupt::CCM_AAR as u8))
        | (1 << (Interrupt::TEMP as u8))
        | (1 << (Interrupt::SWI5 as u8)),
    0,
];

#[cfg(not(any(feature = "nrf52810", feature = "nrf52811")))]
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

#[derive(defmt::Format, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

static CS_FLAG: AtomicBool = AtomicBool::new(false);
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
        let nvic = &*NVIC::ptr();

        let token = disable_all();
        let r = f(&CriticalSection::new());
        enable_all(token);
        r
    }
}

pub unsafe fn disable_all() -> u8 {
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

    compiler_fence(Ordering::SeqCst);

    return nested_cs as u8;
}

pub unsafe fn enable_all(token: u8) {
    compiler_fence(Ordering::SeqCst);

    let nvic = &*NVIC::ptr();
    if token == 0 {
        raw_free(|| {
            CS_FLAG.store(false, Ordering::Relaxed);
            // restore only non-reserved irqs.
            nvic.iser[0].write(CS_MASK[0] & !RESERVED_IRQS[0]);
            nvic.iser[1].write(CS_MASK[1] & !RESERVED_IRQS[1]);
        });
    }
}

#[inline]
fn is_app_accessible_irq(irq: Interrupt) -> bool {
    let nr = irq.nr();
    (RESERVED_IRQS[usize::from(nr / 32)] & 1 << (nr % 32)) == 0
}

#[inline]
fn is_app_accessible_priority(priority: Priority) -> bool {
    match priority {
        Priority::Level0 | Priority::Level1 | Priority::Level4 => false,
        _ => true,
    }
}

macro_rules! assert_app_accessible_irq {
    ($irq:ident) => {
        assert!(
            is_app_accessible_irq($irq),
            "irq {:istr} is reserved for the softdevice",
            irq_str($irq)
        );
    };
}

#[inline]
pub fn enable(irq: Interrupt) {
    assert_app_accessible_irq!(irq);
    let prio = get_priority(irq);
    assert!(
        is_app_accessible_priority(prio),
        "irq {:istr} has priority {:?} which is reserved for the softdevice. Set another prority before enabling it.",
        irq_str(irq),
        prio
    );

    unsafe {
        if CS_FLAG.load(Ordering::SeqCst) {
            let nr = irq.nr();
            CS_MASK[usize::from(nr / 32)] |= 1 << (nr % 32);
        } else {
            NVIC::unmask(irq);
        }
    }
}

#[inline]
pub fn disable(irq: Interrupt) {
    assert_app_accessible_irq!(irq);

    unsafe {
        if CS_FLAG.load(Ordering::SeqCst) {
            let nr = irq.nr();
            CS_MASK[usize::from(nr / 32)] &= !(1 << (nr % 32));
        } else {
            NVIC::mask(irq);
        }
    }
}

#[inline]
pub fn is_active(irq: Interrupt) -> bool {
    assert_app_accessible_irq!(irq);
    NVIC::is_active(irq)
}

#[inline]
pub fn is_enabled(irq: Interrupt) -> bool {
    assert_app_accessible_irq!(irq);
    if CS_FLAG.load(Ordering::SeqCst) {
        let nr = irq.nr();
        unsafe { CS_MASK[usize::from(nr / 32)] & (1 << (nr % 32)) != 0 }
    } else {
        NVIC::is_enabled(irq)
    }
}

#[inline]
pub fn is_pending(irq: Interrupt) -> bool {
    assert_app_accessible_irq!(irq);
    NVIC::is_pending(irq)
}

#[inline]
pub fn pend(irq: Interrupt) {
    assert_app_accessible_irq!(irq);
    NVIC::pend(irq)
}

#[inline]
pub fn unpend(irq: Interrupt) {
    assert_app_accessible_irq!(irq);
    NVIC::unpend(irq)
}

#[inline]
pub fn get_priority(irq: Interrupt) -> Priority {
    assert_app_accessible_irq!(irq);
    Priority::from_nvic(NVIC::get_priority(irq))
}

#[inline]
pub fn set_priority(irq: Interrupt, prio: Priority) {
    assert_app_accessible_irq!(irq);
    assert!(
        is_app_accessible_priority(prio),
        "priority level {:?} is reserved for the softdevice",
        prio
    );
    unsafe {
        cortex_m::peripheral::Peripherals::steal()
            .NVIC
            .set_priority(irq, prio.to_nvic())
    }
}

#[cfg(feature = "nrf52810")]
fn irq_str(irq: Interrupt) -> defmt::Str {
    match irq {
        POWER_CLOCK => defmt::intern!("POWER_CLOCK"),
        RADIO => defmt::intern!("RADIO"),
        UARTE0_UART0 => defmt::intern!("UARTE0_UART0"),
        TWIM0_TWIS0_TWI0 => defmt::intern!("TWIM0_TWIS0_TWI0"),
        SPIM0_SPIS0_SPI0 => defmt::intern!("SPIM0_SPIS0_SPI0"),
        GPIOTE => defmt::intern!("GPIOTE"),
        SAADC => defmt::intern!("SAADC"),
        TIMER0 => defmt::intern!("TIMER0"),
        TIMER1 => defmt::intern!("TIMER1"),
        TIMER2 => defmt::intern!("TIMER2"),
        RTC0 => defmt::intern!("RTC0"),
        TEMP => defmt::intern!("TEMP"),
        RNG => defmt::intern!("RNG"),
        ECB => defmt::intern!("ECB"),
        CCM_AAR => defmt::intern!("CCM_AAR"),
        WDT => defmt::intern!("WDT"),
        RTC1 => defmt::intern!("RTC1"),
        QDEC => defmt::intern!("QDEC"),
        COMP => defmt::intern!("COMP"),
        SWI0_EGU0 => defmt::intern!("SWI0_EGU0"),
        SWI1_EGU1 => defmt::intern!("SWI1_EGU1"),
        SWI2 => defmt::intern!("SWI2"),
        SWI3 => defmt::intern!("SWI3"),
        SWI4 => defmt::intern!("SWI4"),
        SWI5 => defmt::intern!("SWI5"),
        PWM0 => defmt::intern!("PWM0"),
        PDM => defmt::intern!("PDM"),
    }
}

#[cfg(feature = "nrf52811")]
fn irq_str(irq: Interrupt) -> defmt::Str {
    match irq {
        POWER_CLOCK => defmt::intern!("POWER_CLOCK"),
        RADIO => defmt::intern!("RADIO"),
        UARTE0_UART0 => defmt::intern!("UARTE0_UART0"),
        TWIM0_TWIS0_TWI0_SPIM1_SPIS1_SPI1 => defmt::intern!("TWIM0_TWIS0_TWI0_SPIM1_SPIS1_SPI1"),
        SPIM0_SPIS0_SPI0 => defmt::intern!("SPIM0_SPIS0_SPI0"),
        GPIOTE => defmt::intern!("GPIOTE"),
        SAADC => defmt::intern!("SAADC"),
        TIMER0 => defmt::intern!("TIMER0"),
        TIMER1 => defmt::intern!("TIMER1"),
        TIMER2 => defmt::intern!("TIMER2"),
        RTC0 => defmt::intern!("RTC0"),
        TEMP => defmt::intern!("TEMP"),
        RNG => defmt::intern!("RNG"),
        ECB => defmt::intern!("ECB"),
        CCM_AAR => defmt::intern!("CCM_AAR"),
        WDT => defmt::intern!("WDT"),
        RTC1 => defmt::intern!("RTC1"),
        QDEC => defmt::intern!("QDEC"),
        COMP => defmt::intern!("COMP"),
        SWI0_EGU0 => defmt::intern!("SWI0_EGU0"),
        SWI1_EGU1 => defmt::intern!("SWI1_EGU1"),
        SWI2 => defmt::intern!("SWI2"),
        SWI3 => defmt::intern!("SWI3"),
        SWI4 => defmt::intern!("SWI4"),
        SWI5 => defmt::intern!("SWI5"),
        PWM0 => defmt::intern!("PWM0"),
        PDM => defmt::intern!("PDM"),
    }
}

#[cfg(feature = "nrf52832")]
fn irq_str(irq: Interrupt) -> defmt::Str {
    match irq {
        POWER_CLOCK => defmt::intern!("POWER_CLOCK"),
        RADIO => defmt::intern!("RADIO"),
        UARTE0_UART0 => defmt::intern!("UARTE0_UART0"),
        SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => defmt::intern!("SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0"),
        SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => defmt::intern!("SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1"),
        NFCT => defmt::intern!("NFCT"),
        GPIOTE => defmt::intern!("GPIOTE"),
        SAADC => defmt::intern!("SAADC"),
        TIMER0 => defmt::intern!("TIMER0"),
        TIMER1 => defmt::intern!("TIMER1"),
        TIMER2 => defmt::intern!("TIMER2"),
        RTC0 => defmt::intern!("RTC0"),
        TEMP => defmt::intern!("TEMP"),
        RNG => defmt::intern!("RNG"),
        ECB => defmt::intern!("ECB"),
        CCM_AAR => defmt::intern!("CCM_AAR"),
        WDT => defmt::intern!("WDT"),
        RTC1 => defmt::intern!("RTC1"),
        QDEC => defmt::intern!("QDEC"),
        COMP_LPCOMP => defmt::intern!("COMP_LPCOMP"),
        SWI0_EGU0 => defmt::intern!("SWI0_EGU0"),
        SWI1_EGU1 => defmt::intern!("SWI1_EGU1"),
        SWI2_EGU2 => defmt::intern!("SWI2_EGU2"),
        SWI3_EGU3 => defmt::intern!("SWI3_EGU3"),
        SWI4_EGU4 => defmt::intern!("SWI4_EGU4"),
        SWI5_EGU5 => defmt::intern!("SWI5_EGU5"),
        TIMER3 => defmt::intern!("TIMER3"),
        TIMER4 => defmt::intern!("TIMER4"),
        PWM0 => defmt::intern!("PWM0"),
        PDM => defmt::intern!("PDM"),
        MWU => defmt::intern!("MWU"),
        PWM1 => defmt::intern!("PWM1"),
        PWM2 => defmt::intern!("PWM2"),
        SPIM2_SPIS2_SPI2 => defmt::intern!("SPIM2_SPIS2_SPI2"),
        RTC2 => defmt::intern!("RTC2"),
        I2S => defmt::intern!("I2S"),
        FPU => defmt::intern!("FPU"),
    }
}

#[cfg(feature = "nrf52833")]
fn irq_str(irq: Interrupt) -> defmt::Str {
    match irq {
        POWER_CLOCK => defmt::intern!("POWER_CLOCK"),
        RADIO => defmt::intern!("RADIO"),
        UARTE0_UART0 => defmt::intern!("UARTE0_UART0"),
        SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => defmt::intern!("SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0"),
        SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => defmt::intern!("SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1"),
        NFCT => defmt::intern!("NFCT"),
        GPIOTE => defmt::intern!("GPIOTE"),
        SAADC => defmt::intern!("SAADC"),
        TIMER0 => defmt::intern!("TIMER0"),
        TIMER1 => defmt::intern!("TIMER1"),
        TIMER2 => defmt::intern!("TIMER2"),
        RTC0 => defmt::intern!("RTC0"),
        TEMP => defmt::intern!("TEMP"),
        RNG => defmt::intern!("RNG"),
        ECB => defmt::intern!("ECB"),
        CCM_AAR => defmt::intern!("CCM_AAR"),
        WDT => defmt::intern!("WDT"),
        RTC1 => defmt::intern!("RTC1"),
        QDEC => defmt::intern!("QDEC"),
        COMP_LPCOMP => defmt::intern!("COMP_LPCOMP"),
        SWI0_EGU0 => defmt::intern!("SWI0_EGU0"),
        SWI1_EGU1 => defmt::intern!("SWI1_EGU1"),
        SWI2_EGU2 => defmt::intern!("SWI2_EGU2"),
        SWI3_EGU3 => defmt::intern!("SWI3_EGU3"),
        SWI4_EGU4 => defmt::intern!("SWI4_EGU4"),
        SWI5_EGU5 => defmt::intern!("SWI5_EGU5"),
        TIMER3 => defmt::intern!("TIMER3"),
        TIMER4 => defmt::intern!("TIMER4"),
        PWM0 => defmt::intern!("PWM0"),
        PDM => defmt::intern!("PDM"),
        MWU => defmt::intern!("MWU"),
        PWM1 => defmt::intern!("PWM1"),
        PWM2 => defmt::intern!("PWM2"),
        SPIM2_SPIS2_SPI2 => defmt::intern!("SPIM2_SPIS2_SPI2"),
        RTC2 => defmt::intern!("RTC2"),
        I2S => defmt::intern!("I2S"),
        FPU => defmt::intern!("FPU"),
        USBD => defmt::intern!("USBD"),
        UARTE1 => defmt::intern!("UARTE1"),
        PWM3 => defmt::intern!("PWM3"),
        SPIM3 => defmt::intern!("SPIM3"),
    }
}

#[cfg(feature = "nrf52840")]
fn irq_str(irq: Interrupt) -> defmt::Str {
    match irq {
        POWER_CLOCK => defmt::intern!("POWER_CLOCK"),
        RADIO => defmt::intern!("RADIO"),
        UARTE0_UART0 => defmt::intern!("UARTE0_UART0"),
        SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => defmt::intern!("SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0"),
        SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1 => defmt::intern!("SPIM1_SPIS1_TWIM1_TWIS1_SPI1_TWI1"),
        NFCT => defmt::intern!("NFCT"),
        GPIOTE => defmt::intern!("GPIOTE"),
        SAADC => defmt::intern!("SAADC"),
        TIMER0 => defmt::intern!("TIMER0"),
        TIMER1 => defmt::intern!("TIMER1"),
        TIMER2 => defmt::intern!("TIMER2"),
        RTC0 => defmt::intern!("RTC0"),
        TEMP => defmt::intern!("TEMP"),
        RNG => defmt::intern!("RNG"),
        ECB => defmt::intern!("ECB"),
        CCM_AAR => defmt::intern!("CCM_AAR"),
        WDT => defmt::intern!("WDT"),
        RTC1 => defmt::intern!("RTC1"),
        QDEC => defmt::intern!("QDEC"),
        COMP_LPCOMP => defmt::intern!("COMP_LPCOMP"),
        SWI0_EGU0 => defmt::intern!("SWI0_EGU0"),
        SWI1_EGU1 => defmt::intern!("SWI1_EGU1"),
        SWI2_EGU2 => defmt::intern!("SWI2_EGU2"),
        SWI3_EGU3 => defmt::intern!("SWI3_EGU3"),
        SWI4_EGU4 => defmt::intern!("SWI4_EGU4"),
        SWI5_EGU5 => defmt::intern!("SWI5_EGU5"),
        TIMER3 => defmt::intern!("TIMER3"),
        TIMER4 => defmt::intern!("TIMER4"),
        PWM0 => defmt::intern!("PWM0"),
        PDM => defmt::intern!("PDM"),
        MWU => defmt::intern!("MWU"),
        PWM1 => defmt::intern!("PWM1"),
        PWM2 => defmt::intern!("PWM2"),
        SPIM2_SPIS2_SPI2 => defmt::intern!("SPIM2_SPIS2_SPI2"),
        RTC2 => defmt::intern!("RTC2"),
        I2S => defmt::intern!("I2S"),
        FPU => defmt::intern!("FPU"),
        USBD => defmt::intern!("USBD"),
        UARTE1 => defmt::intern!("UARTE1"),
        QSPI => defmt::intern!("QSPI"),
        CRYPTOCELL => defmt::intern!("CRYPTOCELL"),
        PWM3 => defmt::intern!("PWM3"),
        SPIM3 => defmt::intern!("SPIM3"),
    }
}
