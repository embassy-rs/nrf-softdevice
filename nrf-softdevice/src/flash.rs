use core::future::Future;

use crate::error::Error;
use crate::sd;
use crate::util::*;

pub struct Flash {}

impl Flash {
    pub const PAGE_SIZE: usize = 4096;

    /// safety:
    /// - call this method at most once
    /// - do not call before enabling softdevice
    pub unsafe fn new() -> Self {
        Self {}
    }
}

static SIGNAL: Signal<Result<(), async_flash::Error>> = Signal::new();

pub(crate) fn on_flash_success() {
    SIGNAL.signal(Ok(()))
}

pub(crate) fn on_flash_error() {
    SIGNAL.signal(Err(async_flash::Error::Failed))
}

impl async_flash::Flash for Flash {
    type ReadFuture<'a> = impl Future<Output = Result<(), async_flash::Error>> + 'a;
    type WriteFuture<'a> = impl Future<Output = Result<(), async_flash::Error>> + 'a;
    type ErasePageFuture<'a> = impl Future<Output = Result<(), async_flash::Error>> + 'a;

    fn read<'a>(&'a mut self, address: usize, data: &'a mut [u8]) -> Self::ReadFuture<'a> {
        async move {
            // Reading is simple since SoC flash is memory-mapped :)
            // TODO check addr/len is in bounds.

            data.copy_from_slice(unsafe {
                core::slice::from_raw_parts(address as *const u8, data.len())
            });

            Ok(())
        }
    }

    fn write<'a>(&'a mut self, address: usize, data: &'a [u8]) -> Self::WriteFuture<'a> {
        async move {
            let data_ptr = data.as_ptr();
            let data_len = data.len() as u32;

            if address % 4 != 0 {
                return Err(async_flash::Error::AddressMisaligned);
            }
            if (data_ptr as u32) % 4 != 0 || data_len % 4 != 0 {
                return Err(async_flash::Error::BufferMisaligned);
            }

            // This is safe because we've checked ptr and len is aligned above
            let words_ptr = data_ptr as *const u32;
            let words_len = data_len / 4;

            let bomb = DropBomb::new();
            let ret = unsafe { sd::sd_flash_write(address as _, words_ptr, words_len) };
            let ret = match Error::convert(ret) {
                Ok(()) => SIGNAL.wait().await,
                Err(e) => {
                    warn!("sd_flash_write err {:?}", e);
                    Err(async_flash::Error::Failed)
                }
            };

            bomb.defuse();
            ret
        }
    }

    fn erase<'a>(&'a mut self, address: usize) -> Self::ErasePageFuture<'a> {
        async move {
            if address % Flash::PAGE_SIZE != 0 {
                return Err(async_flash::Error::AddressMisaligned);
            }

            let page_number = address / Flash::PAGE_SIZE;

            let bomb = DropBomb::new();
            let ret = unsafe { sd::sd_flash_page_erase(page_number as u32) };
            let ret = match Error::convert(ret) {
                Ok(()) => SIGNAL.wait().await,
                Err(e) => {
                    warn!("sd_flash_page_erase err {:?}", e);
                    Err(async_flash::Error::Failed)
                }
            };

            bomb.defuse();
            ret
        }
    }

    fn size(&self) -> usize {
        256 * 4096
    }

    fn read_size(&self) -> usize {
        1
    }

    fn write_size(&self) -> usize {
        4
    }

    fn erase_size(&self) -> usize {
        4096
    }
}
