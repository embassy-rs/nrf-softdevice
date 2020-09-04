use crate::{Error, Flash};

#[derive(Copy, Clone, Debug)]
pub enum WriterError {
    Flash(Error),
    OutOfBounds,
}

impl From<Error> for WriterError {
    fn from(e: Error) -> Self {
        Self::Flash(e)
    }
}

#[repr(align(4))]
struct AlignedBuf([u8; 256]);

pub struct Writer<'a, F: Flash> {
    flash: &'a mut F,
    address: usize,
    length: usize,

    write_cur: usize,
    erase_cur: usize,

    buf: AlignedBuf,
    buf_have: usize,
}

impl<'a, F: Flash> Writer<'a, F> {
    pub fn new(flash: &'a mut F, address: usize, length: usize) -> Self {
        assert_eq!(256 & (flash.write_size() - 1), 0);
        assert_eq!(address & (flash.erase_size() - 1), 0);
        assert_eq!(length & (flash.erase_size() - 1), 0);

        Self {
            flash,
            address,
            length,

            write_cur: address,
            erase_cur: address,

            buf: AlignedBuf([0; 256]),
            buf_have: 0,
        }
    }

    async fn do_write(&mut self, len: usize) -> Result<(), WriterError> {
        if self.write_cur + len > self.address + self.length {
            return Err(WriterError::OutOfBounds);
        }

        while self.write_cur + len > self.erase_cur {
            self.flash.erase(self.erase_cur).await?;
            self.erase_cur += self.flash.erase_size();
        }

        self.flash.write(self.write_cur, &self.buf.0[..len]).await?;
        self.write_cur += len;

        Ok(())
    }

    pub async fn write(&mut self, mut data: &[u8]) -> Result<(), WriterError> {
        // This code is HORRIBLE.
        //
        // Calls to flash write must have data aligned to 4 bytes.
        // We can't guarantee `data` is, so we're forced to buffer it
        // somewhere we can make aligned.

        while data.len() != 0 {
            let left = self.buf.0.len() - self.buf_have;
            let n = core::cmp::min(left, data.len());

            self.buf.0[self.buf_have..][..n].copy_from_slice(&data[..n]);
            self.buf_have += n;
            data = &data[n..];

            // When buffer is full, write it out
            if self.buf_have == self.buf.0.len() {
                self.do_write(self.buf.0.len()).await?;
                self.buf_have = 0;
            }
        }

        // Whatever's left in the buffer stays there.
        // It will be written in subsequent calls, or in flush.

        Ok(())
    }

    pub async fn flush(mut self) -> Result<(), WriterError> {
        if self.buf_have != 0 {
            let write_size = self.flash.write_size();

            // round up amount
            let have = (self.buf_have + write_size - 1) & (!(write_size - 1));

            // fill the leftover bytes (if any) with 0xFF
            self.buf.0[self.buf_have..have].fill(0xFF);

            self.do_write(have).await?;
        }
        Ok(())
    }
}
