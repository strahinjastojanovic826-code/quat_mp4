use std::os::raw::{c_int, c_uchar, c_char};
use crate::mp4_parser::QuatMp4Box;

/// FFI Export for FFmpeg custom demuxer to probe and parse the 'quat' box.
/// Returns 1 if the 'quat' box is successfully recognized, 0 otherwise.
#[unsafe(no_mangle)]
pub extern "C" fn quat_ffmpeg_probe_box(fourcc: *const c_char) -> c_int {
    if fourcc.is_null() {
        return 0;
    }
    
    let bytes = unsafe { std::slice::from_raw_parts(fourcc as *const u8, 4) };
    if bytes == b"quat" {
        1 // Recognized as Quaternary MP4 custom box
    } else {
        0
    }
}

/// Extracts quaternary payload data to be consumed by an FFmpeg decoder/filter pipeline.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quat_ffmpeg_extract_payload(
    box_ptr: *const QuatMp4Box,
    out_buffer: *mut c_uchar,
    out_capacity: usize,
    out_len: *mut usize,
) -> c_int {
    if box_ptr.is_null() || out_buffer.is_null() || out_len.is_null() {
        return -1; // Null pointer error
    }

    let payload_bytes = unsafe {
        let mbox = &*box_ptr;
        mbox.payload.as_bytes()
    };

    if payload_bytes.len() > out_capacity {
        return -2; // Buffer too small
    }

    unsafe {
        std::ptr::copy_nonoverlapping(
            payload_bytes.as_ptr(),
            out_buffer,
            payload_bytes.len(),
        );
        *out_len = payload_bytes.len();
    }

    0 // Success
}