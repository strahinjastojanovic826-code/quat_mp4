# quat_mp4

`quat_mp4` is a Rust library designed for quaternary logic operations on MP4 data streams. It efficiently packs 2 bits of binary data into 1 quaternary digit (quat), allowing 4 quats to be stored within a single byte.

---

## Features

- **2-bit Packing**: Maps binary bit pairs (`00`, `01`, `10`, `11`) directly to quaternary digits[cite: 8].
- **Bit-packed Buffer**: Stores 4 quats per byte in memory using standard byte vectors with minimal overhead[cite: 8].
- **`no_std` Compatible**: Fully compatible with `no_std` environments utilizing the `alloc` crate.
- **100% Safe Rust**: Strictly forbids unsafe code (`#[deny(unsafe_code)]`) for maximum reliability and safety.
- **Serde Integration**: Native support for serialization and deserialization with strict canonical bit validation[cite: 8].
- **Robust Error Handling**: Safe error management using custom `QuatError`[cite: 8].
- **Ergonomic APIs**: Supports string parsing (`FromStr`), indexing, and both borrowing and owning iterators.

---

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
quat_mp4 = "1.1.0"
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
