use fixed::types::I30F2;

use crate::{raw, RawError, Softdevice};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TempError {
    Raw(RawError),
}

impl From<RawError> for TempError {
    fn from(err: RawError) -> Self {
        TempError::Raw(err)
    }
}

/// Get temperature reading in Celsius
///
/// Note this blocks for ~50us
pub fn temperature_celsius(_sd: &Softdevice) -> Result<I30F2, TempError> {
    let mut temp: i32 = 0;
    let ret = unsafe { raw::sd_temp_get(&mut temp) };
    RawError::convert(ret)?;
    Ok(I30F2::from_bits(temp))
}
