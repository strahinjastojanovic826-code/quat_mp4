#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum QuatDigit {
    Q0 = 0b00,
    Q1 = 0b01,
    Q2 = 0b10,
    Q3 = 0b11,
}

impl QuatDigit {
    /// Converts a 2-bit value into a `QuatDigit`.
    /// Truncates any higher bits.
    #[inline]
    pub const fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0b00 => Self::Q0,
            0b01 => Self::Q1,
            0b10 => Self::Q2,
            _ => Self::Q3,
        }
    }

    /// Returns the 2-bit numeric value (0..=3).
    #[inline]
    pub const fn to_bits(self) -> u8 {
        self as u8
    }
}

impl From<u8> for QuatDigit {
    fn from(value: u8) -> Self {
        Self::from_bits(value)
    }
}

impl From<QuatDigit> for u8 {
    fn from(digit: QuatDigit) -> Self {
        digit.to_bits()
    }
}