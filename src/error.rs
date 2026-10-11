//! # Quat MP4 - Error Module
//! 
//! Defines robust error types for quaternary buffer and digit operations.

use core::fmt;

/// Represents errors that can occur when working with `QuatDigit` and `QuatBuffer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuatError {
    /// Buffer index is out of bounds.
    OutOfBounds { 
        /// The requested index.
        index: usize, 
        /// The current length of the buffer.
        len: usize 
    },
    /// Bit value is invalid (allowed 0..=3).
    InvalidBits(
        /// The invalid raw bits received.
        u8
    ),
    /// Character cannot be converted to a quaternary digit.
    InvalidChar(
        /// The invalid character received.
        char
    ),
}

impl fmt::Display for QuatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds { index, len } => {
                write!(f, "Index {index} is out of bounds for buffer of length {len}")
            }
            Self::InvalidBits(bits) => {
                write!(f, "Invalid 2-bit value: {bits:#b} (allowed 0b00 to 0b11)")
            }
            Self::InvalidChar(c) => {
                write!(f, "Invalid character for quaternary digit: '{c}' (allowed '0'-'3')")
            }
        }
    }
}

impl core::error::Error for QuatError {}