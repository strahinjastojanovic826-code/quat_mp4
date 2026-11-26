//! # Quat MP4 - Buffer Module
//! 
//! Memory-efficient packed buffer storing 4 quaternary digits per single byte.

use core::fmt;
use core::str::FromStr;
use core::ops::Index;
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

    /// Highly optimized bulk push for a slice of QuatDigits.
    /// Processes data in chunks to leverage auto-vectorization (SIMD).
    pub fn push_slice(&mut self, digits: &[QuatDigit]) -> Result<(), QuatError> {
        if digits.is_empty() {
            return Ok(());
        }

        // Reserve capacity beforehand to avoid reallocations
        let additional_len = self.len + digits.len();
        self.reserve(additional_len - self.len);

        // Process chunks of 4 digits (1 byte) for maximum throughput
        let mut chunks = digits.chunks_exact(4);
        for chunk in &mut chunks {
            let b0 = chunk[0].to_bits() & 0b11;
            let b1 = chunk[1].to_bits() & 0b11;
            let b2 = chunk[2].to_bits() & 0b11;
            let b3 = chunk[3].to_bits() & 0b11;

            let packed_byte = (b0 << 6) | (b1 << 4) | (b2 << 2) | b3;
            self.data.push(packed_byte);
        }

        // Handle remaining digits (less than 4)
        let remainder = chunks.remainder();
        if !remainder.is_empty() {
            for digit in remainder {
                // Re-use existing single push logic for the tail
                self.push(*digit);
            }
        }

        self.len = additional_len;
        Ok(())
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

    /// Returns a human-readable diagnostic summary of buffer state and quaternary contents.
    pub fn diagnostic_summary(&self) -> String {
        let digits: String = self
            .iter()
            .map(|d| char::from(b'0' + d.to_bits()))
            .collect();
        format!(
            "QuatBuffer {{ len: {}, packed_bytes: {}, capacity_digits: {}, digits: [{}] }}",
            self.len,
            self.data.len(),
            self.capacity(),
            digits
        )
    }

    /// Applies quaternary NOT (`3 - digit` / `^ 0b11`) to each digit in the buffer.
    pub fn quat_not(&mut self) {
        let mut out = Self::with_capacity(self.len);
        for d in self.iter() {
            if let Ok(inv) = QuatDigit::from_bits(d.to_bits() ^ 0b11) {
                out.push(inv);
            }
        }
        *self = out;
    }

    /// Computes quaternary XOR (`a ^ b` per digit) up to the shorter buffer length.
    pub fn quat_xor(&self, other: &Self) -> Self {
        let min_len = self.len.min(other.len);
        let mut out = Self::with_capacity(min_len);
        for i in 0..min_len {
            let shift = ((3 - (i % 4)) * 2) as u8;
            let a_bits = (self.data[i / 4] >> shift) & 0b11;
            let b_bits = (other.data[i / 4] >> shift) & 0b11;
            if let Ok(d) = QuatDigit::from_bits(a_bits ^ b_bits) {
                out.push(d);
            }
        }
        out
    }

    /// Computes quaternary AND (`a & b` per digit) up to the shorter buffer length.
    pub fn quat_and(&self, other: &Self) -> Self {
        let min_len = self.len.min(other.len);
        let mut out = Self::with_capacity(min_len);
        for i in 0..min_len {
            let shift = ((3 - (i % 4)) * 2) as u8;
            let a_bits = (self.data[i / 4] >> shift) & 0b11;
            let b_bits = (other.data[i / 4] >> shift) & 0b11;
            if let Ok(d) = QuatDigit::from_bits(a_bits & b_bits) {
                out.push(d);
            }
        }
        out
    }

    /// Computes quaternary OR (`a | b` per digit) up to the shorter buffer length.
    pub fn quat_or(&self, other: &Self) -> Self {
        let min_len = self.len.min(other.len);
        let mut out = Self::with_capacity(min_len);
        for i in 0..min_len {
            let shift = ((3 - (i % 4)) * 2) as u8;
            let a_bits = (self.data[i / 4] >> shift) & 0b11;
            let b_bits = (other.data[i / 4] >> shift) & 0b11;
            if let Ok(d) = QuatDigit::from_bits(a_bits | b_bits) {
                out.push(d);
            }
        }
        out
    }

    /// Shifts quaternary digits left by `n` positions, filling vacated right positions with `Q0`.
    pub fn quat_shl(&self, n: usize) -> Self {
        if n >= self.len {
            let mut out = Self::with_capacity(self.len);
            for _ in 0..self.len {
                out.push(QuatDigit::Q0);
            }
            return out;
        }
        let mut out = Self::with_capacity(self.len);
        for i in n..self.len {
            if let Ok(d) = self.try_get(i) {
                out.push(d);
            }
        }
        for _ in 0..n {
            out.push(QuatDigit::Q0);
        }
        out
    }

    /// Shifts quaternary digits right by `n` positions, filling vacated left positions with `Q0`.
    pub fn quat_shr(&self, n: usize) -> Self {
        if n >= self.len {
            let mut out = Self::with_capacity(self.len);
            for _ in 0..self.len {
                out.push(QuatDigit::Q0);
            }
            return out;
        }
        let mut out = Self::with_capacity(self.len);
        for _ in 0..n {
            out.push(QuatDigit::Q0);
        }
        for i in 0..(self.len - n) {
            if let Ok(d) = self.try_get(i) {
                out.push(d);
            }
        }
        out
    }

    /// Rotates quaternary digits left by `n` positions modulo `self.len`.
    pub fn quat_rotl(&self, n: usize) -> Self {
        if self.len == 0 {
            return Self::new();
        }
        let shift = n % self.len;
        let mut out = Self::with_capacity(self.len);
        for i in shift..self.len {
            if let Ok(d) = self.try_get(i) {
                out.push(d);
            }
        }
        for i in 0..shift {
            if let Ok(d) = self.try_get(i) {
                out.push(d);
            }
        }
        out
    }

    /// Rotates quaternary digits right by `n` positions modulo `self.len`.
    pub fn quat_rotr(&self, n: usize) -> Self {
        if self.len == 0 {
            return Self::new();
        }
        let shift = n % self.len;
        self.quat_rotl(self.len - shift)
    }

    /// Truncates the buffer to at most `new_len` quaternary digits.
    pub fn truncate(&mut self, new_len: usize) {
        if new_len >= self.len {
            return;
        }
        let mut trimmed = Self::with_capacity(new_len);
        for i in 0..new_len {
            if let Ok(d) = self.try_get(i) {
                trimmed.push(d);
            }
        }
        *self = trimmed;
    }

    /// Splits the buffer into two at `at`, returning a new buffer containing `[at..len)`.
    pub fn split_off(&mut self, at: usize) -> Result<Self, QuatError> {
        if at > self.len {
            return Err(QuatError::OutOfBounds {
                index: at,
                len: self.len,
            });
        }
        let mut right = Self::with_capacity(self.len - at);
        for i in at..self.len {
            if let Ok(d) = self.try_get(i) {
                right.push(d);
            }
        }
        self.truncate(at);
        Ok(right)
    }

    /// Reverses the order of all quaternary digits in-place.
    pub fn reverse_digits(&mut self) {
        if self.len <= 1 {
            return;
        }
        let mut reversed = Self::with_capacity(self.len);
        for i in (0..self.len).rev() {
            if let Ok(d) = self.try_get(i) {
                reversed.push(d);
            }
        }
        *self = reversed;
    }

    /// Returns `true` if the buffer starts with the given quaternary digit slice.
    pub fn starts_with(&self, prefix: &[QuatDigit]) -> bool {
        if prefix.len() > self.len {
            return false;
        }
        prefix
            .iter()
            .enumerate()
            .all(|(i, expected)| self.get(i) == Some(*expected))
    }

    /// Returns `true` if the buffer ends with the given quaternary digit slice.
    pub fn ends_with(&self, suffix: &[QuatDigit]) -> bool {
        if suffix.len() > self.len {
            return false;
        }
        let start = self.len - suffix.len();
        suffix
            .iter()
            .enumerate()
            .all(|(offset, expected)| self.get(start + offset) == Some(*expected))
    }

    /// Returns the starting index of the first occurrence of `needle`, or `None` if not found.
    pub fn find_subslice(&self, needle: &[QuatDigit]) -> Option<usize> {
        if needle.is_empty() {
            return Some(0);
        }
        if needle.len() > self.len {
            return None;
        }
        for start in 0..=(self.len - needle.len()) {
            let matched = needle
                .iter()
                .enumerate()
                .all(|(offset, expected)| self.get(start + offset) == Some(*expected));
            if matched {
                return Some(start);
            }
        }
        None
    }

    /// Converts all packed quaternary digits into an unpacked `Vec<u8>` with values in `0..=3`.
    pub fn to_unpacked_vec(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.len);
        for d in self.iter() {
            out.push(d.to_bits());
        }
        out
    }

    /// Builds a `QuatBuffer` from a slice of raw digit values (`0..=3`), returning `QuatError::InvalidBits` on invalid input.
    pub fn from_unpacked_slice(values: &[u8]) -> Result<Self, QuatError> {
        let mut buf = Self::with_capacity(values.len());
        for &v in values {
            let digit = QuatDigit::from_bits(v)?;
            buf.push(digit);
        }
        Ok(buf)
    }

    /// Formats the quaternary digits as a base-4 string (`'0'`..`'3'`).
    pub fn to_base4_string(&self) -> String {
        self.iter().map(|d| char::from(b'0' + d.to_bits())).collect()
    }

    /// Formats the underlying packed bytes as a lowercase hexadecimal string.
    pub fn to_hex_packed(&self) -> String {
        let mut s = String::with_capacity(self.data.len() * 2);
        for &b in &self.data {
            s.push(char::from_digit((b >> 4) as u32, 16).unwrap_or('0'));
            s.push(char::from_digit((b & 0x0F) as u32, 16).unwrap_or('0'));
        }
        s
    }

    /// Encodes quaternary digits using a custom 4-symbol alphabet indexed by `0b00..=0b11` (e.g. `['A', 'C', 'G', 'T']`).
    pub fn encode_with_alphabet(&self, alphabet: [char; 4]) -> String {
        self.iter()
            .map(|d| alphabet[(d.to_bits() & 0b11) as usize])
            .collect()
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

impl FromIterator<QuatDigit> for QuatBuffer {
    fn from_iter<T: IntoIterator<Item = QuatDigit>>(iter: T) -> Self {
        let iterator = iter.into_iter();
        let (lower, _) = iterator.size_hint();
        let mut buf = Self::with_capacity(lower);
        for digit in iterator {
            buf.push(digit);
        }
        buf
    }
}

impl Extend<QuatDigit> for QuatBuffer {
    fn extend<T: IntoIterator<Item = QuatDigit>>(&mut self, iter: T) {
        for digit in iter {
            self.push(digit);
        }
    }
}