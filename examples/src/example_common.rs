#![macro_use]

use defmt_rtt as _; // global logger
use nrf52840_hal as _;
use nrf_softdevice::pac;
use panic_probe as _;

pub use defmt::{info, intern};

use core::sync::atomic::{AtomicUsize, Ordering};

#[defmt::timestamp]
fn timestamp() -> u64 {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n as u64
}

// Take peripherals, split by softdevice and application
pub fn take_peripherals() -> (nrf_softdevice::Peripherals, Peripherals) {
    let p = pac::Peripherals::take().dewrap();

    (
        nrf_softdevice::Peripherals {
            AAR: p.AAR,
            ACL: p.ACL,
            CCM: p.CCM,
            CLOCK: p.CLOCK,
            ECB: p.ECB,
            EGU1: p.EGU1,
            EGU2: p.EGU2,
            EGU5: p.EGU5,
            MWU: p.MWU,
            NVMC: p.NVMC,
            POWER: p.POWER,
            RADIO: p.RADIO,
            RNG: p.RNG,
            RTC0: p.RTC0,
            SWI1: p.SWI1,
            SWI2: p.SWI2,
            SWI5: p.SWI5,
            TEMP: p.TEMP,
            TIMER0: p.TIMER0,
        },
        Peripherals {
            CC_HOST_RGF: p.CC_HOST_RGF,
            COMP: p.COMP,
            CRYPTOCELL: p.CRYPTOCELL,
            EGU0: p.EGU0,
            EGU3: p.EGU3,
            EGU4: p.EGU4,
            FICR: p.FICR,
            GPIOTE: p.GPIOTE,
            I2S: p.I2S,
            LPCOMP: p.LPCOMP,
            NFCT: p.NFCT,
            P0: p.P0,
            P1: p.P1,
            PDM: p.PDM,
            PPI: p.PPI,
            PWM0: p.PWM0,
            PWM1: p.PWM1,
            PWM2: p.PWM2,
            PWM3: p.PWM3,
            QDEC: p.QDEC,
            QSPI: p.QSPI,
            RTC2: p.RTC2,
            SAADC: p.SAADC,
            SPI0: p.SPI0,
            SPI1: p.SPI1,
            SPI2: p.SPI2,
            SPIM0: p.SPIM0,
            SPIM1: p.SPIM1,
            SPIM2: p.SPIM2,
            SPIM3: p.SPIM3,
            SPIS0: p.SPIS0,
            SPIS1: p.SPIS1,
            SPIS2: p.SPIS2,
            SWI0: p.SWI0,
            SWI3: p.SWI3,
            SWI4: p.SWI4,
            TIMER1: p.TIMER1,
            TIMER2: p.TIMER2,
            TIMER3: p.TIMER3,
            TIMER4: p.TIMER4,
            TWI0: p.TWI0,
            TWI1: p.TWI1,
            TWIM0: p.TWIM0,
            TWIM1: p.TWIM1,
            TWIS0: p.TWIS0,
            TWIS1: p.TWIS1,
            UART0: p.UART0,
            UARTE0: p.UARTE0,
            UARTE1: p.UARTE1,
            UICR: p.UICR,
            USBD: p.USBD,
            WDT: p.WDT,
        },
    )
}

#[allow(non_snake_case)]
pub struct Peripherals {
    pub CC_HOST_RGF: pac::CC_HOST_RGF,
    pub COMP: pac::COMP,
    pub CRYPTOCELL: pac::CRYPTOCELL,
    pub EGU0: pac::EGU0,
    pub EGU3: pac::EGU3,
    pub EGU4: pac::EGU4,
    pub FICR: pac::FICR,
    pub GPIOTE: pac::GPIOTE,
    pub I2S: pac::I2S,
    pub LPCOMP: pac::LPCOMP,
    pub NFCT: pac::NFCT,
    pub P0: pac::P0,
    pub P1: pac::P1,
    pub PDM: pac::PDM,
    pub PPI: pac::PPI,
    pub PWM0: pac::PWM0,
    pub PWM1: pac::PWM1,
    pub PWM2: pac::PWM2,
    pub PWM3: pac::PWM3,
    pub QDEC: pac::QDEC,
    pub QSPI: pac::QSPI,
    pub RTC2: pac::RTC2,
    pub SAADC: pac::SAADC,
    pub SPI0: pac::SPI0,
    pub SPI1: pac::SPI1,
    pub SPI2: pac::SPI2,
    pub SPIM0: pac::SPIM0,
    pub SPIM1: pac::SPIM1,
    pub SPIM2: pac::SPIM2,
    pub SPIM3: pac::SPIM3,
    pub SPIS0: pac::SPIS0,
    pub SPIS1: pac::SPIS1,
    pub SPIS2: pac::SPIS2,
    pub SWI0: pac::SWI0,
    pub SWI3: pac::SWI3,
    pub SWI4: pac::SWI4,
    pub TIMER1: pac::TIMER1,
    pub TIMER2: pac::TIMER2,
    pub TIMER3: pac::TIMER3,
    pub TIMER4: pac::TIMER4,
    pub TWI0: pac::TWI0,
    pub TWI1: pac::TWI1,
    pub TWIM0: pac::TWIM0,
    pub TWIM1: pac::TWIM1,
    pub TWIS0: pac::TWIS0,
    pub TWIS1: pac::TWIS1,
    pub UART0: pac::UART0,
    pub UARTE0: pac::UARTE0,
    pub UARTE1: pac::UARTE1,
    pub UICR: pac::UICR,
    pub USBD: pac::USBD,
    pub WDT: pac::WDT,
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
    /// dewrap = defmt unwrap
    fn dewrap(self) -> T;

    /// dexpect = defmt expect
    fn dexpect<M: defmt::Format>(self, msg: M) -> T;
}

impl<T> Dewrap<T> for Option<T> {
    fn dewrap(self) -> T {
        match self {
            Some(t) => t,
            None => depanic!("Dewrap failed: enum is none"),
        }
    }

    fn dexpect<M: defmt::Format>(self, msg: M) -> T {
        match self {
            Some(t) => t,
            None => depanic!("Unexpected None: {:?}", msg),
        }
    }
}

impl<T, E: defmt::Format> Dewrap<T> for Result<T, E> {
    fn dewrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => depanic!("Dewrap failed: {:?}", e),
        }
    }

    fn dexpect<M: defmt::Format>(self, msg: M) -> T {
        match self {
            Ok(t) => t,
            Err(e) => depanic!("Unexpected error: {:?}: {:?}", msg, e),
        }
    }
}
