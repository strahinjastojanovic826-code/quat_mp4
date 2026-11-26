use std::io::{Read, Write, Result as IoResult};
use std::fmt;
use std::error::Error as StdError;
use std::io::{self, Seek, SeekFrom};

/// Represents standard and custom FourCC box types for MP4 container compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mp4BoxType {
    /// 'ftyp' - File Type Box
    FileType,
    /// 'moov' - Movie Box
    Movie,
    /// 'mdat' - Media Data Box
    MediaData,
    /// 'quat' - Custom Quaternary Packed Data Box
    QuaternaryData,
    /// Unknown or unhandled standard MP4 box type
    Unknown([u8; 4]),
}

impl From<[u8; 4]> for Mp4BoxType {
    fn from(fourcc: [u8; 4]) -> Self {
        match &fourcc {
            b"ftyp" => Self::FileType,
            b"moov" => Self::Movie,
            b"mdat" => Self::MediaData,
            b"quat" => Self::QuaternaryData,
            other => Self::Unknown(*other),
        }
    }
}

impl Mp4BoxType {
    /// Returns the 4-byte ASCII FourCC identifier for the MP4 box.
    pub const fn as_bytes(&self) -> &[u8; 4] {
        match self {
            Self::FileType => b"ftyp",
            Self::Movie => b"moov",
            Self::MediaData => b"mdat",
            Self::QuaternaryData => b"quat",
            Self::Unknown(bytes) => bytes,
        }
    }
}

/// Robust error types for MP4 Quaternary container parser and builder operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mp4QuatError {
    /// Box size field is structurally invalid or too small.
    InvalidBoxSize,
    /// Unexpected end of file/stream while reading box contents.
    UnexpectedEof,
    /// Provided digit exceeds quaternary limit (must be 0..=3).
    InvalidQuaternaryData(u8),
    /// Unsupported MP4 box version or flags.
    UnsupportedVersion,
    /// IO stream operation failure.
    IoError(String),
}

/// A memory-optimized Quaternary Buffer storing 4 digits per byte (75% space reduction).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuatByteBuffer {
    data: Vec<u8>,
    len: usize,
}

impl QuatByteBuffer {
    /// Creates a new, empty quaternary buffer.
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
            len: 0,
        }
    }

    /// Creates a quaternary buffer with preallocated capacity for high performance.
    pub fn with_capacity(digit_capacity: usize) -> Self {
        let byte_capacity = (digit_capacity + 3) / 4;
        Self {
            data: Vec::with_capacity(byte_capacity),
            len: 0,
        }
    }

    /// Pushes a quaternary digit (0..=3) into the buffer, packing 4 digits into a single byte.
    pub fn push(&mut self, digit: u8) -> Result<(), Mp4QuatError> {
        if digit > 3 {
            return Err(Mp4QuatError::InvalidQuaternaryData(digit));
        }
        let byte_index = self.len / 4;
        let digit_offset = (self.len % 4) as u8;
        let bit_shift = (3 - digit_offset) * 2;

        if byte_index >= self.data.len() {
            self.data.push(0);
        }

        self.data[byte_index] |= digit << bit_shift;
        self.len += 1;
        Ok(())
    }

    /// Returns the raw packed bytes ready to be embedded into an MP4 data stream.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Returns the total number of quaternary digits stored.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Checks if the buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Applies quaternary bitwise NOT (`3 - digit` / `digit ^ 0b11`) to each stored digit.
    pub fn quat_not(&mut self) {
        let mut out = Self::with_capacity(self.len);
        for i in 0..self.len {
            let byte_idx = i / 4;
            let shift = ((3 - (i % 4)) * 2) as u8;
            let digit = (self.data[byte_idx] >> shift) & 0b11;
            let _ = out.push(digit ^ 0b11);
        }
        *self = out;
    }

    /// Applies quaternary XOR (`a ^ b` on 2-bit quaternary digits) up to the shorter length.
    pub fn quat_xor(&self, other: &Self) -> Self {
        let min_len = self.len.min(other.len);
        let mut out = Self::with_capacity(min_len);
        for i in 0..min_len {
            let s_shift = ((3 - (i % 4)) * 2) as u8;
            let o_shift = ((3 - (i % 4)) * 2) as u8;
            let a = (self.data[i / 4] >> s_shift) & 0b11;
            let b = (other.data[i / 4] >> o_shift) & 0b11;
            let _ = out.push(a ^ b);
        }
        out
    }

    /// Applies quaternary AND (`a & b` on 2-bit quaternary digits) up to the shorter length.
    pub fn quat_and(&self, other: &Self) -> Self {
        let min_len = self.len.min(other.len);
        let mut out = Self::with_capacity(min_len);
        for i in 0..min_len {
            let shift = ((3 - (i % 4)) * 2) as u8;
            let a = (self.data[i / 4] >> shift) & 0b11;
            let b = (other.data[i / 4] >> shift) & 0b11;
            let _ = out.push(a & b);
        }
        out
    }

    /// Applies quaternary OR (`a | b` on 2-bit quaternary digits) up to the shorter length.
    pub fn quat_or(&self, other: &Self) -> Self {
        let min_len = self.len.min(other.len);
        let mut out = Self::with_capacity(min_len);
        for i in 0..min_len {
            let shift = ((3 - (i % 4)) * 2) as u8;
            let a = (self.data[i / 4] >> shift) & 0b11;
            let b = (other.data[i / 4] >> shift) & 0b11;
            let _ = out.push(a | b);
        }
        out
    }

    /// Truncates the packed buffer to `new_len` quaternary digits.
    pub fn truncate(&mut self, new_len: usize) {
        if new_len >= self.len {
            return;
        }
        let mut out = Self::with_capacity(new_len);
        for i in 0..new_len {
            let shift = ((3 - (i % 4)) * 2) as u8;
            let d = (self.data[i / 4] >> shift) & 0b11;
            let _ = out.push(d);
        }
        *self = out;
    }

    /// Reverses the order of packed quaternary digits in-place.
    pub fn reverse_digits(&mut self) {
        if self.len <= 1 {
            return;
        }
        let mut out = Self::with_capacity(self.len);
        for i in (0..self.len).rev() {
            let shift = ((3 - (i % 4)) * 2) as u8;
            let d = (self.data[i / 4] >> shift) & 0b11;
            let _ = out.push(d);
        }
        *self = out;
    }

    /// Returns `true` if the packed buffer starts with the given raw digits (`0..=3`).
    pub fn starts_with(&self, prefix: &[u8]) -> bool {
        if prefix.len() > self.len {
            return false;
        }
        for (i, &expected) in prefix.iter().enumerate() {
            let shift = ((3 - (i % 4)) * 2) as u8;
            let actual = (self.data[i / 4] >> shift) & 0b11;
            if actual != expected {
                return false;
            }
        }
        true
    }

    /// Returns `true` if the packed buffer ends with the given raw digits (`0..=3`).
    pub fn ends_with(&self, suffix: &[u8]) -> bool {
        if suffix.len() > self.len {
            return false;
        }
        let start = self.len - suffix.len();
        for (offset, &expected) in suffix.iter().enumerate() {
            let i = start + offset;
            let shift = ((3 - (i % 4)) * 2) as u8;
            let actual = (self.data[i / 4] >> shift) & 0b11;
            if actual != expected {
                return false;
            }
        }
        true
    }

    /// Converts packed quaternary digits into an unpacked `Vec<u8>` with values in `0..=3`.
    pub fn to_unpacked_vec(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.len);
        for i in 0..self.len {
            let shift = ((3 - (i % 4)) * 2) as u8;
            out.push((self.data[i / 4] >> shift) & 0b11);
        }
        out
    }

    /// Builds a `QuatByteBuffer` from a slice of raw digit values (`0..=3`).
    pub fn from_unpacked_slice(values: &[u8]) -> Result<Self, Mp4QuatError> {
        let mut buf = Self::with_capacity(values.len());
        for &v in values {
            buf.push(v)?;
        }
        Ok(buf)
    }

    /// Formats the packed quaternary digits as a base-4 string (`'0'`..`'3'`).
    pub fn to_base4_string(&self) -> String {
        let mut s = String::with_capacity(self.len);
        for i in 0..self.len {
            let shift = ((3 - (i % 4)) * 2) as u8;
            let d = (self.data[i / 4] >> shift) & 0b11;
            s.push(char::from(b'0' + d));
        }
        s
    }

}

/// Represents an ISOBMFF (MP4) Box structure encapsulating quaternary compressed payloads.
#[derive(Debug, Clone)]
pub struct QuatMp4Box {
    pub box_size: u64,
    pub box_type: Mp4BoxType,
    pub payload: QuatByteBuffer,
}

impl QuatMp4Box {
    /// Creates a new custom MP4 box (`quat`) containing quaternary packed data.
    pub fn new(payload: QuatByteBuffer) -> Self {
        // Standard box header size: 4 bytes size + 4 bytes type = 8 bytes
        let payload_len = payload.as_bytes().len() as u64;
        let box_size = 8 + payload_len;
        
        Self {
            box_size,
            box_type: Mp4BoxType::QuaternaryData,
            payload,
        }
    }

    /// Serializes the custom MP4 box directly into an ISO/IEC 14496-12 compliant byte stream/writer.
    pub fn write_to<W: Write>(&self, writer: &mut W) -> IoResult<()> {
        if self.box_size > u32::MAX as u64 {
            // Extended size format (Large box: size = 1, followed by type, followed by 64-bit size)
            writer.write_all(&(1u32).to_be_bytes())?;
            writer.write_all(self.box_type.as_bytes())?;
            writer.write_all(&self.box_size.to_be_bytes())?;
        } else {
            // Standard 32-bit size header
            writer.write_all(&(self.box_size as u32).to_be_bytes())?;
            writer.write_all(self.box_type.as_bytes())?;
        }
        
        // Write the packed quaternary payload bytes (saving 75% space)
        writer.write_all(self.payload.as_bytes())?;
        Ok(())
    }

    /// Parses and reads an MP4 Quaternary Box from a binary stream.
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, Mp4QuatError> {
        let mut size_buf = [0u8; 4];
        reader.read_exact(&mut size_buf).map_err(|e| Mp4QuatError::IoError(e.to_string()))?;
        let initial_size = u32::from_be_bytes(size_buf);

        let mut type_buf = [0u8; 4];
        reader.read_exact(&mut type_buf).map_err(|e| Mp4QuatError::IoError(e.to_string()))?;
        let box_type = Mp4BoxType::from(type_buf);

        let mut box_size = initial_size as u64;
        let header_len;

        if initial_size == 1 {
            let mut large_size_buf = [0u8; 8];
            reader.read_exact(&mut large_size_buf).map_err(|e| Mp4QuatError::IoError(e.to_string()))?;
            box_size = u64::from_be_bytes(large_size_buf);
            header_len = 16;
        } else if initial_size == 0 {
            return Err(Mp4QuatError::InvalidBoxSize); // To the end of file boxes not implemented for simplicity
        } else {
            header_len = 8;
        }

        if box_size < header_len {
            return Err(Mp4QuatError::InvalidBoxSize);
        }

        let payload_len = (box_size - header_len) as usize;
        let mut raw_payload = vec![0u8; payload_len];
        reader.read_exact(&mut raw_payload).map_err(|e| Mp4QuatError::IoError(e.to_string()))?;

        // Reconstruct quaternary buffer layout
        let payload = QuatByteBuffer {
            data: raw_payload,
            len: payload_len * 4, // Assuming full byte alignment representation
        };

        Ok(Self {
            box_size,
            box_type,
            payload,
        })
    }

    /// Serializes the box including exact `digit_len: u64` so non-byte-aligned digit counts are preserved.
    pub fn write_exact_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let packed = self.payload.as_bytes();
        let digit_len = self.payload.len() as u64;
        // Payload layout: [8-byte big-endian digit_len][packed quaternary bytes]
        let total_payload_len = 8 + packed.len() as u64;
        let full_box_size = 8 + total_payload_len;

        if full_box_size > u32::MAX as u64 {
            let large_size = 16 + total_payload_len;
            writer.write_all(&1u32.to_be_bytes())?;
            writer.write_all(self.box_type.as_bytes())?;
            writer.write_all(&large_size.to_be_bytes())?;
        } else {
            writer.write_all(&(full_box_size as u32).to_be_bytes())?;
            writer.write_all(self.box_type.as_bytes())?;
        }

        writer.write_all(&digit_len.to_be_bytes())?;
        writer.write_all(packed)?;
        Ok(())
    }

    /// Scans a seekable MP4 stream, skipping non-`quat` boxes until the first `quat` box is found.
    pub fn read_next_quat_from<R: Read + Seek>(reader: &mut R) -> Result<Self, Mp4QuatError> {
        loop {
            let mut size_buf = [0u8; 4];
            if let Err(e) = reader.read_exact(&mut size_buf) {
                return Err(if e.kind() == io::ErrorKind::UnexpectedEof {
                    Mp4QuatError::UnexpectedEof
                } else {
                    Mp4QuatError::IoError(e.to_string())
                });
            }

            let mut type_buf = [0u8; 4];
            reader
                .read_exact(&mut type_buf)
                .map_err(|e| Mp4QuatError::IoError(e.to_string()))?;

            let initial_size = u32::from_be_bytes(size_buf) as u64;
            let (box_size, header_len) = if initial_size == 1 {
                let mut ext_buf = [0u8; 8];
                reader
                    .read_exact(&mut ext_buf)
                    .map_err(|e| Mp4QuatError::IoError(e.to_string()))?;
                (u64::from_be_bytes(ext_buf), 16u64)
            } else if initial_size == 0 {
                return Err(Mp4QuatError::InvalidBoxSize);
            } else {
                (initial_size, 8u64)
            };

            if box_size < header_len {
                return Err(Mp4QuatError::InvalidBoxSize);
            }

            let box_type = Mp4BoxType::from(type_buf);
            let body_len = box_size - header_len;

            if matches!(box_type, Mp4BoxType::QuaternaryData) {
                if body_len < 8 {
                    return Err(Mp4QuatError::InvalidBoxSize);
                }
                let mut len_buf = [0u8; 8];
                reader
                    .read_exact(&mut len_buf)
                    .map_err(|e| Mp4QuatError::IoError(e.to_string()))?;
                let digit_len = u64::from_be_bytes(len_buf) as usize;
                let packed_len = (body_len - 8) as usize;

                let mut data = vec![0u8; packed_len];
                reader
                    .read_exact(&mut data)
                    .map_err(|e| Mp4QuatError::IoError(e.to_string()))?;

                return Ok(Self {
                    box_size,
                    box_type,
                    payload: QuatByteBuffer {
                        data,
                        len: digit_len,
                    },
                });
            } else {
                reader
                    .seek(SeekFrom::Current(body_len as i64))
                    .map_err(|e| Mp4QuatError::IoError(e.to_string()))?;
            }
        }
    }

}

impl fmt::Display for Mp4QuatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBoxSize => write!(f, "Invalid MP4 box size"),
            Self::UnexpectedEof => write!(f, "Unexpected end of stream while reading MP4 box"),
            Self::InvalidQuaternaryData(v) => {
                write!(f, "Invalid quaternary digit value: {v} (expected 0..=3)")
            }
            Self::UnsupportedVersion => write!(f, "Unsupported MP4 box version or flags"),
            Self::IoError(msg) => write!(f, "I/O error: {msg}"),
        }
    }
}

impl StdError for Mp4QuatError {}

impl From<io::Error> for Mp4QuatError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}