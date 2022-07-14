use core::future::Future;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

use embedded_storage::nor_flash::{ErrorType, NorFlashError, NorFlashErrorKind, ReadNorFlash};
use embedded_storage_async::nor_flash::{AsyncNorFlash, AsyncReadNorFlash};

use crate::util::{DropBomb, Signal};
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

static SIGNAL: Signal<Result<(), FlashError>> = Signal::new();

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
        256 * 4096
    }
}

impl AsyncReadNorFlash for Flash {
    const READ_SIZE: usize = 1;

    type ReadFuture<'a> = impl Future<Output = Result<(), FlashError>> + 'a;
    fn read<'a>(&'a mut self, address: u32, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move { <Self as ReadNorFlash>::read(self, address, data) }
    }

    fn capacity(&self) -> usize {
        <Self as ReadNorFlash>::capacity(self)
    }
}

impl AsyncNorFlash for Flash {
    const WRITE_SIZE: usize = 4;
    const ERASE_SIZE: usize = 4096;

    type WriteFuture<'a> = impl Future<Output = Result<(), FlashError>> + 'a;
    fn write<'a>(&'a mut self, offset: u32, data: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
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
    }

    type EraseFuture<'a> = impl Future<Output = Result<(), FlashError>> + 'a;
    fn erase<'a>(&'a mut self, from: u32, to: u32) -> Self::EraseFuture<'a> {
        async move {
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
}
