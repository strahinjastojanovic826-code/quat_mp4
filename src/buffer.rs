use crate::digit::QuatDigit;

/// A buffer that stores quaternary digits (2 bits per digit).
/// 4 QuatDigits are packed into 1 byte.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct QuatBuffer {
    data: Vec<u8>,
    len: usize, // Total number of quaternary digits stored
}

impl QuatBuffer {
    /// Creates a new empty quaternary buffer.
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
        }
    }

    /// Creates a buffer with a pre-allocated capacity for digits.
    pub fn with_capacity(digit_capacity: usize) -> Self {
        let byte_capacity = (digit_capacity + 3) / 4;
        Self {
            data: Vec::with_capacity(byte_capacity),
            len: 0,
        }
    }

    /// Pushes a single quaternary digit (2 bits) into the buffer.
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

    /// Gets a quaternary digit at the specified index.
    pub fn get(&self, index: usize) -> Option<QuatDigit> {
        if index >= self.len {
            return None;
        }

        let byte_index = index / 4;
        let digit_offset = (index % 4) as u8;
        let bit_shift = (3 - digit_offset) * 2;

        let raw_byte = self.data[byte_index];
        let bits = (raw_byte >> bit_shift) & 0b11;

        Some(QuatDigit::from_bits(bits))
    }

    /// Returns the number of quaternary digits stored.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer contains no digits.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a slice to the underlying raw packed bytes.
    pub fn as_raw_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Returns an iterator over the quaternary digits.
    pub fn iter(&self) -> QuatBufferIter<'_> {
        QuatBufferIter {
            buffer: self,
            index: 0,
        }
    }
}

/// Iterator for `QuatBuffer`.
pub struct QuatBufferIter<'a> {
    buffer: &'a QuatBuffer,
    index: usize,
}

impl Iterator for QuatBufferIter<'_> {
    type Item = QuatDigit;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buffer.len() {
            let digit = self.buffer.get(self.index);
            self.index += 1;
            digit
        } else {
            None
        }
    }
}