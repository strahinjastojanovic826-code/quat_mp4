use core::fmt;

/// Predstavlja greške koje se mogu pojaviti pri radu sa `QuatDigit` i `QuatBuffer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuatError {
    /// Indeks je van opsega buffera.
    OutOfBounds { index: usize, len: usize },
    /// Vrijednost bitova je nevažeća (dozvoljeno je 0..=3).
    InvalidBits(u8),
    /// Karakter se ne može konvertovati u kvatarnu cifru.
    InvalidChar(char),
}

impl fmt::Display for QuatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfBounds { index, len } => {
                write!(f, "Indeks {index} je van opsega za buffer dužine {len}")
            }
            Self::InvalidBits(bits) => {
                write!(f, "Nevažeća 2-bitna vrijednost: {bits:#b} (dozvoljeno 0b00 do 0b11)")
            }
            Self::InvalidChar(c) => {
                write!(f, "Nevažeći karakter za kvatarnu cifru: '{c}' (dozvoljeni '0'-'3')")
            }
        }
    }
}

impl std::error::Error for QuatError {}