//! # Quat MP4 - Digit Module
//! 
//! Provides the fundamental quaternary digit representation (`QuatDigit`) 
//! packed into 2 bits, featuring zero-cost abstractions and strict validation.

use core::fmt;
use crate::error::QuatError;

/// Represents a single quaternary digit (2 bits: values 0 through 3).
/// 
/// # Examples
/// 
/// ```
/// use quat_mp4::QuatDigit;
/// 
/// let digit = QuatDigit::try_from(2).unwrap();
/// assert_eq!(digit.to_bits(), 0b10);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum QuatDigit {
    /// Quaternary digit 0 (binary `0b00`)
    Q0 = 0b00,
    /// Quaternary digit 1 (binary `0b01`)
    Q1 = 0b01,
    /// Quaternary digit 2 (binary `0b10`)
    Q2 = 0b10,
    /// Quaternary digit 3 (binary `0b11`)
    Q3 = 0b11,
}

impl QuatDigit {
    /// Creates a `QuatDigit` from a raw 2-bit value.
    /// 
    /// # Errors
    /// 
    /// Returns `QuatError::InvalidBits` if the provided value is greater than 3.
    #[inline]
    pub const fn from_bits(bits: u8) -> Result<Self, QuatError> {
        match bits {
            0b00 => Ok(Self::Q0),
            0b01 => Ok(Self::Q1),
            0b10 => Ok(Self::Q2),
            _ => Err(QuatError::InvalidBits(bits)),
        }
    }

    /// Returns the raw 2-bit representation as a `u8`.
    #[inline]
    #[must_use]
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