//! # Quat MP4 - Buffer Module
//! 
//! Memory-efficient packed buffer storing 4 quaternary digits per single byte.

use core::fmt;
use core::str::FromStr;
use core::ops::Index;
use alloc::vec::Vec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::digit::QuatDigit;
use crate::error::QuatError;

/// A compressed buffer storing 4 quaternary digits per byte.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct QuatBuffer {
    data: Vec<u8>,
    len: usize,
}

impl QuatBuffer {
    /// Creates a new, empty `QuatBuffer`.
    #[must_use]
    pub const fn new() -> Self {
        Self { data: Vec::new(), len: 0 }
    }

    /// Returns a borrowing iterator over the buffer.
    #[must_use]
    pub fn iter(&self) -> QuatBufferIter<'_> {
       QuatBufferIter { buffer: self, index: 0 }
    }

    /// Creates a new `QuatBuffer` with a preallocated capacity for the given number of digits.
    #[must_use]
    pub fn with_capacity(digit_capacity: usize) -> Self {
        let byte_capacity = (digit_capacity + 3) / 4;
        Self {
            data: Vec::with_capacity(byte_capacity),
            len: 0,
        }
    }

    /// Returns the number of quaternary digits stored in the buffer.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer contains no digits.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the maximum number of digits the buffer can hold without reallocating.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.capacity() * 4
    }

    /// Reserves capacity for at least `additional` more digits to be inserted.
    pub fn reserve(&mut self, additional: usize) {
        let current_byte_capacity = self.data.capacity();
        let needed_bytes = (self.len + additional + 3) / 4;
        if needed_bytes > current_byte_capacity {
            self.data.reserve(needed_bytes - self.data.len());
        }
    }

    /// Appends a quaternary digit to the back of the buffer.
    pub fn push(&mut self, digit: QuatDigit) {
        let byte_index = self.len / 4;
        let digit_offset = (self.len % 4) as u8;
        let bit_shift = (3 - digit_offset) * 2;

        if digit_offset == 0 {
            self.data.push(digit.to_bits() << bit_shift);
        } else {
            self.data[byte_index] |= digit.to_bits() << bit_shift;
        }
        self.len += 1;
    }

    /// Removes the last digit from the buffer and returns it, or `None` if it is empty.
    pub fn pop(&mut self) -> Option<QuatDigit> {
        if self.len == 0 {
            return None;
        }

        let last_index = self.len - 1;
        let digit = self.try_get(last_index).ok()?;

        if last_index % 4 == 0 {
            self.data.pop();
        } else {
            let byte_index = last_index / 4;
            let digit_offset = (last_index % 4) as u8;
            let bit_shift = (3 - digit_offset) * 2;
            let mask = !(0b11 << bit_shift);
            self.data[byte_index] &= mask;
        }

        self.len -= 1;
        Some(digit)
    }

    /// Returns the digit at the given index, or a `QuatError` if out of bounds.
    pub fn try_get(&self, index: usize) -> Result<QuatDigit, QuatError> {
        if index >= self.len {
            return Err(QuatError::OutOfBounds { index, len: self.len });
        }

        let byte_index = index / 4;
        let digit_offset = (index % 4) as u8;
        let bit_shift = (3 - digit_offset) * 2;

        let raw_byte = self.data[byte_index];
        let bits = (raw_byte >> bit_shift) & 0b11;

        QuatDigit::from_bits(bits)
    }

    /// Returns the digit at the given index, or `None` if out of bounds.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<QuatDigit> {
        self.try_get(index).ok()
    }

    /// Clears the buffer, removing all digits.
    pub fn clear(&mut self) {
        self.data.clear();
        self.len = 0;
    }

    /// Returns a slice of the underlying raw bytes.
    #[must_use]
    pub fn as_raw_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl From<Vec<QuatDigit>> for QuatBuffer {
    fn from(digits: Vec<QuatDigit>) -> Self {
        let mut buffer = Self::with_capacity(digits.len());
        for digit in digits {
            buffer.push(digit);
        }
        buffer
    }
}

impl AsRef<[u8]> for QuatBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl FromStr for QuatBuffer {
    type Err = QuatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buffer = Self::with_capacity(s.len());
        for c in s.chars() {
            let digit = QuatDigit::try_from(c)?;
            buffer.push(digit);
        }
        Ok(buffer)
    }
}

impl Index<usize> for QuatBuffer {
    type Output = QuatDigit;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        static DIGITS: [QuatDigit; 4] = [
            QuatDigit::Q0, QuatDigit::Q1, QuatDigit::Q2, QuatDigit::Q3,
        ];
        let digit = self.try_get(index).expect("Index out of bounds in QuatBuffer indexing");
        &DIGITS[digit.to_bits() as usize]
    }
}

/// An owning iterator over the items of a `QuatBuffer`.
pub struct QuatBufferIntoIter {
    buffer: QuatBuffer,
    index: usize,
}

impl Iterator for QuatBufferIntoIter {
    type Item = QuatDigit;

    fn next(&mut self) -> Option<Self::Item> {
        let digit = self.buffer.get(self.index)?;
        self.index += 1;
        Some(digit)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for QuatBufferIntoIter {}

impl IntoIterator for QuatBuffer {
    type Item = QuatDigit;
    type IntoIter = QuatBufferIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        QuatBufferIntoIter { buffer: self, index: 0 }
    }
}

/// A borrowing iterator over the items of a `QuatBuffer`.
pub struct QuatBufferIter<'a> {
    buffer: &'a QuatBuffer,
    index: usize,
}

impl<'a> Iterator for QuatBufferIter<'a> {
    type Item = QuatDigit;

    fn next(&mut self) -> Option<Self::Item> {
        let digit = self.buffer.get(self.index)?;
        self.index += 1;
        Some(digit)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for QuatBufferIter<'_> {}

impl<'a> IntoIterator for &'a QuatBuffer {
    type Item = QuatDigit;
    type IntoIter = QuatBufferIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        QuatBufferIter { buffer: self, index: 0 }
    }
}

impl fmt::Display for QuatBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for digit in self {
            write!(f, "{}", digit.to_bits())?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct QuatBufferDto {
    data: Vec<u8>,
    len: usize,
}

impl Serialize for QuatBuffer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let dto = QuatBufferDto {
            data: self.data.clone(),
            len: self.len,
        };
        dto.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for QuatBuffer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let dto = QuatBufferDto::deserialize(deserializer)?;
        let max_possible_len = dto.data.len() * 4;

        if dto.len > max_possible_len {
            return Err(serde::de::Error::custom("Invalid length for stored bytes quantity"));
        }

        if dto.len > 0 {
            let last_byte_idx = (dto.len - 1) / 4;
            let digits_in_last_byte = (dto.len % 4) as u8;
            if digits_in_last_byte > 0 && last_byte_idx < dto.data.len() {
                let valid_digits_mask = match digits_in_last_byte {
                    1 => 0b1100_0000,
                    2 => 0b1111_0000,
                    3 => 0b1111_1100,
                    _ => 0b1111_1111,
                };
                if (dto.data[last_byte_idx] & !valid_digits_mask) != 0 {
                    return Err(serde::de::Error::custom(
                        "Unused bits in the last byte must be zero (canonical form violation)"
                    ));
                }
            }
        }

        Ok(Self {
            data: dto.data,
            len: dto.len,
        })
    }
}