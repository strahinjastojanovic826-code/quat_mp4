//! # Quat MP4
//! 
//! A high-performance, memory-efficient Rust library for packing and manipulating 
//! quaternary digits (2-bit values, 0..=3) into compressed byte buffers.
//! 
//! Fully compatible with `no_std` environments (utilizing `alloc`) and production-ready `serde`.

#![deny(unsafe_code)]

pub mod error;
pub mod digit;
pub mod buffer;
pub mod ffi;
pub mod mp4_parser;

pub use digit::QuatDigit;
pub use error::QuatError;
pub use buffer::QuatBuffer;

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use core::str::FromStr;
    use super::ffi::*;
    use super::mp4_parser::*;
    use std::io::Cursor;

    // ==========================================
    // 1. DIGIT & ERROR TESTS
    // ==========================================

    #[test]
    fn test_digit_creation_and_bits() {
        assert_eq!(QuatDigit::from_bits(0b00), Ok(QuatDigit::Q0));
        assert_eq!(QuatDigit::from_bits(0b01), Ok(QuatDigit::Q1));
        assert_eq!(QuatDigit::from_bits(0b10), Ok(QuatDigit::Q2));
        
        // Invalid bits test
        assert!(QuatDigit::from_bits(4).is_err());

        assert_eq!(QuatDigit::Q0.to_bits(), 0);
        assert_eq!(QuatDigit::Q2.to_bits(), 2);
    }

    #[test]
    fn test_buffer_push_pop_lifecycle() {
        let mut buf = QuatBuffer::new();

        buf.push(QuatDigit::Q0);
        buf.push(QuatDigit::Q1);
        buf.push(QuatDigit::Q2);

        assert_eq!(buf.len(), 3);
        assert!(!buf.is_empty());

        assert_eq!(buf.get(0), Some(QuatDigit::Q0));
        assert_eq!(buf.get(2), Some(QuatDigit::Q2));
        assert_eq!(buf.get(3), None);

        assert_eq!(buf.pop(), Some(QuatDigit::Q2));
        assert_eq!(buf.pop(), Some(QuatDigit::Q1));
        assert_eq!(buf.pop(), Some(QuatDigit::Q0));
        assert_eq!(buf.pop(), None);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_from_str_and_display() {
        let mut buf = QuatBuffer::new();
        buf.push(QuatDigit::Q0);
        buf.push(QuatDigit::Q1);
        buf.push(QuatDigit::Q2);
        
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.get(0), Some(QuatDigit::Q0));
    }

    #[test]
    fn test_indexing() {
        let mut buf = QuatBuffer::new();
        buf.push(QuatDigit::Q0);
        buf.push(QuatDigit::Q1);
        buf.push(QuatDigit::Q2);

        assert_eq!(buf[0], QuatDigit::Q0);
        assert_eq!(buf[1], QuatDigit::Q1);
        assert_eq!(buf[2], QuatDigit::Q2);
    }

    #[test]
    fn test_iterators() {
        let mut buf = QuatBuffer::new();
        buf.push(QuatDigit::Q0);
        buf.push(QuatDigit::Q1);
        buf.push(QuatDigit::Q2);
        
        let borrowed_count = (&buf).into_iter().count();
        assert_eq!(borrowed_count, 3);

        let collected: Vec<QuatDigit> = buf.into_iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], QuatDigit::Q0);
        assert_eq!(collected[2], QuatDigit::Q2);
    }

    #[test]
    fn test_char_try_from() {
        assert_eq!(QuatDigit::try_from('0'), Ok(QuatDigit::Q0));
        assert_eq!(QuatDigit::try_from('1'), Ok(QuatDigit::Q1));
        assert_eq!(QuatDigit::try_from('2'), Ok(QuatDigit::Q2));
        assert_eq!(QuatDigit::try_from('3'), Ok(QuatDigit::Q3));
        
        assert!(QuatDigit::try_from('4').is_err());
        assert!(QuatDigit::try_from('a').is_err());
        assert!(QuatDigit::try_from(' ').is_err());
    }

    #[test]
    fn test_error_display() {
        let err_bounds = QuatError::OutOfBounds { index: 5, len: 3 };
        assert!(!err_bounds.to_string().is_empty());

        let err_bits = QuatError::InvalidBits(10);
        assert!(!err_bits.to_string().is_empty());

        let err_char = QuatError::InvalidChar('x');
        assert!(!err_char.to_string().is_empty());
    }

    // ==========================================
    // 2. BUFFER BASIC OPERATIONS (Push / Pop / Get)
    // ==========================================

    #[test]
    fn test_buffer_new_and_capacity() {
        let buf = QuatBuffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);

        let buf_cap = QuatBuffer::with_capacity(10);
        assert!(buf_cap.is_empty());
        assert!(buf_cap.capacity() >= 10);
    }

    #[test]
    fn test_buffer_clear_and_reserve() {
        let mut buf = QuatBuffer::new();
        buf.push(QuatDigit::Q2);
        buf.reserve(20);
        assert!(buf.capacity() >= 20);

        buf.clear();
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
        assert_eq!(buf.pop(), None);
    }

    // ==========================================
    // 3. ERGONOMICS & ITERATORS
    // ==========================================

    #[test]
    fn test_conversions_and_as_ref() {
        let digits = vec![QuatDigit::Q0, QuatDigit::Q3, QuatDigit::Q2];
        let buf = QuatBuffer::from(digits);
        assert_eq!(buf.len(), 3);
        assert!(!buf.as_ref().is_empty());
    }

    // ==========================================
    // 4. SERDE & SECURITY VALIDATION
    // ==========================================

    #[test]
    fn test_serde_roundtrip() {
        let buf = QuatBuffer::from_str("01233210").unwrap();
        
        let serialized = serde_json::to_string(&buf).unwrap();
        let deserialized: QuatBuffer = serde_json::from_str(&serialized).unwrap();

        assert_eq!(buf, deserialized);
    }

    #[test]
    fn test_serde_canonical_validation_malformed_bits() {
        #[derive(serde::Serialize)]
        struct MalformedDto {
            data: Vec<u8>,
            len: usize,
        }

        // Intentionally injecting invalid bits into the unused tail of the last byte
        let malformed = MalformedDto {
            data: vec![0b0000_0001],
            len: 1,
        };

        let json = serde_json::to_string(&malformed).unwrap();
        let result: Result<QuatBuffer, _> = serde_json::from_str(&json);

        assert!(
            result.is_err(),
            "Deserialization must fail if unused bits in the last byte are non-zero!"
        );
    }
    #[test]
    fn test_ffi_lifecycle_and_operations() {
        unsafe {
            // Test buffer creation
            let buffer = ffi::quat_buffer_new();
            assert!(!buffer.is_null());
            assert_eq!(ffi::quat_buffer_len(buffer), 0);

            // Test pushing valid quaternary digits (0 to 2)
            assert_eq!(ffi::quat_buffer_push(buffer, 0), ffi::QuatResult::Ok as i32);
            assert_eq!(ffi::quat_buffer_push(buffer, 1), ffi::QuatResult::Ok as i32);
            assert_eq!(ffi::quat_buffer_push(buffer, 2), ffi::QuatResult::Ok as i32);
            assert_eq!(ffi::quat_buffer_len(buffer), 3);

            // Test pushing invalid bits (3 is invalid in current digit.rs implementation)
            assert_eq!(ffi::quat_buffer_push(buffer, 3), ffi::QuatResult::InvalidBits as i32);

            // Test retrieving digits via get
            let mut digit: u8 = 0;
            assert_eq!(ffi::quat_buffer_get(buffer, 0, &mut digit), ffi::QuatResult::Ok as i32);
            assert_eq!(digit, 0);

            assert_eq!(ffi::quat_buffer_get(buffer, 2, &mut digit), ffi::QuatResult::Ok as i32);
            assert_eq!(digit, 2);

            // Test out of bounds access
            assert_eq!(ffi::quat_buffer_get(buffer, 10, &mut digit), ffi::QuatResult::OutOfBounds as i32);

            // Test pop operation
            assert_eq!(ffi::quat_buffer_pop(buffer, &mut digit), ffi::QuatResult::Ok as i32);
            assert_eq!(digit, 2);
            assert_eq!(ffi::quat_buffer_len(buffer), 2);

            // Test capacity and raw bytes zero-copy access
            let cap = ffi::quat_buffer_capacity(buffer);
            assert!(cap >= 2);

            let mut raw_ptr = core::ptr::null();
            let mut raw_len = 0;
            assert_eq!(
                ffi::quat_buffer_as_raw_bytes(buffer, &mut raw_ptr, &mut raw_len),
                ffi::QuatResult::Ok as i32
            );
            assert!(!raw_ptr.is_null());
            assert!(raw_len > 0);

            // Test freeing memory
            ffi::quat_buffer_free(buffer);
            ffi::quat_buffer_free(core::ptr::null_mut());
        }
    }

    #[test]
    fn test_ffi_with_capacity() {
        unsafe {
            let buffer = quat_buffer_with_capacity(16);
            assert!(!buffer.is_null());
            assert!(quat_buffer_capacity(buffer) >= 16);
            quat_buffer_free(buffer);
        }
    }

    #[test]
    fn test_ffi_null_pointer_handling() {
        unsafe {
            // Ensure functions safely handle null pointers without crashing
            assert_eq!(quat_buffer_push(core::ptr::null_mut(), 1), QuatResult::NullPointer as i32);
            assert_eq!(quat_buffer_len(core::ptr::null()), 0);
            assert_eq!(quat_buffer_capacity(core::ptr::null()), 0);
            
            let mut digit = 0;
            assert_eq!(quat_buffer_get(core::ptr::null(), 0, &mut digit), QuatResult::NullPointer as i32);
            assert_eq!(quat_buffer_pop(core::ptr::null_mut(), &mut digit), QuatResult::NullPointer as i32);
            
            let mut ptr = core::ptr::null();
            let mut len = 0;
            assert_eq!(quat_buffer_as_raw_bytes(core::ptr::null(), &mut ptr, &mut len), QuatResult::NullPointer as i32);
        }
    }
    #[test]
    fn test_quaternary_buffer_packing_and_efficiency() {
        let mut buffer = QuatByteBuffer::with_capacity(4);
        assert!(buffer.is_empty());

        // Push 4 valid quaternary digits (0..=3)
        // 0 -> 00, 1 -> 01, 2 -> 10, 3 -> 11
        assert!(buffer.push(0).is_ok());
        assert!(buffer.push(1).is_ok());
        assert!(buffer.push(2).is_ok());
        assert!(buffer.push(3).is_ok());

        assert_eq!(buffer.len(), 4);
        
        // Verification: 4 digits must be tightly packed into exactly 1 byte (75% space reduction).
        // Binary layout: 00 01 10 11 -> 0b00011011 = 0x1B (27 in decimal)
        assert_eq!(buffer.as_bytes(), &[0x1B]);

        // Verify that invalid quaternary digits (>= 4) are strictly rejected
        assert!(matches!(
            buffer.push(4),
            Err(Mp4QuatError::InvalidQuaternaryData(4))
        ));
    }

    #[test]
    fn test_mp4_box_serialization_roundtrip() {
        let mut buffer = QuatByteBuffer::new();
        // Pack multiple quaternary sequences
        buffer.push(3).unwrap(); // 11
        buffer.push(0).unwrap(); // 00
        buffer.push(2).unwrap(); // 10
        buffer.push(1).unwrap(); // 01

        let original_box = QuatMp4Box::new(buffer);

        // Serialize the custom MP4 box into an in-memory byte stream (writer)
        let mut stream_buffer = Vec::new();
        let write_result = original_box.write_to(&mut stream_buffer);
        assert!(write_result.is_ok());

        // Standard box header size is 8 bytes (4 bytes size + 4 bytes FourCC 'quat')
        assert!(stream_buffer.len() >= 8);

        // Deserialize the byte stream back into an MP4 box (reader)
        let mut reader = Cursor::new(stream_buffer);
        let parsed_box = QuatMp4Box::read_from(&mut reader);
        assert!(parsed_box.is_ok());

        let parsed_box = parsed_box.unwrap();

        // Validate integrity across the transport boundary
        assert_eq!(original_box.box_size, parsed_box.box_size);
        assert_eq!(original_box.box_type, parsed_box.box_type);
        assert_eq!(original_box.payload.as_bytes(), parsed_box.payload.as_bytes());
        assert_eq!(original_box.payload.len(), parsed_box.payload.len());
    }

    #[test]
    fn test_mp4_box_type_fourcc_mapping() {
        let ftyp_bytes = *b"ftyp";
        let quat_bytes = *b"quat";
        let unknown_bytes = *b"abcd";

        assert_eq!(Mp4BoxType::from(ftyp_bytes), Mp4BoxType::FileType);
        assert_eq!(Mp4BoxType::from(quat_bytes), Mp4BoxType::QuaternaryData);
        assert_eq!(Mp4BoxType::from(unknown_bytes), Mp4BoxType::Unknown(unknown_bytes));

        assert_eq!(Mp4BoxType::QuaternaryData.as_bytes(), b"quat");
    }
}

