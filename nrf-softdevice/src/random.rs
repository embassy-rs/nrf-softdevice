use crate::{raw, RawError, Softdevice};
use defmt::info;
use fixed::types::I30F2;

#[derive(defmt::Format)]
pub enum RandomError {
    BufferTooBig,
    NotEnoughEntropy,
    Raw(RawError),
}

impl From<RawError> for RandomError {
    fn from(err: RawError) -> Self {
        Self::Raw(err)
    }
}

/// Get cryptographically-securerandom bytes.
pub fn random_bytes(_sd: &Softdevice, buf: &mut [u8]) -> Result<(), RandomError> {
    if buf.len() > u8::MAX as usize {
        return Err(RandomError::BufferTooBig);
    }

    let ret = unsafe { raw::sd_rand_application_vector_get(buf[..].as_mut_ptr(), buf.len() as u8) };
    match RawError::convert(ret) {
        Ok(()) => Ok(()),
        Err(RawError::SocRandNotEnoughValues) => Err(RandomError::NotEnoughEntropy),
        Err(e) => Err(e.into()),
    }
}
