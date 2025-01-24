#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub struct Flag(u8);

impl Flag {
    pub const LIMITED_DISCOVERY: Self = Self(0b1);
    pub const GENERAL_DISCOVERY: Self = Self(0b10);
    pub const LE_ONLY: Self = Self(0b100);

    // i don't understand these but in case people want them
    pub const BIT3: Self = Self(0b1000);
    pub const BIT4: Self = Self(0b10000);
    // the rest are "reserved for future use"

    pub const fn raw(self) -> u8 {
        self.0
    }
}
