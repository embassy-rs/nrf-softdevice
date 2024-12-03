use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embedded_storage::nor_flash::{ErrorType, NorFlashError, NorFlashErrorKind, ReadNorFlash};
use embedded_storage_async::nor_flash::{
    MultiwriteNorFlash, NorFlash as AsyncNorFlash, ReadNorFlash as AsyncReadNorFlash,
};

use crate::util::DropBomb;
use crate::{raw, RawError, Softdevice};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum FlashError {
    Failed,
    AddressMisaligned,
    BufferMisaligned,
}

impl NorFlashError for FlashError {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            Self::Failed => NorFlashErrorKind::Other,
            Self::AddressMisaligned => NorFlashErrorKind::NotAligned,
            Self::BufferMisaligned => NorFlashErrorKind::NotAligned,
        }
    }
}

/// Singleton instance of the Flash softdevice functionality.
pub struct Flash {
    // Prevent Send, Sync
    _private: PhantomData<*mut ()>,
}

static FLASH_TAKEN: AtomicBool = AtomicBool::new(false);

impl Flash {
    const PAGE_SIZE: usize = 4096;

    /// Takes the Flash instance from the softdevice.
    ///
    /// # Panics
    ///
    /// Panics if called more than once.
    pub fn take(_sd: &Softdevice) -> Flash {
        if FLASH_TAKEN
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            panic!("nrf_softdevice::Softdevice::take_flash() called multiple times.")
        }

        Flash { _private: PhantomData }
    }
}

static SIGNAL: Signal<CriticalSectionRawMutex, Result<(), FlashError>> = Signal::new();

pub(crate) fn on_flash_success() {
    SIGNAL.signal(Ok(()))
}

pub(crate) fn on_flash_error() {
    SIGNAL.signal(Err(FlashError::Failed))
}

impl ErrorType for Flash {
    type Error = FlashError;
}

impl ReadNorFlash for Flash {
    const READ_SIZE: usize = 1;

    fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), Self::Error> {
        // Reading is simple since SoC flash is memory-mapped :)
        // TODO check addr/len is in bounds.

        data.copy_from_slice(unsafe { core::slice::from_raw_parts(address as *const u8, data.len()) });

        Ok(())
    }

    fn capacity(&self) -> usize {
        #[cfg(any(feature = "nrf52805", feature = "nrf52810", feature = "nrf52811",))]
        return 48 * 4096; // 192KB

        #[cfg(feature = "nrf52820")]
        return 64 * 4096; // 256KB

        // TODO: nrf52832 has also "lite" edition (QFAB) with 256KB/32KB of flash/RAM
        #[cfg(any(feature = "nrf52832", feature = "nrf52833",))]
        return 128 * 4096; // 512KB

        #[cfg(feature = "nrf52840")]
        return 256 * 4096; // 1024KB
    }
}

impl AsyncReadNorFlash for Flash {
    const READ_SIZE: usize = 1;

    async fn read(&mut self, address: u32, data: &mut [u8]) -> Result<(), FlashError> {
        <Self as ReadNorFlash>::read(self, address, data)
    }

    fn capacity(&self) -> usize {
        <Self as ReadNorFlash>::capacity(self)
    }
}

impl AsyncNorFlash for Flash {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 4096;

    async fn write(&mut self, offset: u32, data: &[u8]) -> Result<(), FlashError> {
        let data_ptr = data.as_ptr();
        let data_len = data.len() as u32;

        let address = offset as usize;
        if address % 4 != 0 {
            return Err(FlashError::AddressMisaligned);
        }
        if (data_ptr as u32) % 4 != 0 || data_len % 4 != 0 {
            return Err(FlashError::BufferMisaligned);
        }

        // This is safe because we've checked ptr and len is aligned above
        let words_ptr = data_ptr as *const u32;
        let words_len = data_len / 4;

        let bomb = DropBomb::new();
        let ret = unsafe { raw::sd_flash_write(address as _, words_ptr, words_len) };
        let ret = match RawError::convert(ret) {
            Ok(()) => SIGNAL.wait().await,
            Err(_e) => {
                warn!("sd_flash_write err {:?}", _e);
                Err(FlashError::Failed)
            }
        };

        bomb.defuse();
        ret
    }

    async fn erase(&mut self, from: u32, to: u32) -> Result<(), FlashError> {
        if from as usize % Self::PAGE_SIZE != 0 {
            return Err(FlashError::AddressMisaligned);
        }
        if to as usize % Self::PAGE_SIZE != 0 {
            return Err(FlashError::AddressMisaligned);
        }

        let bomb = DropBomb::new();
        for address in (from as usize..to as usize).step_by(Self::PAGE_SIZE) {
            let page_number = (address / Self::PAGE_SIZE) as u32;
            let ret = unsafe { raw::sd_flash_page_erase(page_number) };
            match RawError::convert(ret) {
                Ok(()) => match SIGNAL.wait().await {
                    Err(_e) => {
                        warn!("sd_flash_page_erase err {:?}", _e);
                        bomb.defuse();
                        return Err(_e);
                    }
                    _ => {}
                },
                Err(_e) => {
                    warn!("sd_flash_page_erase err {:?}", _e);
                    bomb.defuse();
                    return Err(FlashError::Failed);
                }
            }
        }

        bomb.defuse();
        Ok(())
    }
}

/// According to Nordic, it is possible to perform multiple writes but only changing a bit from 1 -> 0, which
/// is what MultiwriteNorFlash is for.
///
/// "The NVMC is only able to write 0 to bits in flash memory that are erased (set to 1). It cannot rewrite a bit back to 1.
/// Only full 32-bit words can be written to flash memory using the NVMC interface. To write less than 32 bits, write the data
/// as a full 32-bit word and set all the bits that should remain unchanged in the word to 1."
impl MultiwriteNorFlash for Flash {}
