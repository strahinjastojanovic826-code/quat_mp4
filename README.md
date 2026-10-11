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
quat_mp4 = "1.0.0"
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