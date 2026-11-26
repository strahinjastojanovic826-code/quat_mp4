//! # Quat MP4 - FFmpeg Bridge Module
//!
//! Provides C-compatible `extern "C"` entry points for integrating `quat` MP4 boxes
//! and packed quaternary payloads (`0b00`, `0b01`, `0b10`, `0b11`, 4 digits per byte)
//! with FFmpeg demuxer, decoder, and filter pipelines.
//!
//! ## Capabilities
//! - **Box Probing & Payload Extraction**: Detects `'quat'` box identifiers (`quat_ffmpeg_probe_box`)
//!   and copies payload bytes into caller-provided buffers with null and capacity checks (`quat_ffmpeg_extract_payload`).
//! - **ABI Versioning & Memory Management**: Exposes the bridge ABI version (`quat_ffmpeg_abi_version`)
//!   and safely releases Rust-allocated byte buffers (`quat_ffmpeg_free_bytes`).
//! - **Quaternary Byte Operations**: Applies in-place quaternary bitwise NOT (`^ 0b11` across all
//!   2-bit digit lanes in each byte) over packed buffers (`quat_ffmpeg_quat_not_bytes`).
//!
//! ## Safety & Ownership
//! All exported functions validate pointer arguments where applicable, but callers must guarantee
//! pointer validity, proper alignment, non-overlapping or properly sized buffers, and exact
//! allocation ownership when freeing memory across the FFI boundary.

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

/// Returns the ABI version of the Quaternary FFmpeg bridge.
#[unsafe(no_mangle)]
pub extern "C" fn quat_ffmpeg_abi_version() -> u32 {
    1
}

/// Frees a byte buffer previously allocated by Rust and returned to C/FFmpeg.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quat_ffmpeg_free_bytes(ptr: *mut c_uchar, len: usize) {
    if ptr.is_null() || len == 0 {
        return;
    }
    unsafe {
        let _ = Vec::from_raw_parts(ptr, len, len);
    }
}

/// Applies quaternary bitwise NOT (`^ 0b11` per 2-bit digit) in-place over `len` packed bytes.
/// Returns `1` on success, `0` if `data` is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quat_ffmpeg_quat_not_bytes(data: *mut c_uchar, len: usize) -> c_int {
    if data.is_null() {
        return 0;
    }
    unsafe {
        let slice = std::slice::from_raw_parts_mut(data as *mut u8, len);
        // Each byte holds four 2-bit quaternary digits: (0b11 << 6) | (0b11 << 4) | (0b11 << 2) | 0b11 == 0xFF
        for b in slice.iter_mut() {
            *b ^= 0xFF;
        }
    }
    1
}