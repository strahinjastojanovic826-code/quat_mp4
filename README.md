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

---

## Benchmark

| Test Name | Iterations / Calls | Total Duration | Status |
| :--- | :--- | :--- | :--- |
| `test_benchmark_ffmpeg_probe` | 10,000 calls[cite: 16] | 329.9 µs[cite: 16] | Passed[cite: 16] |
| `test_benchmark_bulk_push` | 40,000 ops[cite: 16] | 839.8 µs[cite: 16] | Passed[cite: 16] |
| `test_digit_creation_and_bits_bench` | 100,000 iterations[cite: 16] | 5.8554 ms[cite: 16] | Passed[cite: 16] |
| `test_ffmpeg_bridge_integration_bench` | 100,000 iterations[cite: 16] | 7.2033 ms[cite: 16] | Passed[cite: 16] |
| `test_mp4_box_type_fourcc_mapping_bench` | 100,000 iterations[cite: 16] | 6.7638 ms[cite: 16] | Passed[cite: 16] |
| `test_char_try_from_bench` | 100,000 iterations[cite: 16] | 8.9689 ms[cite: 16] | Passed[cite: 16] |
| `test_buffer_clear_and_reserve_bench` | 50,000 iterations[cite: 16] | 9.1479 ms[cite: 16] | Passed[cite: 16] |
| `test_from_str_and_display_bench` | 50,000 iterations[cite: 16] | 10.0579 ms[cite: 16] | Passed[cite: 16] |
| `test_push_slice_and_bulk_efficiency_bench` | 50,000 iterations[cite: 18] | 10.0877 ms[cite: 18] | Passed[cite: 18] |
| `test_quaternary_buffer_packing_and_efficiency_bench` | 50,000 iterations[cite: 18] | 12.4436 ms[cite: 18] | Passed[cite: 18] |
| `test_indexing_bench` | 50,000 iterations[cite: 16] | 12.5138 ms[cite: 16] | Passed[cite: 16] |
| `test_buffer_new_and_capacity_bench` | 100,000 iterations[cite: 16] | 17.212 ms[cite: 16] | Passed[cite: 16] |
| `test_buffer_push_pop_lifecycle_bench` | 50,000 iterations[cite: 18] | 18.8606 ms[cite: 18] | Passed[cite: 18] |
| `test_conversions_and_as_ref_bench` | 50,000 iterations[cite: 18] | 20.12 ms[cite: 18] | Passed[cite: 18] |
| `test_mp4_box_serialization_roundtrip_bench` | 20,000 iterations[cite: 18] | 22.0588 ms[cite: 18] | Passed[cite: 18] |
| `test_iterators_bench` | 50,000 iterations[cite: 18] | 26.2384 ms[cite: 18] | Passed[cite: 18] |
| `test_error_display_bench` | 100,000 iterations[cite: 18] | 130.4282 ms[cite: 18] | Passed[cite: 18] |
| `test_benchmark_single_push` | 40,000 ops[cite: 18] | 4.3675 s[cite: 18] | Passed[cite: 18] |

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
quat_mp4 = "1.3.0"
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