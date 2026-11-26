//! # Quat MP4 - FFI Module
//!
//! Provides a stable C-compatible foreign function interface (FFI) 
//! for integrating QuatBuffer and QuatDigit into C, C++, and C#.

//! # Quat MP4 - C ABI & FFI Layer
//!
//! Exposes a C-compatible (`extern "C"`) Foreign Function Interface for constructing,
//! inspecting, mutating, and freeing heap-allocated `QuatBuffer` instances from C, C++,
//! C#, and other native callers.
//!
//! ## Capabilities
//! - **Lifecycle & Memory Management**: Allocates buffers on the heap (`quat_buffer_new`,
//!   `quat_buffer_with_capacity`) and safely deallocates non-null pointers (`quat_buffer_free`).
//! - **Element & Bulk Access**: Pushes and pops quaternary digits (`quat_buffer_push`, `quat_buffer_pop`),
//!   queries lengths and capacities (`quat_buffer_len`, `quat_buffer_capacity`), reads by index
//!   (`quat_buffer_get`), and exposes packed raw bytes (`quat_buffer_as_raw_bytes`).
//! - **Quaternary Logic Operations**: Applies quaternary bitwise NOT (`^ 0b11` per digit via
//!   `quat_buffer_quat_not`) and computes per-digit quaternary XOR into a newly allocated
//!   heap buffer (`quat_buffer_quat_xor`).
//!
//! ## Error Handling & Safety
//! Functions returning `c_int` map error conditions to `QuatResult` status codes (`NullPointer`,
//! `OutOfBounds`, `InvalidBits`, etc.). Callers must ensure raw pointers are valid,
//! properly aligned, and not aliased mutably while accessed across FFI boundaries

#![allow(unsafe_code)]

use core::ffi::{c_int, c_uchar};

use crate::buffer::QuatBuffer;
use crate::digit::QuatDigit;

/// FFI result and error codes returned by native functions.
#[repr(C)]
pub enum QuatResult {
    /// Operation completed successfully.
    Ok = 0,
    /// A null pointer was passed where a valid pointer was expected.
    NullPointer = -1,
    /// Index or operation is out of buffer bounds.
    OutOfBounds = -2,
    /// Invalid 2-bit quaternary digit value provided.
    InvalidBits = -3,
    /// Invalid character provided during conversion.
    InvalidChar = -4,
}

impl From<crate::error::QuatError> for QuatResult {
    fn from(err: crate::error::QuatError) -> Self {
        use crate::error::QuatError::*;
        match err {
            OutOfBounds { .. } => QuatResult::OutOfBounds,
            InvalidBits(_) => QuatResult::InvalidBits,
            InvalidChar(_) => QuatResult::InvalidChar,
        }
    }
}

/// Creates a new empty QuatBuffer on the heap.
#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_new() -> *mut QuatBuffer {
    Box::into_raw(Box::new(QuatBuffer::new()))
}

/// Creates a QuatBuffer with preallocated capacity.
#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_with_capacity(capacity: usize) -> *mut QuatBuffer {
    Box::into_raw(Box::new(QuatBuffer::with_capacity(capacity)))
}

/// Frees the QuatBuffer allocated on the heap.
#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_free(buffer: *mut QuatBuffer) {
    if !buffer.is_null() {
        unsafe {
            let _ = Box::from_raw(buffer);
        }
    }
}

/// Pushes a single quaternary digit (0-3) into the buffer.
#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_push(buffer: *mut QuatBuffer, digit_val: c_uchar) -> c_int {
    if buffer.is_null() {
        return QuatResult::NullPointer as c_int;
    }
    let digit = match QuatDigit::from_bits(digit_val) {
        Ok(d) => d,
        Err(e) => return QuatResult::from(e) as c_int,
    };
    unsafe {
        (*buffer).push(digit);
    }
    QuatResult::Ok as c_int
}

/// Pops the last quaternary digit from the buffer.
#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_pop(buffer: *mut QuatBuffer, out_digit: *mut c_uchar) -> c_int {
    if buffer.is_null() || out_digit.is_null() {
        return QuatResult::NullPointer as c_int;
    }
    unsafe {
        match (*buffer).pop() {
            Some(digit) => {
                *out_digit = digit.to_bits();
                QuatResult::Ok as c_int
            }
            None => QuatResult::OutOfBounds as c_int,
        }
    }
}

/// Returns the current number of quaternary digits stored.
#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_len(buffer: *const QuatBuffer) -> usize {
    if buffer.is_null() {
        return 0;
    }
    unsafe { (*buffer).len() }
}

/// Returns the maximum capacity in digits without reallocating.
#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_capacity(buffer: *const QuatBuffer) -> usize {
    if buffer.is_null() {
        return 0;
    }
    unsafe { (*buffer).capacity() }
}

/// Retrieves a digit at the specified index.
#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_get(
    buffer: *const QuatBuffer,
    index: usize,
    out_digit: *mut c_uchar,
) -> c_int {
    if buffer.is_null() || out_digit.is_null() {
        return QuatResult::NullPointer as c_int;
    }
    unsafe {
        match (*buffer).try_get(index) {
            Ok(digit) => {
                *out_digit = digit.to_bits();
                QuatResult::Ok as c_int
            }
            Err(e) => QuatResult::from(e) as c_int,
        }
    }
}

/// Exposes underlying raw packed bytes for zero-copy bulk reading.
#[unsafe(no_mangle)]
pub extern "C" fn quat_buffer_as_raw_bytes(
    buffer: *const QuatBuffer,
    out_ptr: *mut *const c_uchar,
    out_len: *mut usize,
) -> c_int {
    if buffer.is_null() || out_ptr.is_null() || out_len.is_null() {
        return QuatResult::NullPointer as c_int;
    }
    unsafe {
        let bytes = (*buffer).as_raw_bytes();
        *out_ptr = bytes.as_ptr();
        *out_len = bytes.len();
    }
    QuatResult::Ok as c_int
}

/// Applies quaternary bitwise NOT (`3 - digit`) to every digit in `buffer`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quat_buffer_quat_not(buffer: *mut QuatBuffer) -> c_int {
    if buffer.is_null() {
        return QuatResult::NullPointer as c_int;
    }
    unsafe {
        let buf = &mut *buffer;
        let len = buf.len();
        let mut updated = QuatBuffer::with_capacity(len);
        for i in 0..len {
            match buf.try_get(i) {
                Ok(d) => {
                    let inv_bits = d.to_bits() ^ 0b11;
                    match QuatDigit::from_bits(inv_bits) {
                        Ok(inv_d) => updated.push(inv_d),
                        Err(e) => return QuatResult::from(e) as c_int,
                    }
                }
                Err(e) => return QuatResult::from(e) as c_int,
            }
        }
        *buf = updated;
    }
    QuatResult::Ok as c_int
}

/// Computes quaternary XOR (`a ^ b` per digit) of `a` and `b`, writing a new heap `QuatBuffer` into `out_ptr`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn quat_buffer_quat_xor(
    a: *const QuatBuffer,
    b: *const QuatBuffer,
    out_ptr: *mut *mut QuatBuffer,
) -> c_int {
    if a.is_null() || b.is_null() || out_ptr.is_null() {
        return QuatResult::NullPointer as c_int;
    }
    unsafe {
        let buf_a = &*a;
        let buf_b = &*b;
        let min_len = buf_a.len().min(buf_b.len());
        let mut res = QuatBuffer::with_capacity(min_len);

        for i in 0..min_len {
            let da = match buf_a.try_get(i) {
                Ok(v) => v,
                Err(e) => return QuatResult::from(e) as c_int,
            };
            let db = match buf_b.try_get(i) {
                Ok(v) => v,
                Err(e) => return QuatResult::from(e) as c_int,
            };
            let xor_bits = da.to_bits() ^ db.to_bits();
            match QuatDigit::from_bits(xor_bits) {
                Ok(qd) => res.push(qd),
                Err(e) => return QuatResult::from(e) as c_int,
            }
        }

        *out_ptr = Box::into_raw(Box::new(res));
    }
    QuatResult::Ok as c_int
}