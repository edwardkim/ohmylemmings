use core::fmt;

use crate::DatSection;

// Pinned DOS algorithm and strict input-consumption policy:
// mydocs/tech_insights/issue-9_dat_decompression.md.
impl DatSection<'_> {
    /// Decompresses into a buffer of exactly the declared output size.
    ///
    /// Decodes all six DOS command forms, writing from the end backwards. Every
    /// declared meaningful input bit must be consumed. High padding bits in the
    /// final payload byte (all eight when `initial_bits == 0`) are ignored.
    ///
    /// An output-size mismatch leaves the buffer unchanged. Other errors may
    /// leave partial or complete output: discard the buffer's contents on error.
    /// This method allocates nothing and performs no I/O.
    ///
    /// ```
    /// use lemmings_resources::DatSections;
    ///
    /// // 00 / 000 / 01000001: one literal 'A'.
    /// let bytes = [5, 0x82, 0, 0, 0, 1, 0, 0, 0, 12, 0x82, 0];
    /// let mut output = [0];
    /// let section = DatSections::new(&bytes).next().unwrap()?;
    /// section.decompress_into(&mut output)?;
    /// assert_eq!(&output, b"A");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn decompress_into(&self, output: &mut [u8]) -> Result<(), DecompressError> {
        let offset = self.offset();
        let expected = usize::from(self.header().decompressed_size);
        if output.len() != expected {
            return Err(DecompressError::OutputSize {
                offset,
                expected,
                actual: output.len(),
            });
        }
        let mut bits = ReverseBits::new(self.payload(), self.header().initial_bits, offset);
        let mut cursor = expected;
        while cursor > 0 {
            let command_bit = bits.consumed;
            let (length, distance) = if bits.read(1)? == 0 {
                if bits.read(1)? == 0 {
                    (usize::from(bits.read(3)?) + 1, None)
                } else {
                    (2, Some(usize::from(bits.read(8)?) + 1))
                }
            } else {
                match bits.read(2)? {
                    0 => (3, Some(usize::from(bits.read(9)?) + 1)),
                    1 => (4, Some(usize::from(bits.read(10)?) + 1)),
                    2 => {
                        let length = usize::from(bits.read(8)?) + 1;
                        (length, Some(usize::from(bits.read(12)?) + 1))
                    }
                    _ => (usize::from(bits.read(8)?) + 9, None),
                }
            };
            if length > cursor {
                return Err(DecompressError::OutputOverrun {
                    offset,
                    command_bit,
                    length,
                    remaining: cursor,
                });
            }
            if let Some(distance) = distance {
                let produced = expected - cursor;
                if distance > produced {
                    return Err(DecompressError::InvalidBackReference {
                        offset,
                        command_bit,
                        distance,
                        produced,
                    });
                }
                // Validate against the initialized suffix before copying.
                // Decreasing the cursor makes overlapping distance-one copies
                // reuse newly written bytes; every source stays within output.
                for _ in 0..length {
                    cursor -= 1;
                    output[cursor] = output[cursor + distance];
                }
            } else {
                for _ in 0..length {
                    let byte = bits.read(8)? as u8;
                    cursor -= 1;
                    output[cursor] = byte;
                }
            }
        }
        let remaining = bits.remaining();
        if remaining != 0 {
            return Err(DecompressError::TrailingBits {
                offset,
                bit_offset: bits.consumed,
                remaining,
            });
        }
        Ok(())
    }
}

/// Decode errors; `offset` is the section header's file offset.
///
/// Bit positions count consumed meaningful bits in reverse read order, starting
/// at zero; they are not physical byte offsets within the file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecompressError {
    OutputSize {
        offset: usize,
        expected: usize,
        actual: usize,
    },
    UnexpectedEnd {
        offset: usize,
        bit_offset: usize,
        requested: u8,
        remaining: usize,
    },
    OutputOverrun {
        offset: usize,
        command_bit: usize,
        length: usize,
        remaining: usize,
    },
    InvalidBackReference {
        offset: usize,
        command_bit: usize,
        distance: usize,
        produced: usize,
    },
    TrailingBits {
        offset: usize,
        bit_offset: usize,
        remaining: usize,
    },
}

impl fmt::Display for DecompressError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::OutputSize {
                offset,
                expected,
                actual,
            } => write!(
                f,
                "DAT section at offset {offset} needs an output buffer of {expected} bytes, got {actual}"
            ),
            Self::UnexpectedEnd {
                offset,
                bit_offset,
                requested,
                remaining,
            } => write!(
                f,
                "DAT section at offset {offset} needs {requested} bits at bit {bit_offset}: {remaining} remain"
            ),
            Self::OutputOverrun {
                offset,
                command_bit,
                length,
                remaining,
            } => write!(
                f,
                "DAT section at offset {offset}, command at bit {command_bit}, writes {length} bytes: {remaining} output bytes remain"
            ),
            Self::InvalidBackReference {
                offset,
                command_bit,
                distance,
                produced,
            } => write!(
                f,
                "DAT section at offset {offset}, command at bit {command_bit}, references distance {distance}: only {produced} bytes were produced"
            ),
            Self::TrailingBits {
                offset,
                bit_offset,
                remaining,
            } => write!(
                f,
                "DAT section at offset {offset} filled its output at bit {bit_offset}: {remaining} meaningful bits remain"
            ),
        }
    }
}

impl core::error::Error for DecompressError {}

struct ReverseBits<'a> {
    bytes: &'a [u8],
    index: usize,
    current: u8,
    left: u8,
    consumed: usize,
    offset: usize,
}

impl<'a> ReverseBits<'a> {
    fn new(bytes: &'a [u8], initial_bits: u8, offset: usize) -> Self {
        // Only constructed from a validated DatSection outside unit tests.
        debug_assert!(!bytes.is_empty() && initial_bits <= 8);
        let index = bytes.len() - 1;
        Self {
            bytes,
            index,
            current: bytes[index],
            left: initial_bits,
            consumed: 0,
            offset,
        }
    }

    fn remaining(&self) -> usize {
        self.index * 8 + usize::from(self.left)
    }

    fn read(&mut self, count: u8) -> Result<u16, DecompressError> {
        let remaining = self.remaining();
        if usize::from(count) > remaining {
            return Err(DecompressError::UnexpectedEnd {
                offset: self.offset,
                bit_offset: self.consumed,
                requested: count,
                remaining,
            });
        }
        // Decoder fields contain at most twelve bits. The preflight check makes
        // each read atomic and prevents loading a byte before the payload start.
        let mut value = 0;
        for _ in 0..count {
            if self.left == 0 {
                self.index -= 1;
                self.current = self.bytes[self.index];
                self.left = 8;
            }
            value = (value << 1) | u16::from(self.current & 1);
            self.current >>= 1;
            self.left -= 1;
        }
        self.consumed += usize::from(count);
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::{DecompressError, ReverseBits};
    use crate::{DatSection, DatSections};
    use std::{format, string::String, vec, vec::Vec};

    // Hand-written golden vector: 00 / 000 / 01000001 -> literal 'A'.
    const GOLDEN_A: [u8; 12] = [5, 0x82, 0, 0, 0, 1, 0, 0, 0, 12, 0x82, 0];
    const A_BITS: &str = "0000001000001";

    fn section(bytes: &[u8]) -> DatSection<'_> {
        DatSections::new(bytes).next().unwrap().unwrap()
    }

    // Test-only framing, using command bit strings in their read order.
    fn frame(bits: &str, output_size: u16) -> Vec<u8> {
        assert!(bits.bytes().all(|b| b == b'0' || b == b'1'));
        let mut payload: Vec<u8> = bits
            .as_bytes()
            .rchunks(8)
            .map(|chunk| {
                chunk
                    .iter()
                    .enumerate()
                    .fold(0, |value, (i, b)| value | ((b - b'0') << i))
            })
            .collect();
        let initial = if bits.is_empty() {
            payload.push(0);
            0
        } else {
            ((bits.len() - 1) % 8 + 1) as u8
        };
        let checksum = payload.iter().fold(0, |a, b| a ^ b);
        let mut bytes = vec![initial, checksum, 0, 0];
        bytes.extend(output_size.to_be_bytes());
        bytes.extend([0, 0]);
        bytes.extend(u16::try_from(10 + payload.len()).unwrap().to_be_bytes());
        bytes.extend(payload);
        bytes
    }

    fn literal(bytes: &[u8]) -> String {
        let mut bits = if bytes.len() <= 8 {
            format!("00{:03b}", bytes.len() - 1)
        } else {
            format!("111{:08b}", bytes.len() - 9)
        };
        for byte in bytes.iter().rev() {
            bits.push_str(&format!("{byte:08b}"));
        }
        bits
    }

    fn literals(bytes: &[u8]) -> String {
        bytes.rchunks(264).map(literal).collect()
    }

    #[test]
    fn hand_written_literal_vector_decodes_into_the_callers_buffer() {
        assert_eq!(frame(A_BITS, 1), GOLDEN_A);
        let mut output = [0xcc];
        assert_eq!(section(&GOLDEN_A).decompress_into(&mut output), Ok(()));
        assert_eq!(&output, b"A");
    }

    #[test]
    fn every_short_literal_length_preserves_byte_order() {
        for length in 1..=8 {
            let expected: Vec<u8> = (0..length).map(|i| i as u8 * 31).collect();
            let bytes = frame(&literal(&expected), length as u16);
            let mut output = vec![0xcc; length];
            assert_eq!(section(&bytes).decompress_into(&mut output), Ok(()));
            assert_eq!(output, expected);
        }
    }

    #[test]
    fn every_long_literal_length_including_264_is_supported() {
        for length in 9..=264 {
            let expected: Vec<u8> = (0..length).map(|i| (i % 256) as u8).collect();
            let bytes = frame(&literal(&expected), length as u16);
            let mut output = vec![0xcc; length];
            assert_eq!(section(&bytes).decompress_into(&mut output), Ok(()));
            assert_eq!(output, expected);
        }
    }

    #[test]
    fn fixed_length_references_copy_nonrepeating_suffixes() {
        for (code, width, expected) in [
            ("01", 8, b"CDABCD".as_slice()),
            ("100", 9, b"BCDABCD".as_slice()),
            ("101", 10, b"ABCDABCD".as_slice()),
        ] {
            let bits = literal(b"ABCD") + code + &format!("{:0width$b}", 3);
            let bytes = frame(&bits, expected.len() as u16);
            let mut output = vec![0xcc; expected.len()];
            assert_eq!(section(&bytes).decompress_into(&mut output), Ok(()));
            assert_eq!(output, expected);
        }
    }

    #[test]
    fn fixed_length_references_allow_overlapping_distance_one() {
        for (code, width, length) in [("01", 8, 2), ("100", 9, 3), ("101", 10, 4)] {
            let bits = String::from(A_BITS) + code + &"0".repeat(width);
            let bytes = frame(&bits, (length + 1) as u16);
            let mut output = vec![0xcc; length + 1];
            assert_eq!(section(&bytes).decompress_into(&mut output), Ok(()));
            assert_eq!(output, vec![b'A'; length + 1]);
        }
    }

    #[test]
    fn every_variable_reference_length_including_256_can_overlap() {
        for length in 1..=256 {
            let bits = String::from(A_BITS) + &format!("110{:08b}000000000000", length - 1);
            let bytes = frame(&bits, (length + 1) as u16);
            let mut output = vec![0xcc; length + 1];
            assert_eq!(section(&bytes).decompress_into(&mut output), Ok(()));
            assert_eq!(output, vec![b'A'; length + 1]);
        }
    }

    #[test]
    fn each_reference_format_supports_its_maximum_distance() {
        for (code, width, length) in [
            ("01", 8, 2),
            ("100", 9, 3),
            ("101", 10, 4),
            ("11000000000", 12, 1),
        ] {
            let distance = 1usize << width;
            let seed: Vec<u8> = (0..distance).map(|i| (i % 251) as u8).collect();
            let bits = literals(&seed) + code + &format!("{:0width$b}", distance - 1);
            let bytes = frame(&bits, (distance + length) as u16);
            let mut expected = seed[distance - length..].to_vec();
            expected.extend(seed);
            let mut output = vec![0xcc; expected.len()];
            assert_eq!(section(&bytes).decompress_into(&mut output), Ok(()));
            assert_eq!(output, expected);
        }
    }

    #[test]
    fn byte_aligned_streams_accept_eight_bits_or_a_zero_bit_sentinel() {
        let expected = b"12345678";
        let bits: String = expected.iter().rev().map(|b| literal(&[*b])).collect();
        let mut bytes = frame(&bits, 8);
        assert_eq!(bytes[0], 8);
        let mut output = [0xcc; 8];
        assert_eq!(section(&bytes).decompress_into(&mut output), Ok(()));
        assert_eq!(&output, expected);

        // The zero-bit final byte is skipped but still participates in XOR.
        bytes.push(0x69);
        bytes[0] = 0;
        bytes[1] ^= 0x69;
        let size = bytes.len() as u16;
        bytes[8..10].copy_from_slice(&size.to_be_bytes());
        output.fill(0xcc);
        assert_eq!(section(&bytes).decompress_into(&mut output), Ok(()));
        assert_eq!(&output, expected);
    }

    #[test]
    fn output_size_mismatch_does_not_modify_the_buffer() {
        for actual in [0, 2, 64] {
            let mut output = vec![0xcc; actual];
            assert_eq!(
                section(&GOLDEN_A).decompress_into(&mut output),
                Err(DecompressError::OutputSize {
                    offset: 0,
                    expected: 1,
                    actual,
                })
            );
            assert_eq!(output, vec![0xcc; actual]);
        }
    }

    #[test]
    fn every_truncation_of_a_mixed_command_stream_is_an_error() {
        let bits = literal(b"ABCDEFGHI")
            + "0100000000"
            + "100000000000"
            + "1010000000000"
            + "11000000100000000000000"
            + &literal(b"Z");
        let bytes = frame(&bits, 24);
        let mut output = [0xcc; 24];
        assert_eq!(section(&bytes).decompress_into(&mut output), Ok(()));
        assert_eq!(&output[..1], b"Z");
        assert_eq!(&output[1..15], &[b'A'; 14]);
        assert_eq!(&output[15..], b"ABCDEFGHI");
        for end in 0..bits.len() {
            let bytes = frame(&bits[..end], 24);
            assert!(
                matches!(
                    section(&bytes).decompress_into(&mut output),
                    Err(DecompressError::UnexpectedEnd { .. })
                ),
                "prefix {end}"
            );
        }
    }

    #[test]
    fn literal_commands_cannot_overrun_the_declared_output() {
        for expected in [b"AB".as_slice(), b"ABCDEFGHI".as_slice()] {
            let remaining = expected.len() - 1;
            let bytes = frame(&literal(expected), remaining as u16);
            let mut output = vec![0xcc; remaining];
            assert_eq!(
                section(&bytes).decompress_into(&mut output),
                Err(DecompressError::OutputOverrun {
                    offset: 0,
                    command_bit: 0,
                    length: expected.len(),
                    remaining,
                })
            );
            assert_eq!(output, vec![0xcc; remaining]);
        }
    }

    #[test]
    fn reference_commands_cannot_overrun_and_preserve_the_previous_suffix() {
        let bytes = frame(&(String::from(A_BITS) + "0100000000"), 2);
        let mut output = [0xcc; 2];
        assert_eq!(
            section(&bytes).decompress_into(&mut output),
            Err(DecompressError::OutputOverrun {
                offset: 0,
                command_bit: 13,
                length: 2,
                remaining: 1,
            })
        );
        assert_eq!(output, [0xcc, b'A']);
    }

    #[test]
    fn references_cannot_read_caller_bytes_before_any_output_is_produced() {
        for bits in [
            "0100000000",
            "100000000000",
            "1010000000000",
            "11000000000000000000000",
        ] {
            let bytes = frame(bits, 4);
            let mut output = [0xcc; 4];
            assert_eq!(
                section(&bytes).decompress_into(&mut output),
                Err(DecompressError::InvalidBackReference {
                    offset: 0,
                    command_bit: 0,
                    distance: 1,
                    produced: 0,
                })
            );
            assert_eq!(output, [0xcc; 4]);
        }
    }

    #[test]
    fn references_must_stay_within_the_suffix_already_written() {
        let bytes = frame(&(String::from(A_BITS) + "0100000001"), 3);
        let mut output = [0xcc; 3];
        assert_eq!(
            section(&bytes).decompress_into(&mut output),
            Err(DecompressError::InvalidBackReference {
                offset: 0,
                command_bit: 13,
                distance: 2,
                produced: 1,
            })
        );
        assert_eq!(output, [0xcc, 0xcc, b'A']);
    }

    #[test]
    fn unused_meaningful_bits_are_rejected_even_when_zero() {
        for tail in ["0", "101", "00000000"] {
            let bytes = frame(&(String::from(A_BITS) + tail), 1);
            let mut output = [0xcc];
            assert_eq!(
                section(&bytes).decompress_into(&mut output),
                Err(DecompressError::TrailingBits {
                    offset: 0,
                    bit_offset: 13,
                    remaining: tail.len(),
                })
            );
            assert_eq!(&output, b"A");
        }
    }

    #[test]
    fn errors_report_the_later_sections_file_offset() {
        let mut bytes = GOLDEN_A.to_vec();
        bytes.extend(frame("0", 1));
        let mut sections = DatSections::new(&bytes);
        let mut output = [0xcc];
        assert_eq!(
            sections
                .next()
                .unwrap()
                .unwrap()
                .decompress_into(&mut output),
            Ok(())
        );
        assert_eq!(
            sections
                .next()
                .unwrap()
                .unwrap()
                .decompress_into(&mut output),
            Err(DecompressError::UnexpectedEnd {
                offset: 12,
                bit_offset: 1,
                requested: 1,
                remaining: 0,
            })
        );
        assert_eq!(sections.next(), None);
    }

    #[test]
    fn maximum_output_size_and_repeated_decodes_are_supported() {
        let mut bits = String::from(A_BITS);
        let mut remaining = usize::from(u16::MAX) - 1;
        while remaining > 0 {
            let length = remaining.min(256);
            bits += &format!("110{:08b}000000000000", length - 1);
            remaining -= length;
        }
        let bytes = frame(&bits, u16::MAX);
        let view = section(&bytes);
        for initial in [0xcc, 0x55] {
            let mut output = vec![initial; usize::from(u16::MAX)];
            assert_eq!(view.decompress_into(&mut output), Ok(()));
            assert!(output.iter().all(|b| *b == b'A'));
        }
    }

    #[test]
    fn reverse_bits_cross_byte_boundaries_without_reversing_field_significance() {
        let mut reader = ReverseBits::new(&[0xa6, 0x05], 3, 0);
        assert_eq!(reader.read(5), Ok(0b10101));
        assert_eq!(reader.read(6), Ok(0b100101));
        assert_eq!(reader.remaining(), 0);
        assert_eq!(reader.consumed, 11);
    }

    #[test]
    fn reverse_bits_support_every_initial_bit_count_and_ignore_high_padding() {
        for initial in 0..=8 {
            let mut reader = ReverseBits::new(&[0x96, 0xca], initial, 0);
            let expected = String::from(&"01010011"[..initial as usize]) + "01101001";
            for digit in expected.bytes() {
                assert_eq!(reader.read(1), Ok(u16::from(digit - b'0')));
            }
            assert_eq!(reader.remaining(), 0);
            assert_eq!(
                reader.read(1),
                Err(DecompressError::UnexpectedEnd {
                    offset: 0,
                    bit_offset: expected.len(),
                    requested: 1,
                    remaining: 0,
                })
            );
        }
    }

    #[test]
    fn failed_bit_reads_are_atomic_and_include_context() {
        let mut reader = ReverseBits::new(&[0xa6, 0x05], 3, 42);
        assert_eq!(
            reader.read(12),
            Err(DecompressError::UnexpectedEnd {
                offset: 42,
                bit_offset: 0,
                requested: 12,
                remaining: 11,
            })
        );
        assert_eq!(reader.consumed, 0);
        assert_eq!(reader.remaining(), 11);
        assert_eq!(reader.read(3), Ok(5));
        assert_eq!(reader.read(8), Ok(0x65));
        assert_eq!(
            reader.read(1),
            Err(DecompressError::UnexpectedEnd {
                offset: 42,
                bit_offset: 11,
                requested: 1,
                remaining: 0,
            })
        );
    }
}
