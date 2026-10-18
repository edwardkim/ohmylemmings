use core::{fmt, iter::FusedIterator};

use crate::{ByteReader, ReadError};

// DOS TCompressionHeaderRec: two bytes followed by four big-endian words.
// See mydocs/tech_insights/issue-8_dat_sections.md for pinned references.
const HEADER_SIZE: usize = 10;

/// Metadata from a validated DOS DAT section header.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DatHeader {
    /// Meaningful bits in the last payload byte; zero is valid. Range: 0..=8.
    pub initial_bits: u8,
    /// XOR of every compressed payload byte, excluding the header.
    pub checksum: u8,
    /// Declared output length; command-stream validity is checked by decoding.
    pub decompressed_size: u16,
    /// Total encoded length, including the ten-byte header.
    pub section_size: u16,
}

/// A section borrowing its compressed payload from the original input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DatSection<'a> {
    offset: usize,
    header: DatHeader,
    payload: &'a [u8],
}

impl<'a> DatSection<'a> {
    pub const fn offset(&self) -> usize {
        self.offset
    }

    pub const fn header(&self) -> DatHeader {
        self.header
    }

    pub const fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

/// Validates consecutive sections, returning one error before stopping.
///
/// Empty input contains no sections. Each section must have a nonempty payload
/// and output, zero reserved words, an initial bit count in `0..=8`, and a valid
/// payload XOR. This validates framing, not compressed commands or output bytes.
/// Exhaust the iterator to validate the entire file, including trailing bytes.
///
/// ```
/// use lemmings_resources::DatSections;
///
/// // Synthetic framing example; not a decodable command stream.
/// let bytes = [8, 0xff, 0, 0, 0, 1, 0, 0, 0, 11, 0xff];
/// for section in DatSections::new(&bytes) {
///     let section = section?;
///     assert_eq!(section.offset(), 0);
///     assert_eq!(section.header().section_size, 11);
///     assert_eq!(section.payload(), &[0xff]);
/// }
/// # Ok::<(), lemmings_resources::DatError>(())
/// ```
#[derive(Debug)]
pub struct DatSections<'a> {
    reader: ByteReader<'a>,
    finished: bool,
}

impl<'a> DatSections<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self {
            reader: ByteReader::new(bytes),
            finished: false,
        }
    }

    fn read_section(&mut self) -> Result<DatSection<'a>, DatError> {
        let offset = self.reader.position();
        let remaining = self.reader.remaining();
        let truncated_header = |_: ReadError| DatError::TruncatedHeader { offset, remaining };
        let mut header = ByteReader::new(
            self.reader
                .read_bytes(HEADER_SIZE)
                .map_err(truncated_header)?,
        );
        let prefix = header.read_bytes(2).map_err(truncated_header)?;
        let initial_bits = prefix[0];
        let checksum = prefix[1];
        let first = header.read_u16_be().map_err(truncated_header)?;
        let decompressed_size = header.read_u16_be().map_err(truncated_header)?;
        let second = header.read_u16_be().map_err(truncated_header)?;
        let section_size = header.read_u16_be().map_err(truncated_header)?;

        if first != 0 || second != 0 {
            return Err(DatError::UnsupportedReservedWords {
                offset,
                first,
                second,
            });
        }
        if initial_bits > 8 {
            return Err(DatError::InvalidBitCount {
                offset,
                found: initial_bits,
            });
        }
        if usize::from(section_size) <= HEADER_SIZE {
            return Err(DatError::InvalidSectionSize {
                offset,
                size: section_size,
            });
        }
        if decompressed_size == 0 {
            return Err(DatError::EmptyOutput { offset });
        }

        // Subtract only after checking the header-inclusive size. ByteReader
        // checks remaining input before advancing, including on 32-bit targets.
        let expected = usize::from(section_size) - HEADER_SIZE;
        let remaining = self.reader.remaining();
        let payload = self
            .reader
            .read_bytes(expected)
            .map_err(|_| DatError::TruncatedPayload {
                offset,
                expected,
                remaining,
            })?;
        let actual = payload.iter().fold(0, |xor, byte| xor ^ byte);
        if actual != checksum {
            return Err(DatError::ChecksumMismatch {
                offset,
                expected: checksum,
                actual,
            });
        }

        Ok(DatSection {
            offset,
            header: DatHeader {
                initial_bits,
                checksum,
                decompressed_size,
                section_size,
            },
            payload,
        })
    }
}

impl<'a> Iterator for DatSections<'a> {
    type Item = Result<DatSection<'a>, DatError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished || self.reader.remaining() == 0 {
            self.finished = true;
            return None;
        }
        let section = self.read_section();
        if section.is_err() {
            self.finished = true;
        }
        Some(section)
    }
}

impl FusedIterator for DatSections<'_> {}

/// Structural errors; `offset` always identifies the section's header start.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatError {
    TruncatedHeader {
        offset: usize,
        remaining: usize,
    },
    InvalidBitCount {
        offset: usize,
        found: u8,
    },
    UnsupportedReservedWords {
        offset: usize,
        first: u16,
        second: u16,
    },
    InvalidSectionSize {
        offset: usize,
        size: u16,
    },
    EmptyOutput {
        offset: usize,
    },
    TruncatedPayload {
        offset: usize,
        expected: usize,
        remaining: usize,
    },
    ChecksumMismatch {
        offset: usize,
        expected: u8,
        actual: u8,
    },
}

impl fmt::Display for DatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::TruncatedHeader { offset, remaining } => write!(
                f,
                "DAT header at offset {offset} needs {HEADER_SIZE} bytes: {remaining} remain"
            ),
            Self::InvalidBitCount { offset, found } => write!(
                f,
                "DAT section at offset {offset} has initial bit count {found}, expected 0..=8"
            ),
            Self::UnsupportedReservedWords {
                offset,
                first,
                second,
            } => write!(
                f,
                "DAT section at offset {offset} has unsupported reserved words {first:#06x}, {second:#06x}"
            ),
            Self::InvalidSectionSize { offset, size } => write!(
                f,
                "DAT section at offset {offset} has size {size}, expected more than {HEADER_SIZE}"
            ),
            Self::EmptyOutput { offset } => {
                write!(f, "DAT section at offset {offset} declares an empty output")
            }
            Self::TruncatedPayload {
                offset,
                expected,
                remaining,
            } => write!(
                f,
                "DAT section at offset {offset} needs {expected} payload bytes: {remaining} remain"
            ),
            Self::ChecksumMismatch {
                offset,
                expected,
                actual,
            } => write!(
                f,
                "DAT section at offset {offset} has payload XOR {actual:#04x}, expected {expected:#04x}"
            ),
        }
    }
}

impl core::error::Error for DatError {}

#[cfg(test)]
mod tests {
    use super::{DatError, DatHeader, DatSections};
    use core::iter::FusedIterator;

    // Hand-written framing fixtures, not compressed command streams.
    const FIRST: [u8; 13] = [0, 0x70, 0, 0, 0x12, 0x34, 0, 0, 0, 13, 0x12, 0x34, 0x56];
    const SECOND: [u8; 11] = [8, 0xff, 0, 0, 0, 1, 0, 0, 0, 11, 0xff];

    fn assert_stops(bytes: &[u8], expected: DatError) {
        let mut sections = DatSections::new(bytes);
        assert_eq!(sections.next(), Some(Err(expected)));
        assert_eq!(sections.next(), None);
        assert_eq!(sections.next(), None);
    }

    #[test]
    fn multiple_sections_preserve_offsets_metadata_and_borrowed_payloads() {
        let mut bytes = [0; 24];
        bytes[..13].copy_from_slice(&FIRST);
        bytes[13..].copy_from_slice(&SECOND);
        let mut sections = DatSections::new(&bytes);

        let first = sections.next().unwrap().unwrap();
        assert_eq!(first.offset(), 0);
        assert_eq!(
            first.header(),
            DatHeader {
                initial_bits: 0,
                checksum: 0x70,
                decompressed_size: 0x1234,
                section_size: 13,
            }
        );
        assert_eq!(first.payload(), &[0x12, 0x34, 0x56]);
        assert_eq!(first.payload().as_ptr(), bytes[10..].as_ptr());

        let second = sections.next().unwrap().unwrap();
        assert_eq!(second.offset(), 13);
        assert_eq!(second.header().initial_bits, 8);
        assert_eq!(second.header().decompressed_size, 1);
        assert_eq!(second.header().section_size, 11);
        assert_eq!(second.payload(), &[0xff]);
        assert_eq!(second.payload().as_ptr(), bytes[23..].as_ptr());
        // Previously returned views remain usable after advancing.
        assert_eq!(first.payload().len(), 3);
        assert_eq!(sections.next(), None);
        assert_eq!(sections.next(), None);
    }

    #[test]
    fn empty_input_is_an_empty_fused_iterator() {
        fn requires_fused<T: FusedIterator>(_: &T) {}
        let mut sections = DatSections::new(&[]);
        requires_fused(&sections);
        assert_eq!(sections.next(), None);
        assert_eq!(sections.next(), None);
    }

    #[test]
    fn every_partial_header_is_rejected() {
        for remaining in 1..10 {
            assert_stops(
                &FIRST[..remaining],
                DatError::TruncatedHeader {
                    offset: 0,
                    remaining,
                },
            );
        }
    }

    #[test]
    fn every_partial_payload_is_rejected() {
        for remaining in 0..3 {
            assert_stops(
                &FIRST[..10 + remaining],
                DatError::TruncatedPayload {
                    offset: 0,
                    expected: 3,
                    remaining,
                },
            );
        }
    }

    #[test]
    fn section_size_must_include_a_header_and_nonempty_payload() {
        for size in 0u16..=10 {
            let mut bytes = FIRST;
            bytes[8..10].copy_from_slice(&size.to_be_bytes());
            assert_stops(&bytes, DatError::InvalidSectionSize { offset: 0, size });
        }
    }

    #[test]
    fn maximum_declared_size_cannot_overrun_the_input() {
        let mut bytes = FIRST;
        bytes[8..10].copy_from_slice(&u16::MAX.to_be_bytes());
        assert_stops(
            &bytes,
            DatError::TruncatedPayload {
                offset: 0,
                expected: 65525,
                remaining: 3,
            },
        );
    }

    #[test]
    fn zero_decompressed_size_is_rejected() {
        let mut bytes = FIRST;
        bytes[4..6].copy_from_slice(&[0, 0]);
        assert_stops(&bytes, DatError::EmptyOutput { offset: 0 });
    }

    #[test]
    fn initial_bits_accept_zero_through_eight_and_reject_every_larger_value() {
        for found in 0..=u8::MAX {
            let mut bytes = FIRST;
            bytes[0] = found;
            if found <= 8 {
                let mut sections = DatSections::new(&bytes);
                assert_eq!(
                    sections.next().unwrap().unwrap().header().initial_bits,
                    found
                );
                assert_eq!(sections.next(), None);
            } else {
                assert_stops(&bytes, DatError::InvalidBitCount { offset: 0, found });
            }
        }
    }

    #[test]
    fn reserved_words_are_not_silently_interpreted_as_32_bit_sizes() {
        for (first, second) in [(0x0100u16, 0u16), (0, 0x8000), (0xffff, 1)] {
            let mut bytes = FIRST;
            bytes[2..4].copy_from_slice(&first.to_be_bytes());
            bytes[6..8].copy_from_slice(&second.to_be_bytes());
            assert_stops(
                &bytes,
                DatError::UnsupportedReservedWords {
                    offset: 0,
                    first,
                    second,
                },
            );
        }
    }

    #[test]
    fn payload_corruption_and_wrong_header_checksum_are_rejected() {
        for position in 10..13 {
            let mut bytes = FIRST;
            bytes[position] ^= 1;
            assert_stops(
                &bytes,
                DatError::ChecksumMismatch {
                    offset: 0,
                    expected: 0x70,
                    actual: 0x71,
                },
            );
        }
        let mut bytes = FIRST;
        bytes[1] = 0x71;
        assert_stops(
            &bytes,
            DatError::ChecksumMismatch {
                offset: 0,
                expected: 0x71,
                actual: 0x70,
            },
        );
    }

    #[test]
    fn a_zero_checksum_is_valid() {
        let bytes = [8, 0, 0, 0, 0, 1, 0, 0, 0, 12, 0xaa, 0xaa];
        let mut sections = DatSections::new(&bytes);
        assert_eq!(sections.next().unwrap().unwrap().payload(), &[0xaa, 0xaa]);
        assert_eq!(sections.next(), None);
    }

    #[test]
    fn trailing_bytes_including_an_exact_header_are_not_ignored() {
        let mut bytes = [0; 23];
        bytes[..13].copy_from_slice(&FIRST);
        for length in 1..=10 {
            let mut sections = DatSections::new(&bytes[..13 + length]);
            assert!(sections.next().unwrap().is_ok());
            let expected = if length < 10 {
                DatError::TruncatedHeader {
                    offset: 13,
                    remaining: length,
                }
            } else {
                DatError::InvalidSectionSize {
                    offset: 13,
                    size: 0,
                }
            };
            assert_eq!(sections.next(), Some(Err(expected)));
            assert_eq!(sections.next(), None);
        }
    }

    #[test]
    fn errors_in_later_sections_use_file_offsets_and_stop_before_valid_suffixes() {
        let mut bytes = [0; 35];
        bytes[..13].copy_from_slice(&FIRST);
        bytes[13..24].copy_from_slice(&SECOND);
        bytes[24..].copy_from_slice(&SECOND);
        bytes[23] ^= 1;
        let mut sections = DatSections::new(&bytes);
        assert!(sections.next().unwrap().is_ok());
        assert_eq!(
            sections.next(),
            Some(Err(DatError::ChecksumMismatch {
                offset: 13,
                expected: 0xff,
                actual: 0xfe,
            }))
        );
        assert_eq!(sections.next(), None);
        assert_eq!(sections.next(), None);
    }

    #[test]
    fn the_largest_16_bit_section_and_output_size_are_supported() {
        let mut bytes = [1; u16::MAX as usize];
        bytes[..10].copy_from_slice(&[8, 1, 0, 0, 0xff, 0xff, 0, 0, 0xff, 0xff]);
        let mut sections = DatSections::new(&bytes);
        let section = sections.next().unwrap().unwrap();
        assert_eq!(section.header().section_size, u16::MAX);
        assert_eq!(section.header().decompressed_size, u16::MAX);
        assert_eq!(section.payload().len(), 65525);
        assert_eq!(section.payload().as_ptr(), bytes[10..].as_ptr());
        assert_eq!(sections.next(), None);
    }
}
