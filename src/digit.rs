use core::fmt;
use crate::error::QuatError;

/// Predstavlja jednu kvatarnu cifru (2 bita: 0..=3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum QuatDigit {
    Q0 = 0b00,
    Q1 = 0b01,
    Q2 = 0b10,
    Q3 = 0b11,
}

impl QuatDigit {
    #[inline]
    pub const fn from_bits(bits: u8) -> Result<Self, QuatError> {
        match bits & 0b11 {
            0b00 => Ok(Self::Q0),
            0b01 => Ok(Self::Q1),
            0b10 => Ok(Self::Q2),
            _ => Err(QuatError::InvalidBits(bits)),
        }
    }

    #[inline]
    pub const fn to_bits(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for QuatDigit {
    type Error = QuatError;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_bits(value)
    }
}

impl TryFrom<char> for QuatDigit {
    type Error = QuatError;

    #[inline]
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '0' => Ok(Self::Q0),
            '1' => Ok(Self::Q1),
            '2' => Ok(Self::Q2),
            '3' => Ok(Self::Q3),
            _ => Err(QuatError::InvalidChar(c)),
        }
    }
}

impl fmt::Display for QuatDigit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_bits())
    }
}