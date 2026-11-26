//! # Quat MP4
//! 
//! A high-performance, memory-efficient Rust library for packing and manipulating 
//! quaternary digits (2-bit values, 0..=3) into compressed byte buffers.
//! 
//! Fully compatible with `no_std` environments (utilizing `alloc`) and production-ready `serde`.

pub mod error;
pub mod digit;
pub mod buffer;
pub mod ffi;
pub mod mp4_parser;
pub mod ffmpeg_bridge;

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
    use std::time::Duration;
    use std::ffi::c_char;
    use std::time::Instant;

    // ==========================================
    // 1. DIGIT & ERROR TESTS
    // ==========================================

    #[test]
    fn test_digit_creation_and_bits_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 100_000;

        for _ in 0..ITERATIONS {
            assert_eq!(QuatDigit::from_bits(0b00), Ok(QuatDigit::Q0));
            assert_eq!(QuatDigit::from_bits(0b01), Ok(QuatDigit::Q1));
            assert_eq!(QuatDigit::from_bits(0b10), Ok(QuatDigit::Q2));
            assert!(QuatDigit::from_bits(4).is_err());
            assert_eq!(QuatDigit::Q0.to_bits(), 0);
            assert_eq!(QuatDigit::Q2.to_bits(), 2);
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_digit_creation_and_bits ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_buffer_push_pop_lifecycle_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
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

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_buffer_push_pop_lifecycle ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_from_str_and_display_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let mut buf = QuatBuffer::new();
            buf.push(QuatDigit::Q0);
            buf.push(QuatDigit::Q1);
            buf.push(QuatDigit::Q2);

            assert_eq!(buf.len(), 3);
            assert_eq!(buf.get(0), Some(QuatDigit::Q0));
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_from_str_and_display ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_indexing_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let mut buf = QuatBuffer::new();
            buf.push(QuatDigit::Q0);
            buf.push(QuatDigit::Q1);
            buf.push(QuatDigit::Q2);

            assert_eq!(buf[0], QuatDigit::Q0);
            assert_eq!(buf[1], QuatDigit::Q1);
            assert_eq!(buf[2], QuatDigit::Q2);
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_indexing ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_iterators_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let mut buf = QuatBuffer::new();
            buf.push(QuatDigit::Q0);
            buf.push(QuatDigit::Q1);
            buf.push(QuatDigit::Q2);

            let borrowed_count: usize = (&buf).into_iter().count();
            assert_eq!(borrowed_count, 3);

            let collected: Vec<QuatDigit> = buf.into_iter().collect();
            assert_eq!(collected.len(), 3);
            assert_eq!(collected[0], QuatDigit::Q0);
            assert_eq!(collected[2], QuatDigit::Q2);
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_iterators ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_char_try_from_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 100_000;

        for _ in 0..ITERATIONS {
            assert_eq!(QuatDigit::try_from('0'), Ok(QuatDigit::Q0));
            assert_eq!(QuatDigit::try_from('1'), Ok(QuatDigit::Q1));
            assert_eq!(QuatDigit::try_from('2'), Ok(QuatDigit::Q2));
            assert_eq!(QuatDigit::try_from('3'), Ok(QuatDigit::Q3));

            assert!(QuatDigit::try_from('4').is_err());
            assert!(QuatDigit::try_from('a').is_err());
            assert!(QuatDigit::try_from(' ').is_err());
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_char_try_from ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_error_display_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 100_000;

        for _ in 0..ITERATIONS {
            let err_bounds = QuatError::OutOfBounds { index: 5, len: 3 };
            assert!(!err_bounds.to_string().is_empty());

            let err_bits = QuatError::InvalidBits(10);
            assert!(!err_bits.to_string().is_empty());

            let err_char = QuatError::InvalidChar('x');
            assert!(!err_char.to_string().is_empty());
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_error_display ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_buffer_new_and_capacity_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 100_000;

        for _ in 0..ITERATIONS {
            let buf = QuatBuffer::new();
            assert!(buf.is_empty());
            assert_eq!(buf.len(), 0);

            let buf_cap = QuatBuffer::with_capacity(10);
            assert!(buf_cap.is_empty());
            assert!(buf_cap.capacity() >= 10);
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_buffer_new_and_capacity ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_buffer_clear_and_reserve_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let mut buf = QuatBuffer::new();
            buf.push(QuatDigit::Q2);
            buf.reserve(20);
            assert!(buf.capacity() >= 20);

            buf.clear();
            assert_eq!(buf.len(), 0);
            assert!(buf.is_empty());
            assert_eq!(buf.pop(), None);
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_buffer_clear_and_reserve ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_conversions_and_as_ref_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let digits = vec![QuatDigit::Q0, QuatDigit::Q3, QuatDigit::Q2];
            let buf = QuatBuffer::from(digits);
            assert_eq!(buf.len(), 3);
            assert!(!buf.as_ref().is_empty());
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_conversions_and_as_ref ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_quaternary_buffer_packing_and_efficiency_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let mut buffer = QuatByteBuffer::with_capacity(4);
            assert!(buffer.is_empty());

            assert!(buffer.push(0).is_ok());
            assert!(buffer.push(1).is_ok());
            assert!(buffer.push(2).is_ok());
            assert!(buffer.push(3).is_ok());

            assert_eq!(buffer.len(), 4);
            assert_eq!(buffer.as_bytes(), &[0x1B]);
            assert!(matches!(buffer.push(4), Err(Mp4QuatError::InvalidQuaternaryData(4))));
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_quaternary_buffer_packing_and_efficiency ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_mp4_box_serialization_roundtrip_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 20_000;

        for _ in 0..ITERATIONS {
            let mut buffer = QuatByteBuffer::new();
            let _ = buffer.push(3);
            let _ = buffer.push(0);
            let _ = buffer.push(2);
            let _ = buffer.push(1);

            let original_box = QuatMp4Box::new(buffer);
            let mut stream_buffer: Vec<u8> = Vec::new();
            let write_result = original_box.write_to(&mut stream_buffer);
            assert!(write_result.is_ok());

            let mut reader = std::io::Cursor::new(stream_buffer);
            let parsed_box = QuatMp4Box::read_from(&mut reader);
            assert!(parsed_box.is_ok());
            let parsed_box = parsed_box.unwrap();

            assert_eq!(original_box.box_size, parsed_box.box_size);
            assert_eq!(original_box.box_type, parsed_box.box_type);
            assert_eq!(original_box.payload.as_bytes(), parsed_box.payload.as_bytes());
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_mp4_box_serialization_roundtrip ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }
    
    #[test]
    fn test_mp4_box_type_fourcc_mapping_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 100_000;

        for _ in 0..ITERATIONS {
            let ftyp_bytes: [u8; 4] = *b"ftyp";
            let quat_bytes: [u8; 4] = *b"quat";
            let unknown_bytes: [u8; 4] = *b"abcd";

            assert_eq!(Mp4BoxType::from(ftyp_bytes), Mp4BoxType::FileType);
            assert_eq!(Mp4BoxType::from(quat_bytes), Mp4BoxType::QuaternaryData);
            assert_eq!(Mp4BoxType::from(unknown_bytes), Mp4BoxType::Unknown(unknown_bytes));
            assert_eq!(Mp4BoxType::QuaternaryData.as_bytes(), b"quat");
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_mp4_box_type_fourcc_mapping ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_push_slice_and_bulk_efficiency_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let mut buf = QuatByteBuffer::new();
            let _ = buf.push(0);
            let _ = buf.push(1);
            let _ = buf.push(2);
            let _ = buf.push(3);

            assert_eq!(buf.len(), 4);
            assert_eq!(buf.as_bytes(), &[0x1B]);

            let raw: &[u8] = buf.as_bytes();
            assert!(!raw.is_empty());
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_push_slice_and_bulk_efficiency ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_ffmpeg_bridge_integration_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 100_000;

        for _ in 0..ITERATIONS {
            let fourcc: &[u8; 5] = b"quat\0";
            let is_valid: i32 = crate::ffmpeg_bridge::quat_ffmpeg_probe_box(fourcc.as_ptr() as *const std::os::raw::c_char);
            assert_eq!(is_valid, 1);

            let invalid_fourcc: &[u8; 5] = b"avc1\0";
            let is_invalid: i32 = crate::ffmpeg_bridge::quat_ffmpeg_probe_box(invalid_fourcc.as_ptr() as *const std::os::raw::c_char);
            assert_eq!(is_invalid, 0);
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_ffmpeg_bridge_integration ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_benchmark_single_push() {
        const ITERATIONS: usize = 10_000;
        let bytes_data: Vec<u8> = vec![0b00011011; ITERATIONS * 4];

        let start_single = Instant::now();
        let mut buf_single = QuatByteBuffer::with_capacity(ITERATIONS * 4);
        for _ in 0..ITERATIONS {
            for &b in &bytes_data {
                let _ = buf_single.push(b);
            }
        }
        let duration_single = start_single.elapsed();

        println!("\n--- SINGLE PUSH BENCHMARK ---");
        println!("Single push time ({0} ops): {1:?}", ITERATIONS * 4, duration_single);
    }

    #[test]
    fn test_benchmark_bulk_push() {
        const ITERATIONS: usize = 10_000;
        let bytes_data: Vec<u8> = vec![0b00011011; ITERATIONS * 4];

        let start_slice = Instant::now();
        let mut buf_slice = QuatByteBuffer::with_capacity(ITERATIONS * 4);
        for &b in &bytes_data {
            let _ = buf_slice.push(b);
        }
        let duration_slice = start_slice.elapsed();

        println!("\n--- BULK PUSH BENCHMARK ---");
        println!("Bulk push time ({0} ops): {1:?}", ITERATIONS * 4, duration_slice);
    }

    #[test]
    fn test_benchmark_ffmpeg_probe() {
        const ITERATIONS: usize = 10_000;
        let fourcc: &[u8; 5] = b"quat\0";

        let start_probe = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = crate::ffmpeg_bridge::quat_ffmpeg_probe_box(fourcc.as_ptr() as *const std::os::raw::c_char);
        }
        let duration_probe = start_probe.elapsed();

        println!("\n--- FFMPEG PROBE BENCHMARK ---");
        println!("FFmpeg probe time ({0} calls): {1:?}", ITERATIONS, duration_probe);
    }

    #[test]
    fn test_quat_logic_and_shifts_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let mut a = QuatBuffer::new();
            a.push(QuatDigit::Q0); // 0b00
            a.push(QuatDigit::Q1); // 0b01
            a.push(QuatDigit::Q2); // 0b10
            a.push(QuatDigit::Q3); // 0b11

            let mut b = QuatBuffer::new();
            b.push(QuatDigit::Q3); // 0b11
            b.push(QuatDigit::Q2); // 0b10
            b.push(QuatDigit::Q1); // 0b01
            b.push(QuatDigit::Q0); // 0b00

            let xored = a.quat_xor(&b); 
            assert_eq!(xored.get(0), Some(QuatDigit::Q3));
            assert_eq!(xored.get(1), Some(QuatDigit::Q3));
            assert_eq!(xored.get(2), Some(QuatDigit::Q3));
            assert_eq!(xored.get(3), Some(QuatDigit::Q3));

            let anded = a.quat_and(&b);
            assert_eq!(anded.get(0), Some(QuatDigit::Q0));
            assert_eq!(anded.get(1), Some(QuatDigit::Q0));

            let ored = a.quat_or(&b);
            assert_eq!(ored.get(0), Some(QuatDigit::Q3));
            assert_eq!(ored.get(3), Some(QuatDigit::Q3));

            let mut not_buf = a.clone();
            not_buf.quat_not();
            assert_eq!(not_buf.get(0), Some(QuatDigit::Q3));
            assert_eq!(not_buf.get(3), Some(QuatDigit::Q0));

            let rotl = a.quat_rotl(1);
            assert_eq!(rotl.get(0), Some(QuatDigit::Q1));
            assert_eq!(rotl.get(3), Some(QuatDigit::Q0));

            let rotr = rotl.quat_rotr(1);
            assert_eq!(rotr.get(0), Some(QuatDigit::Q0));
            assert_eq!(rotr.get(3), Some(QuatDigit::Q3));

            let shl = a.quat_shl(2);
            assert_eq!(shl.get(0), Some(QuatDigit::Q2));
            assert_eq!(shl.get(2), Some(QuatDigit::Q0));

            let shr = a.quat_shr(2);
            assert_eq!(shr.get(0), Some(QuatDigit::Q0));
            assert_eq!(shr.get(2), Some(QuatDigit::Q0));
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_quat_logic_and_shifts ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_buffer_slicing_search_and_byte_ops_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let mut buf = QuatBuffer::new();
            buf.push(QuatDigit::Q1);
            buf.push(QuatDigit::Q2);
            buf.push(QuatDigit::Q3);
            buf.push(QuatDigit::Q2);
            buf.push(QuatDigit::Q3);

            assert!(buf.starts_with(&[QuatDigit::Q1, QuatDigit::Q2]));
            assert!(buf.ends_with(&[QuatDigit::Q2, QuatDigit::Q3]));
            assert_eq!(buf.find_subslice(&[QuatDigit::Q2, QuatDigit::Q3]), Some(1));

            let tail = buf.split_off(3).unwrap();
            assert_eq!(buf.len(), 3);
            assert_eq!(tail.len(), 2);

            let mut rev = buf.clone();
            rev.reverse_digits();
            assert_eq!(rev.get(0), Some(QuatDigit::Q3));
            assert_eq!(rev.get(2), Some(QuatDigit::Q1));

            let mut byte_buf = QuatByteBuffer::new();
            let _ = byte_buf.push(0);
            let _ = byte_buf.push(1);
            let _ = byte_buf.push(2);
            let _ = byte_buf.push(3);
            assert!(byte_buf.starts_with(&[0, 1]));
            assert!(byte_buf.ends_with(&[2, 3]));
            byte_buf.quat_not();
            assert_eq!(byte_buf.to_base4_string(), "3210");
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_buffer_slicing_search_and_byte_ops ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_conversions_and_alphabet_encoding_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let raw = [0u8, 1u8, 2u8, 3u8, 1u8];
            let buf = QuatBuffer::from_unpacked_slice(&raw).unwrap();
            assert_eq!(buf.to_unpacked_vec(), raw);
            assert_eq!(buf.to_base4_string(), "01231");
            assert!(!buf.to_hex_packed().is_empty());

            let dna = buf.encode_with_alphabet(['A', 'C', 'G', 'T']);
            assert_eq!(dna, "ACGTC");

            assert!(QuatBuffer::from_unpacked_slice(&[0, 4]).is_err());
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_conversions_and_alphabet_encoding ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_mp4_exact_len_and_skip_boxes_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 20_000;

        for _ in 0..ITERATIONS {
            let mut payload = QuatByteBuffer::new();
            let _ = payload.push(3);
            let _ = payload.push(1);
            let _ = payload.push(2); // 3 digits (non-multiple of 4)

            let box_item = QuatMp4Box::new(payload);
            let mut stream = Vec::new();

            // Write a dummy non-quat box ('free') before the 'quat' box
            let dummy_payload = [0xAAu8, 0xBBu8];
            let dummy_size: u32 = 8 + dummy_payload.len() as u32;
            use std::io::Write;
            stream.write_all(&dummy_size.to_be_bytes()).unwrap();
            stream.write_all(b"free").unwrap();
            stream.write_all(&dummy_payload).unwrap();

            // Write exact 'quat' box preserving exact digit length
            box_item.write_exact_to(&mut stream).unwrap();

            let mut cursor = std::io::Cursor::new(stream);
            let parsed = QuatMp4Box::read_next_quat_from(&mut cursor).unwrap();
            assert_eq!(parsed.box_type, Mp4BoxType::QuaternaryData);
            assert_eq!(parsed.payload.len(), 3);
            assert_eq!(parsed.payload.to_base4_string(), "312");
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_mp4_exact_len_and_skip_boxes ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

    #[test]
    fn test_serialization_diagnostics_and_ffi_helpers_bench() {
        let start = Instant::now();
        const ITERATIONS: usize = 50_000;

        for _ in 0..ITERATIONS {
            let mut buf = QuatBuffer::new();
            buf.push(QuatDigit::Q0);
            buf.push(QuatDigit::Q2);

            let diag = buf.diagnostic_summary();
            assert!(diag.contains("len: 2"));

            let json = serde_json::to_string(&buf).unwrap();
            let decoded: QuatBuffer = serde_json::from_str(&json).unwrap();
            assert_eq!(decoded, buf);

            let mut packed = [0b00_01_10_11u8];
            unsafe {
                let res = crate::ffmpeg_bridge::quat_ffmpeg_quat_not_bytes(
                    packed.as_mut_ptr(),
                    packed.len(),
                );
                assert_eq!(res, 1);
            }
            assert_eq!(packed[0], 0b11_10_01_00u8);
            assert_eq!(crate::ffmpeg_bridge::quat_ffmpeg_abi_version(), 1);
        }

        let duration = start.elapsed();
        println!("\n--- BENCHMARK: test_serialization_diagnostics_and_ffi_helpers ---");
        println!("Time for {} iterations: {:?}", ITERATIONS, duration);
    }

}
