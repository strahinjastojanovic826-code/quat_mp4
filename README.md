# quat_mp4

`quat_mp4` is a Rust library designed for quaternary logic operations on MP4 data streams. It efficiently packs 2 bits of binary data into 1 quaternary digit (quat), allowing 4 quats to be stored within a single byte.

---

## Features

- **2-bit Packing**: Maps binary bit pairs (`00`, `01`, `10`, `11`) to quaternary digits (`quat`), achieving an efficient 75% memory space reduction.
- **MP4 Container Integration**: Native support for ISO/IEC 14496-12 (ISOBMFF) file structures using custom `quat` boxes for seamless multimedia and data embedding.
- **FFI & C Interoperability**: Stable C-compatible foreign function interface (`pub extern "C"`) enabling integration with C, C++, and C#.
- **Bit-packed Buffer**: Stores 4 quats per byte in memory using optimized byte vectors with strict boundary checks.
- **Serde Integration**: Native support for serialization and deserialization with strict canonical form validation.
- **Robust Error Handling**: Safe error management using custom `QuatError` and `Mp4QuatError` types.
- **Ergonomic APIs**: Supports string parsing (`FromStr`), indexing, iterators, and both borrowing and owning data flows.
- **FFmpeg Integration**: Native custom demuxer and probe support for parsing and extracting quaternary payloads within FFmpeg filter pipelines.
- **SIMD & Auto-Vectorization Optimization**: High-performance bulk operations (`push_slice`) utilizing exact chunks for maximum data throughput and hardware auto-vectorization.
- **Zero-Cost Abstractions**: Strict validation and type-safe wrappers (`QuatDigit`, `QuatBuffer`) with zero runtime overhead over raw binary data.
- **Memory-Optimized Iteration**: Comprehensive collection iterators (`IntoIter`, borrowing iterators) supporting `ExactSizeIterator` for efficient traversal.
- **FFI Memory Safety**: Robust native interoperability with explicit null-pointer checks, safe memory allocation/deallocation (`Box::into_raw` / `Box::from_raw`), and panic prevention.
- **Quaternary Logic & Bitwise Operations**: Supports per-digit and buffer-level quaternary logic (`quat_not`, `quat_and`, `quat_or`, `quat_xor`) as well as shifts and rotations (`quat_shl`, `quat_shr`, `quat_rotl`, `quat_rotr`).
- **Buffer Manipulation & Subsequence Search**: Provides truncation, splitting (`split_off`), digit reversal (`reverse_digits`), prefix/suffix checks (`starts_with`, `ends_with`), and subsequence search (`find_subslice`).
- **Conversions & Custom Alphabet Encoding**: Converts between packed buffers and unpacked `0..=3` byte sequences, base-4 strings, hex representation, and custom 4-symbol alphabets.
- **Exact Length & Stream Box Skipping**: Preserves exact quaternary digit counts (`digit_len`) across box serialization/deserialization and scans MP4 streams while skipping non-`quat` boxes until a `quat` box is found.
- **Diagnostics & Standard Traits**: Implements standard error and iterator traits (`std::error::Error`, `From<io::Error>`, `FromIterator`, `Extend`) alongside human-readable diagnostics (`diagnostic_summary`) and FFI byte/buffer helpers.

---

## Benchmark

| Test / Benchmark Name | Iterations / Calls | Total Duration | Avg. per Iter / Call | Status |
| :--- | :---: | :---: | :---: | :---: |
| `test_benchmark_ffmpeg_probe` | 10,000 calls | 370 µs | ~37.00 ns | ok |
| `test_benchmark_bulk_push` | 40,000 ops | 614.8 µs | ~15.37 ns | ok |
| `test_digit_creation_and_bits_bench` | 100,000 iterations | 6.6141 ms | ~66.14 ns | ok |
| `test_char_try_from_bench` | 100,000 iterations | 7.349 ms | ~73.49 ns | ok |
| `test_buffer_clear_and_reserve_bench` | 50,000 iterations | 8.2576 ms | ~165.15 ns | ok |
| `test_ffmpeg_bridge_integration_bench` | 100,000 iterations | 6.8119 ms | ~68.12 ns | ok |
| `test_from_str_and_display_bench` | 50,000 iterations | 10.7622 ms | ~215.24 ns | ok |
| `test_mp4_box_type_fourcc_mapping_bench` | 100,000 iterations | 6.9191 ms | ~69.19 ns | ok |
| `test_buffer_new_and_capacity_bench` | 100,000 iterations | 18.757 ms | ~187.57 ns | ok |
| `test_conversions_and_as_ref_bench` | 50,000 iterations | 19.8308 ms | ~396.62 ns | ok |
| `test_buffer_push_pop_lifecycle_bench` | 50,000 iterations | 21.6971 ms | ~433.94 ns | ok |
| `test_indexing_bench` | 50,000 iterations | 14.6431 ms | ~292.86 ns | ok |
| `test_push_slice_and_bulk_efficiency_bench` | 50,000 iterations | 11.5572 ms | ~231.14 ns | ok |
| `test_quaternary_buffer_packing_and_efficiency_bench` | 50,000 iterations | 13.2045 ms | ~264.09 ns | ok |
| `test_iterators_bench` | 50,000 iterations | 28.3457 ms | ~566.91 ns | ok |
| `test_mp4_box_serialization_roundtrip_bench` | 20,000 iterations | 31.2677 ms | ~1.56 µs | ok |
| `test_mp4_exact_len_and_skip_boxes_bench` | 20,000 iterations | 40.5465 ms | ~2.03 µs | ok |
| `test_conversions_and_alphabet_encoding_bench` | 50,000 iterations | 100.7242 ms | ~2.01 µs | ok |
| `test_buffer_slicing_search_and_byte_ops_bench` | 50,000 iterations | 128.4954 ms | ~2.57 µs | ok |
| `test_quat_logic_and_shifts_bench` | 50,000 iterations | 144.8631 ms | ~2.90 µs | ok |
| `test_error_display_bench` | 100,000 iterations | 167.1213 ms | ~1.67 µs | ok |
| `test_serialization_diagnostics_and_ffi_helpers_bench` | 50,000 iterations | 206.0409 ms | ~4.12 µs | ok |
| `test_benchmark_single_push` | 40,000 ops | 4.4925122 s | ~112.31 µs | ok |

## License

This project is dual-licensed to accommodate both open-source and commercial use cases:

### 1. Open-Source License (AGPL-3.0)
By default, this software is licensed under the terms of the **GNU Affero General Public License v3.0 (AGPL-3.0)**. You can freely use, modify, and distribute it under the terms of the AGPL. Note that AGPL requires any software that incorporates or interacts with it over a network to also be open-sourced under the same license.

### 2. Commercial License
If you wish to use `quat_mp4` in proprietary, closed-source, or commercial products without the copyleft obligations of the AGPL, a **Commercial License** is required. 

To obtain a commercial license, please contact us at [strahinjastojanovic826@gmail.com].

---

## Quaternary Mapping Table

| Binary Bits | Quat Value | `QuatDigit` Enum Variant |
|-------------|------------|--------------------------|
| `00`        | 0          | `QuatDigit::Q0`          |
| `01`        | 1          | `QuatDigit::Q1`          |
| `10`        | 2          | `QuatDigit::Q2`          |
| `11`        | 3          | `QuatDigit::Q3`          |

---

## Usage

Add `quat_mp4` to your `Cargo.toml`:

```toml
[dependencies]
quat_mp4 = "1.4.0"
```

### Basic Example

```rust
use quat_mp4::{QuatBuffer, QuatDigit};
use core::str::FromStr;

fn main() -> Result<(), quat_mp4::QuatError> {
    // Create a buffer directly from a string of quaternary digits
    let buffer = QuatBuffer::from_str("0123")?;

    // Total stored digits
    assert_eq!(buffer.len(), 4);

    // Access individual digits by index
    assert_eq!(buffer.get(0), Some(QuatDigit::Q0));
    assert_eq!(buffer.get(3), Some(QuatDigit::Q3));

    // Iterate over digits efficiently
    for digit in &buffer {
        println!("Quat digit: {digit}");
    }

    Ok(())
}
```

---

### Bulk Operations Example

```rust
use quat_mp4::{QuatByteBuffer, QuatDigit};

fn main() -> Result<(), quat_mp4::QuatError> {
    let mut buffer = QuatByteBuffer::new();
    
    // Efficiently push a slice of quaternary digits at once
    let digits = &[QuatDigit::Q3, QuatDigit::Q0, QuatDigit::Q2, QuatDigit::Q1];
    buffer.push_slice(digits)?;

    println!("Total digits stored: {}", buffer.len());
    Ok(())
}
```

---

### MP4 Container Integration Example

```rust
use quat_mp4::{QuatByteBuffer, QuatMp4Box};
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create a quaternary buffer and pack digits (achieving 75% space reduction)
    let mut buffer = QuatByteBuffer::new();
    buffer.push(3)?; // Binary: 11
    buffer.push(0)?; // Binary: 00
    buffer.push(2)?; // Binary: 10
    buffer.push(1)?; // Binary: 01

    // 2. Wrap the compressed quaternary data into a custom ISO/IEC 14496-12 (MP4) 'quat' box
    let mp4_box = QuatMp4Box::new(buffer);

    // 3. Serialize the box directly into an MP4 file stream or writer
    let mut file_stream = Vec::new();
    mp4_box.write_to(&mut file_stream)?;
    println!("Successfully wrote MP4 'quat' box, total size: {} bytes", file_stream.len());

    // 4. Read and parse the MP4 box back from the stream
    let mut reader = Cursor::new(file_stream);
    let parsed_box = QuatMp4Box::read_from(&mut reader)?;
    
    println!("Parsed box type successfully with payload length: {} digits", parsed_box.payload.len());

    Ok(())
}
```

---

### Quaternary Logic & Conversions Example

```rust
use quat_mp4::{QuatBuffer, QuatDigit};
use core::str::FromStr;

fn main() -> Result<(), quat_mp4::QuatError> {
    let mut a = QuatBuffer::from_str("0123")?;
    let b = QuatBuffer::from_str("3210")?;

    let xored = a.quat_xor(&b);
    assert_eq!(xored.to_base4_string(), "3333");

    a.quat_not();
    let dna = a.encode_with_alphabet(['A', 'C', 'G', 'T']);
    assert_eq!(dna, "TGCA");

    let rotated = xored.quat_rotl(1);
    assert_eq!(rotated.len(), 4);
    assert_eq!(a.to_unpacked_vec(), vec![3, 2, 1, 0]);

    Ok(())
}
```

---

### Exact Digit Length & Stream Box Skipping Example

```rust
use quat_mp4::{QuatByteBuffer, QuatMp4Box};
use std::io::{Cursor, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut payload = QuatByteBuffer::new();
    payload.push(3)?;
    payload.push(1)?;
    payload.push(2)?; // 3 digits (non-multiple of 4)

    let box_item = QuatMp4Box::new(payload);
    let mut stream = Vec::new();

    // Write a non-'quat' box ('free') first
    let skip_data = [0x01u8, 0x02u8];
    let skip_size = (8 + skip_data.len()) as u32;
    stream.write_all(&skip_size.to_be_bytes())?;
    stream.write_all(b"free")?;
    stream.write_all(&skip_data)?;

    // Write 'quat' box preserving exact digit length
    box_item.write_exact_to(&mut stream)?;

    let mut cursor = Cursor::new(stream);
    let parsed = QuatMp4Box::read_next_quat_from(&mut cursor)?;
    assert_eq!(parsed.payload.len(), 3);
    assert_eq!(parsed.payload.to_base4_string(), "312");

    Ok(())
}
```

---

## Testing

Run the included unit tests using Cargo:

```bash
cargo test
```

---

# Quat MP4 FFI Integration Guide

`quat_mp4` provides a stable C-compatible ABI layer, allowing seamless integration into C, C++, and C# environments with zero-copy performance and strict error handling.

---

## 1. C Integration (`main.c`)

To use `quat_mp4` in C, include the generated header file `quat_mp4.h` and link against the compiled dynamic or static library.

```c
#include <stdio.h>
#include <stdint.h>
#include "quat_mp4.h"

int main() {
    // Create a new buffer
    QuatBuffer* buffer = quat_buffer_new();
    if (!buffer) {
        fprintf(stderr, "Failed to create buffer.\n");
        return 1;
    }

    // Push quaternary digits (0 to 3)
    quat_buffer_push(buffer, 0);
    quat_buffer_push(buffer, 2);
    quat_buffer_push(buffer, 3);

    printf("Buffer length: %zu\n", quat_buffer_len(buffer));

    // Retrieve a digit by index
    uint8_t digit = 0;
    if (quat_buffer_get(buffer, 1, &digit) == QUAT_OK) {
        printf("Digit at index 1: %u\n", digit);
    }

    // Clean up memory
    quat_buffer_free(buffer);
    return 0;
}
```

---

### Quaternary Logic & FFmpeg Bridge in C (`main.c`)

```c
#include <stdio.h>
#include <stdint.h>
#include "quat_mp4.h"

int main(void) {
    printf("FFmpeg bridge ABI version: %u\n", quat_ffmpeg_abi_version());

    QuatBuffer* a = quat_buffer_new();
    QuatBuffer* b = quat_buffer_new();
    if (!a || !b) {
        fprintf(stderr, "Failed to allocate QuatBuffers.\n");
        quat_buffer_free(a);
        quat_buffer_free(b);
        return 1;
    }

    // Push quaternary digits (0..3 -> 0b00..0b11)
    quat_buffer_push(a, 0);
    quat_buffer_push(a, 1);
    quat_buffer_push(a, 2);
    quat_buffer_push(a, 3);

    quat_buffer_push(b, 3);
    quat_buffer_push(b, 2);
    quat_buffer_push(b, 1);
    quat_buffer_push(b, 0);

    // Quaternary XOR into a newly allocated buffer
    QuatBuffer* xored = NULL;
    if (quat_buffer_quat_xor(a, b, &xored) == QUAT_OK && xored != NULL) {
        uint8_t d0 = 0;
        if (quat_buffer_get(xored, 0, &d0) == QUAT_OK) {
            printf("XOR digit at index 0: %u\n", d0);
        }
        quat_buffer_free(xored);
    }

    // Quaternary NOT in-place on buffer `a`
    if (quat_buffer_quat_not(a) == QUAT_OK) {
        uint8_t inv0 = 0;
        if (quat_buffer_get(a, 0, &inv0) == QUAT_OK) {
            printf("NOT digit at index 0: %u\n", inv0);
        }
    }

    // In-place quaternary NOT over raw packed bytes (FFmpeg bridge helper)
    uint8_t packed_bytes[1] = { 0b00011011u };
    if (quat_ffmpeg_quat_not_bytes(packed_bytes, 1) == 1) {
        printf("Packed byte after quat NOT: 0x%02X\n", packed_bytes[0]);
    }

    quat_buffer_free(a);
    quat_buffer_free(b);
    return 0;
}
```

---

## 2. C++ Integration (`main.cpp`)

The C++ wrapper implements the **RAII** (Resource Acquisition Is Initialization) pattern, move semantics, and automatically translates FFI error codes into standard C++ exceptions.

```cpp
#include <iostream>
#include "quat_mp4.hpp"

int main() {
    try {
        // Initialize buffer with preallocated capacity
        QuatMp4::Buffer buffer(16);

        buffer.push(1);
        buffer.push(3);

        std::cout << "Buffer size: " << buffer.size() << "\n";
        std::cout << "Capacity: " << buffer.capacity() << "\n";

        // Access element safely
        std::cout << "Element at index 0: " << (int)buffer.get(0) << "\n";

        // Zero-copy access to underlying packed raw bytes
        auto rawBytes = buffer.raw_bytes();
        std::cout << "Raw packed bytes count: " << rawBytes.size() << "\n";

    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
        return 1;
    }

    return 0;
}
```

---

## 3. C# Integration (`Program.cs`)

The C# wrapper uses **P/Invoke** and implements `IDisposable` for safe unmanaged memory management in .NET.

```csharp
using System;
using QuatMp4;

class Program
{
    static void Main()
    {
        try
        {
            // Using statement ensures proper disposal of unmanaged memory
            using (var buffer = new QuatBuffer())
            {
                buffer.Push(0);
                buffer.Push(1);
                buffer.Push(3);

                Console.WriteLine($"Buffer Length: {buffer.Length}");
                Console.WriteLine($"Capacity: {buffer.Capacity}");

                // Read value
                byte digit = buffer.Get(2);
                Console.WriteLine($"Element at index 2: {digit}");

                // Pop element
                byte popped = buffer.Pop();
                Console.WriteLine($"Popped element: {popped}");
                Console.WriteLine($"New Length: {buffer.Length}");
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"An error occurred: {ex.Message}");
        }
    }
}
```