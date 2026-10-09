# quat_mp4

`quat_mp4` is a Rust library designed for quaternary logic operations on MP4 data streams. It efficiently packs 2 bits of binary data into 1 quaternary digit (quat), allowing 4 quats to be stored within a single byte.

---

## Features

- **2-bit Packing**: Maps binary bit pairs (`00`, `01`, `10`, `11`) directly to quaternary values (`0`, `1`, `2`, `3`).
- **Bit-packed Buffer**: Stores 4 quats per byte in memory using standard bit-shifts and masks.
- **Modular Design**: Separated into dedicated modules for digits, buffers, and iterators.
- **Serde Integration**: Native support for serialization and deserialization of packed buffers.
- **Robust Error Handling**: Safe error management using custom `QuatError` variants instead of panics.

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
quat_mp4 = "0.2.0"
```

### Basic Example

```rust
use quat_mp4::{QuatBuffer, QuatDigit};

fn main() {
    let mut buffer = QuatBuffer::new();

    // Push quaternary digits (2 bits each)
    buffer.push(QuatDigit::Q0); // 0b00
    buffer.push(QuatDigit::Q1); // 0b01
    buffer.push(QuatDigit::Q2); // 0b10
    buffer.push(QuatDigit::Q3); // 0b11

    // Total stored digits
    assert_eq!(buffer.len(), 4);

    // Get packed raw bytes (0b00_01_10_11 == 0x1B)
    assert_eq!(buffer.as_raw_bytes(), &[0x1B]);

    // Access individual digits by index
    assert_eq!(buffer.get(0), Some(QuatDigit::Q0));
    assert_eq!(buffer.get(3), Some(QuatDigit::Q3));

    // Iterate over digits
    for digit in buffer.iter() {
        println!("Quat digit: {:?}", digit);
    }
}
```

---

## Testing

Run the included unit tests using Cargo:

```bash
cargo test
```

---