//! # Quat MP4 - FFI Module
//!
//! Provides a stable C-compatible foreign function interface (FFI) 
//! for integrating QuatBuffer and QuatDigit into C, C++, and C#.

#![allow(unsafe_code)]

extern crate alloc;

use alloc::boxed::Box;
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