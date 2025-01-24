pub mod ad;
pub mod appearance;
pub mod flag;
pub mod service_uuid16;

pub use ad::AdvertisementDataType;
pub use appearance::Appearance;
pub use flag::Flag;
pub use service_uuid16::ServiceUuid16;

const LEGACY_PAYLOAD_LEN: usize = 31;
const EXTENDED_PAYLOAD_LEN: usize = 254;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub enum Error {
    Oversize { expected: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub enum ServiceList {
    Incomplete,
    Complete,
}

pub struct AdvertisementBuilder<const N: usize> {
    buf: [u8; N],
    ptr: usize,
}

pub struct AdvertisementPayload<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> AsRef<[u8]> for AdvertisementPayload<N> {
    fn as_ref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

impl<const N: usize> core::ops::Deref for AdvertisementPayload<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.buf[..self.len]
    }
}

impl<const K: usize> AdvertisementBuilder<K> {
    pub const fn new() -> Self {
        Self { buf: [0; K], ptr: 0 }
    }

    const fn write(mut self, data: &[u8]) -> Self {
        if self.ptr + data.len() <= K {
            let mut i = 0;
            while i < data.len() {
                self.buf[self.ptr] = data[i];
                i += 1;
                self.ptr += 1;
            }
        } else {
            // Overflow, but still track how much data was attempted to be written
            self.ptr += data.len();
        }

        self
    }

    pub const fn capacity() -> usize {
        K
    }

    pub const fn len(&self) -> usize {
        self.ptr
    }

    /// Write raw bytes to the advertisement data.
    ///
    /// *Note: The length is automatically computed and prepended.*
    pub const fn raw(self, ad: AdvertisementDataType, data: &[u8]) -> Self {
        self.write(&[data.len() as u8 + 1, ad.to_u8()]).write(data)
    }

    /// Get the resulting advertisement payload.
    ///
    /// Returns `Error::Oversize` if more than `K` bytes were written to the builder.
    pub const fn try_build(self) -> Result<AdvertisementPayload<K>, Error> {
        if self.ptr <= K {
            Ok(AdvertisementPayload {
                buf: self.buf,
                len: self.ptr,
            })
        } else {
            Err(Error::Oversize { expected: self.ptr })
        }
    }

    /// Get the resulting advertisement payload.
    ///
    /// Panics if more than `K` bytes were written to the builder.
    pub const fn build(self) -> AdvertisementPayload<K> {
        // Use core::assert! even if defmt is enabled because it is const
        core::assert!(self.ptr <= K, "advertisement exceeded buffer length");

        AdvertisementPayload {
            buf: self.buf,
            len: self.ptr,
        }
    }

    /// Add flags to the advertisement data.
    pub const fn flags(self, flags: &[Flag]) -> Self {
        let mut i = 0;
        let mut bits = 0;
        while i < flags.len() {
            bits |= flags[i].raw();
            i += 1;
        }

        self.raw(AdvertisementDataType::FLAGS, &[bits])
    }

    /// Add a list of 16-bit service uuids to the advertisement data.
    pub const fn services_16(self, complete: ServiceList, services: &[ServiceUuid16]) -> Self {
        let ad_type = match complete {
            ServiceList::Incomplete => AdvertisementDataType::INCOMPLETE_16_SERVICE_LIST,
            ServiceList::Complete => AdvertisementDataType::COMPLETE_16_SERVICE_LIST,
        };

        let mut res = self.write(&[(services.len() * 2) as u8 + 1, ad_type.to_u8()]);
        let mut i = 0;
        while i < services.len() {
            res = res.write(&(services[i].raw()).to_le_bytes());
            i += 1;
        }
        res
    }

    /// Add a list of 128-bit service uuids to the advertisement data.
    ///
    /// Note that each UUID in the list needs to be in little-endian format, i.e. opposite to what you would
    /// normally write UUIDs.
    pub const fn services_128(self, complete: ServiceList, services: &[[u8; 16]]) -> Self {
        let ad_type = match complete {
            ServiceList::Incomplete => AdvertisementDataType::INCOMPLETE_128_SERVICE_LIST,
            ServiceList::Complete => AdvertisementDataType::COMPLETE_128_SERVICE_LIST,
        };

        let mut res = self.write(&[(services.len() * 16) as u8 + 1, ad_type.to_u8()]);
        let mut i = 0;
        while i < services.len() {
            res = res.write(&services[i]);
            i += 1;
        }
        res
    }

    /// Add a name to the advertisement data.
    pub const fn short_name(self, name: &str) -> Self {
        self.raw(AdvertisementDataType::SHORT_NAME, name.as_bytes())
    }

    /// Add a name to the advertisement data.
    pub const fn full_name(self, name: &str) -> Self {
        self.raw(AdvertisementDataType::FULL_NAME, name.as_bytes())
    }

    /// Adds the provided string as a name, truncating and typing as needed.
    ///
    /// *Note: This modifier should be placed last.*
    pub const fn adapt_name(self, name: &str) -> Self {
        let p = self.ptr;
        if p + 2 + name.len() <= K {
            self.full_name(name)
        } else {
            let mut res = self.write(&[(K - p) as u8, AdvertisementDataType::SHORT_NAME.to_u8()]);
            let mut i: usize = 0;
            let bytes = name.as_bytes();
            while res.ptr < K {
                res.buf[res.ptr] = bytes[i];
                res.ptr += 1;
                i += 1;
            }
            res
        }
    }

    /// Add an appearance to the advertisement data.
    pub const fn appearance(self, appearance: Appearance) -> Self {
        self.raw(AdvertisementDataType::APPEARANCE, &appearance.raw().to_le_bytes())
    }
}

pub type LegacyAdvertisementBuilder = AdvertisementBuilder<LEGACY_PAYLOAD_LEN>;
pub type ExtendedAdvertisementBuilder = AdvertisementBuilder<EXTENDED_PAYLOAD_LEN>;

pub type LegacyAdvertisementPayload = AdvertisementPayload<LEGACY_PAYLOAD_LEN>;
pub type ExtendedAdvertisementPayload = AdvertisementPayload<EXTENDED_PAYLOAD_LEN>;
