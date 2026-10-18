use core::fmt;

/// A borrowed byte cursor. Failed reads leave its position unchanged.
#[derive(Debug)]
pub struct ByteReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ByteReader<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub const fn position(&self) -> usize {
        self.position
    }

    pub const fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    /// Borrows exactly `count` bytes and advances only on success.
    pub fn read_bytes(&mut self, count: usize) -> Result<&'a [u8], ReadError> {
        if count > self.remaining() {
            return Err(ReadError::UnexpectedEof {
                offset: self.position,
                requested: count,
                remaining: self.remaining(),
            });
        }

        let start = self.position;
        self.position += count;
        Ok(&self.bytes[start..self.position])
    }

    /// Reads a big-endian word, as used in DOS level metadata.
    pub fn read_u16_be(&mut self) -> Result<u16, ReadError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }
}

/// A resource read that cannot be satisfied by the remaining input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadError {
    UnexpectedEof {
        offset: usize,
        requested: usize,
        remaining: usize,
    },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnexpectedEof {
                offset,
                requested,
                remaining,
            } => write!(
                f,
                "cannot read {requested} bytes at offset {offset}: {remaining} bytes remain"
            ),
        }
    }
}

impl core::error::Error for ReadError {}

#[cfg(test)]
mod tests {
    use super::{ByteReader, ReadError};

    #[test]
    fn reads_big_endian_words_and_consumes_the_exact_remaining_slice() {
        let mut reader = ByteReader::new(&[0x12, 0x34, 0xab, 0xcd, 0xff]);

        assert_eq!(reader.read_u16_be(), Ok(0x1234));
        assert_eq!(reader.position(), 2);
        assert_eq!(reader.remaining(), 3);
        assert_eq!(reader.read_bytes(3), Ok([0xab, 0xcd, 0xff].as_slice()));
        assert_eq!(reader.position(), 5);
        assert_eq!(reader.remaining(), 0);
        assert_eq!(reader.read_bytes(0), Ok([].as_slice()));
        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn truncated_word_returns_context_and_preserves_the_unread_byte() {
        let mut reader = ByteReader::new(&[0x12, 0x34, 0xab]);
        assert_eq!(reader.read_u16_be(), Ok(0x1234));

        assert_eq!(
            reader.read_u16_be(),
            Err(ReadError::UnexpectedEof {
                offset: 2,
                requested: 2,
                remaining: 1,
            })
        );
        assert_eq!(reader.position(), 2);
        assert_eq!(reader.remaining(), 1);
        assert_eq!(reader.read_bytes(1), Ok([0xab].as_slice()));
    }

    #[test]
    fn empty_and_oversized_reads_return_errors_without_advancing() {
        let mut empty = ByteReader::new(&[]);
        assert_eq!(
            empty.read_bytes(1),
            Err(ReadError::UnexpectedEof {
                offset: 0,
                requested: 1,
                remaining: 0,
            })
        );
        assert_eq!(empty.position(), 0);
        assert_eq!(empty.read_bytes(0), Ok([].as_slice()));

        let mut reader = ByteReader::new(&[0x11, 0xaa]);
        assert_eq!(reader.read_bytes(1), Ok([0x11].as_slice()));
        assert_eq!(
            reader.read_bytes(usize::MAX),
            Err(ReadError::UnexpectedEof {
                offset: 1,
                requested: usize::MAX,
                remaining: 1,
            })
        );
        assert_eq!(reader.position(), 1);
        assert_eq!(reader.read_bytes(1), Ok([0xaa].as_slice()));
    }
}
