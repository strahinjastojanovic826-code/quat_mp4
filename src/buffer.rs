use core::ops::Index;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::digit::QuatDigit;
use crate::error::QuatError;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct QuatBuffer {
    data: Vec<u8>,
    len: usize,
}

impl QuatBuffer {
    #[must_use]
    pub const fn new() -> Self {
        Self { data: Vec::new(), len: 0 }
    }

    #[must_use]
    pub fn with_capacity(digit_capacity: usize) -> Self {
        let byte_capacity = (digit_capacity + 3) / 4;
        Self {
            data: Vec::with_capacity(byte_capacity),
            len: 0,
        }
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize { self.len }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool { self.len == 0 }

    #[must_use]
    pub fn capacity(&self) -> usize { self.data.capacity() * 4 }

    pub fn reserve(&mut self, additional: usize) {
        let current_byte_capacity = self.data.capacity();
        let needed_bytes = (self.len + additional + 3) / 4;
        if needed_bytes > current_byte_capacity {
            self.data.reserve(needed_bytes - self.data.len());
        }
    }

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

    pub fn pop(&mut self) -> Option<QuatDigit> {
        if self.len == 0 { return None; }

        let last_index = self.len - 1;
        let digit = self.try_get(last_index).ok();

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
        digit
    }

    /// Bezbjedna metoda koja vraća `Result` umjesto panike.
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

    #[must_use]
    pub fn get(&self, index: usize) -> Option<QuatDigit> {
        self.try_get(index).ok()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.len = 0;
    }

    #[must_use]
    pub fn as_raw_bytes(&self) -> &[u8] { &self.data }

    pub fn iter(&self) -> QuatBufferIter<'_> {
        QuatBufferIter { buffer: self, index: 0 }
    }
}

impl Index<usize> for QuatBuffer {
    type Output = QuatDigit;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        static DIGITS: [QuatDigit; 4] = [
            QuatDigit::Q0, QuatDigit::Q1, QuatDigit::Q2, QuatDigit::Q3,
        ];
        let digit = self.try_get(index).expect("Indeks van opsega u QuatBuffer indeksiranju");
        &DIGITS[digit.to_bits() as usize]
    }
}

impl FromIterator<QuatDigit> for QuatBuffer {
    fn from_iter<T: IntoIterator<Item = QuatDigit>>(iter: T) -> Self {
        let iterator = iter.into_iter();
        let (lower, _) = iterator.size_hint();
        let mut buffer = Self::with_capacity(lower);
        for digit in iterator { buffer.push(digit); }
        buffer
    }
}

impl Extend<QuatDigit> for QuatBuffer {
    fn extend<T: IntoIterator<Item = QuatDigit>>(&mut self, iter: T) {
        let iterator = iter.into_iter();
        let (lower, _) = iterator.size_hint();
        self.reserve(lower);
        for digit in iterator { self.push(digit); }
    }
}

#[derive(Debug, Clone)]
pub struct QuatBufferIter<'a> {
    buffer: &'a QuatBuffer,
    index: usize,
}

impl Iterator for QuatBufferIter<'_> {
    type Item = QuatDigit;

    fn next(&mut self) -> Option<Self::Item> {
        let digit = self.buffer.get(self.index)?;
        self.index += 1;
        Some(digit)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.len().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for QuatBufferIter<'_> {}

impl<'a> IntoIterator for &'a QuatBuffer {
    type Item = QuatDigit;
    type IntoIter = QuatBufferIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// --- SERDE IMPLEMENTACIJA ---
// Serijalizujemo direktno sirove bajtove i tačan broj cifara radi maksimalne efikasnosti.

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
        
        // Osnovna validacija učitanih podataka
        let max_possible_len = dto.data.len() * 4;
        if dto.len > max_possible_len {
            return Err(serde::de::Error::custom("Nevalidna dužina za količinu sačuvanih bajtova"));
        }

        Ok(Self {
            data: dto.data,
            len: dto.len,
        })
    }
}