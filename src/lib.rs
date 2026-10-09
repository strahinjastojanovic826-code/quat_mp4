pub mod digit;
pub mod error;
pub mod buffer;

pub use digit::QuatDigit;
pub use error::QuatError;
pub use buffer::{QuatBuffer, QuatBufferIter};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_easiest_digit_creation() {
        let d0 = QuatDigit::from_bits(0b00).unwrap();
        let d2 = QuatDigit::from_bits(0b10).unwrap();
        assert_eq!(d0.to_bits(), 0);
        assert_eq!(d2.to_bits(), 2);
        
        // Test da 0b11 ispravno vraća grešku u trenutnoj implementaciji
        let invalid = QuatDigit::from_bits(0b11);
        assert!(matches!(invalid, Err(QuatError::InvalidBits(3))));

        println!("[TEST 1 REPORT] Status: PASSED | Basic digit creation and bit conversion verified successfully.");
    }

    #[test]
    fn test_2_basic_buffer_push_and_get() {
        let mut buf = QuatBuffer::new();
        assert!(buf.is_empty());
        
        buf.push(QuatDigit::Q1);
        buf.push(QuatDigit::Q2);
        
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.get(0), Some(QuatDigit::Q1));
        assert_eq!(buf.get(1), Some(QuatDigit::Q2));
        println!("[TEST 2 REPORT] Status: PASSED | Buffer push and retrieval operations verified. Len: {}", buf.len());
    }

    #[test]
    fn test_3_indexing_and_iteration() {
        let mut buf = QuatBuffer::new();
        buf.push(QuatDigit::Q0);
        buf.push(QuatDigit::Q2);
        buf.push(QuatDigit::Q1);

        // Test Index trait
        assert_eq!(buf[0], QuatDigit::Q0);
        assert_eq!(buf[1], QuatDigit::Q2);
        assert_eq!(buf[2], QuatDigit::Q1);

        // Test Iterator
        let collected: Vec<QuatDigit> = buf.iter().collect();
        assert_eq!(collected, vec![QuatDigit::Q0, QuatDigit::Q2, QuatDigit::Q1]);
        println!("[TEST 3 REPORT] Status: PASSED | Index trait and iterator traversal verified.");
    }

    #[test]
    fn test_4_pop_operation() {
        let mut buf = QuatBuffer::new();
        buf.push(QuatDigit::Q2);
        buf.push(QuatDigit::Q1);

        assert_eq!(buf.len(), 2);
        let popped = buf.pop();
        assert_eq!(popped, Some(QuatDigit::Q1));
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.get(0), Some(QuatDigit::Q2));
        println!("[TEST 4 REPORT] Status: PASSED | Pop operation and buffer contraction verified. New len: {}", buf.len());
    }

    #[test]
    fn test_5_error_handling_out_of_bounds() {
        let mut buf = QuatBuffer::new();
        buf.push(QuatDigit::Q1);

        let res = buf.try_get(5);
        assert!(matches!(res, Err(QuatError::OutOfBounds { index: 5, len: 1 })));
        
        if let Err(e) = res {
            println!("[TEST 5 REPORT] Status: PASSED | Successfully intercepted expected error: '{}'", e);
        }
    }

    #[test]
    fn test_6_serde_serialization_roundtrip() {
        let mut buf = QuatBuffer::new();
        buf.push(QuatDigit::Q0);
        buf.push(QuatDigit::Q1);
        buf.push(QuatDigit::Q2);

        // Serialize to JSON
        let serialized = serde_json::to_string(&buf).expect("Failed to serialize buffer");
        
        // Deserialize back
        let deserialized: QuatBuffer = serde_json::from_str(&serialized).expect("Failed to deserialize buffer");

        assert_eq!(buf.len(), deserialized.len());
        for i in 0..buf.len() {
            assert_eq!(buf[i], deserialized[i]);
        }
        println!("[TEST 6 REPORT] Status: PASSED | Serde roundtrip successful. Payload JSON size: {} bytes", serialized.len());
    }

    #[test]
    fn test_7_stress_test_large_scale_packing() {
        let total_elements = 1000;
        let mut buf = QuatBuffer::with_capacity(total_elements);
        let mut expected = Vec::with_capacity(total_elements);

        // Stress push 1000 elements using valid values (0..=2)
        for i in 0..total_elements {
            let digit_val = (i % 3) as u8;
            let digit = QuatDigit::from_bits(digit_val).unwrap();
            buf.push(digit);
            expected.push(digit);
        }

        assert_eq!(buf.len(), total_elements);

        // Verify every element across all packed bytes
        for i in 0..total_elements {
            assert_eq!(buf.try_get(i).unwrap(), expected[i], "Data corruption at index {}", i);
            assert_eq!(buf[i], expected[i]);
        }

        // Test collection from iterator
        let collected_buf: QuatBuffer = expected.iter().copied().collect();
        assert_eq!(collected_buf.len(), total_elements);

        println!(
            "[TEST 7 REPORT] Status: PASSED | ULTIMATE STRESS TEST COMPLETED.\n\
             - Total Elements Processed: {}\n\
             - Raw Bytes Allocated: {}\n\
             - Data Integrity: 100% Verified across byte boundaries.",
            total_elements,
            buf.as_raw_bytes().len()
        );
    }
}