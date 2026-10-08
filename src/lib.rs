pub mod buffer;
pub mod digit;

pub use buffer::QuatBuffer;
pub use digit::QuatDigit;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quat_digit_conversion() {
        assert_eq!(QuatDigit::from_bits(0b00), QuatDigit::Q0);
        assert_eq!(QuatDigit::from_bits(0b01), QuatDigit::Q1);
        assert_eq!(QuatDigit::from_bits(0b10), QuatDigit::Q2);
        assert_eq!(QuatDigit::from_bits(0b11), QuatDigit::Q3);

        assert_eq!(QuatDigit::Q0.to_bits(), 0b00);
        assert_eq!(QuatDigit::Q1.to_bits(), 0b01);
        assert_eq!(QuatDigit::Q2.to_bits(), 0b10);
        assert_eq!(QuatDigit::Q3.to_bits(), 0b11);
    }

    #[test]
    fn test_packing_and_unpacking() {
        let mut buffer = QuatBuffer::new();

        // Pack 4 digits into 1 byte (00 01 10 11 -> 0x1B)
        buffer.push(QuatDigit::Q0);
        buffer.push(QuatDigit::Q1);
        buffer.push(QuatDigit::Q2);
        buffer.push(QuatDigit::Q3);

        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.as_raw_bytes(), &[0b00_01_10_11]);

        assert_eq!(buffer.get(0), Some(QuatDigit::Q0));
        assert_eq!(buffer.get(1), Some(QuatDigit::Q1));
        assert_eq!(buffer.get(2), Some(QuatDigit::Q2));
        assert_eq!(buffer.get(3), Some(QuatDigit::Q3));
        assert_eq!(buffer.get(4), None);
    }

    #[test]
    fn test_iterator() {
        let mut buffer = QuatBuffer::new();
        let digits = vec![QuatDigit::Q3, QuatDigit::Q2, QuatDigit::Q1, QuatDigit::Q0];

        for &d in &digits {
            buffer.push(d);
        }

        let collected: Vec<QuatDigit> = buffer.iter().collect();
        assert_eq!(collected, digits);
    }
}