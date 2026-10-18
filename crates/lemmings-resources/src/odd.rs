use crate::{
    LevelError,
    level::{LevelProperties, read_properties},
};
use core::{fmt, str::Utf8Error};

/// Twelve big-endian words followed by an uninterpreted 32-byte title.
pub const ODD_RECORD_SIZE: usize = 56;

/// A validated override borrowing its raw record and title.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OddRecord<'a> {
    bytes: &'a [u8],
    properties: LevelProperties,
}

impl<'a> OddRecord<'a> {
    pub const fn properties(&self) -> LevelProperties {
        self.properties
    }
    pub fn title_bytes(&self) -> &'a [u8] {
        &self.bytes[24..]
    }
    pub fn title(&self) -> Result<&'a str, Utf8Error> {
        core::str::from_utf8(self.title_bytes())
    }
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }
}

/// Borrowed ODD table; parsing validates every record, including unused entries.
/// Empty tables are allowed. Length must be a multiple of 56, and every initial
/// property must satisfy the same supported ranges as a base-level record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OddTable<'a> {
    bytes: &'a [u8],
}

impl<'a> OddTable<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, OddError> {
        if !bytes.len().is_multiple_of(ODD_RECORD_SIZE) {
            return Err(OddError::Length {
                actual: bytes.len(),
                record_size: ODD_RECORD_SIZE,
            });
        }
        for (index, record) in bytes.chunks_exact(ODD_RECORD_SIZE).enumerate() {
            read_properties(record).map_err(|error| OddError::InvalidRecord { index, error })?;
        }
        Ok(Self { bytes })
    }
    pub const fn len(&self) -> usize {
        self.bytes.len() / ODD_RECORD_SIZE
    }
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }
    /// Indexes the base-record number, not a displayed catalog position.
    pub fn get(&self, index: usize) -> Result<OddRecord<'a>, OddError> {
        if index >= self.len() {
            return Err(OddError::Index {
                index,
                count: self.len(),
            });
        }
        // The bounds check proves the multiplication and end offset fit input.
        let start = index * ODD_RECORD_SIZE;
        let bytes = &self.bytes[start..start + ODD_RECORD_SIZE];
        let properties =
            read_properties(bytes).map_err(|error| OddError::InvalidRecord { index, error })?;
        Ok(OddRecord { bytes, properties })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OddError {
    Length { actual: usize, record_size: usize },
    Index { index: usize, count: usize },
    InvalidRecord { index: usize, error: LevelError },
}
impl fmt::Display for OddError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Length {
                actual,
                record_size,
            } => write!(
                f,
                "ODD table length {actual} is not a multiple of {record_size}"
            ),
            Self::Index { index, count } => {
                write!(f, "ODD index {index} is outside {count} records")
            }
            Self::InvalidRecord { index, error } => write!(f, "ODD record {index}: {error}"),
        }
    }
}
impl core::error::Error for OddError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::InvalidRecord { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use crate::LevelField;
    use std::vec;

    fn record() -> [u8; 56] {
        let mut b = [0; 56];
        for (i, w) in [88u16, 80, 76, 5, 1, 2, 3, 4, 5, 6, 7, 8]
            .iter()
            .enumerate()
        {
            b[2 * i..2 * i + 2].copy_from_slice(&w.to_be_bytes());
        }
        b[24..].fill(b' ');
        b[24..27].copy_from_slice(b"ODD");
        b
    }

    #[test]
    fn odd_records_borrow_big_endian_properties_and_all_title_bytes() {
        let bytes = record();
        let table = OddTable::parse(&bytes).unwrap();
        assert_eq!(table.len(), 1);
        assert!(!table.is_empty());
        assert_eq!(table.as_bytes().as_ptr(), bytes.as_ptr());
        let row = table.get(0).unwrap();
        assert_eq!(
            row.properties(),
            LevelProperties {
                release_rate: 88,
                total: 80,
                required: 76,
                minutes: 5,
                skills: [1, 2, 3, 4, 5, 6, 7, 8]
            }
        );
        assert_eq!(row.as_bytes(), bytes);
        assert_eq!(row.title_bytes().as_ptr(), bytes[24..].as_ptr());
        assert_eq!(row.title().unwrap().len(), 32);
        assert_eq!(row.title().unwrap().trim(), "ODD");
    }

    #[test]
    fn empty_tables_and_huge_indexes_report_bounds_without_overflow() {
        let empty = OddTable::parse(&[]).unwrap();
        assert!(empty.is_empty());
        assert_eq!(empty.get(0), Err(OddError::Index { index: 0, count: 0 }));
        let bytes = record();
        let table = OddTable::parse(&bytes).unwrap();
        for index in [1, 80, usize::MAX] {
            assert_eq!(table.get(index), Err(OddError::Index { index, count: 1 }));
        }
    }

    #[test]
    fn partial_records_are_rejected_including_after_complete_records() {
        for prefix in [0, 56, 112] {
            for remainder in 1..56 {
                let actual = prefix + remainder;
                assert_eq!(
                    OddTable::parse(&vec![0; actual]),
                    Err(OddError::Length {
                        actual,
                        record_size: 56
                    })
                );
            }
        }
    }

    #[test]
    fn invalid_numeric_fields_in_any_record_reject_the_entire_table() {
        for (field_index, field, value, min, max) in [
            (0, LevelField::ReleaseRate, 0, 1, 99),
            (0, LevelField::ReleaseRate, 100, 1, 99),
            (1, LevelField::Total, 0, 1, 255),
            (1, LevelField::Total, 256, 1, 255),
            (2, LevelField::Required, 0, 1, 80),
            (2, LevelField::Required, 81, 1, 80),
            (3, LevelField::Minutes, 0, 1, 99),
            (3, LevelField::Minutes, 100, 1, 99),
        ] {
            let mut bytes = record().repeat(2);
            let offset = field_index * 2;
            bytes[56 + offset..58 + offset].copy_from_slice(&u16::to_be_bytes(value));
            assert_eq!(
                OddTable::parse(&bytes),
                Err(OddError::InvalidRecord {
                    index: 1,
                    error: LevelError::InvalidValue {
                        offset,
                        field,
                        value,
                        min,
                        max
                    }
                })
            );
        }
        for index in 0..8 {
            let mut bytes = record();
            let offset = 8 + index * 2;
            bytes[offset..offset + 2].copy_from_slice(&100u16.to_be_bytes());
            assert_eq!(
                OddTable::parse(&bytes),
                Err(OddError::InvalidRecord {
                    index: 0,
                    error: LevelError::InvalidValue {
                        offset,
                        field: LevelField::Skill(index),
                        value: 100,
                        min: 0,
                        max: 99
                    }
                })
            );
        }
    }

    #[test]
    fn valid_extremes_and_multiple_records_do_not_normalize_titles() {
        let mut bytes = record().repeat(2);
        let words = [1u16, 255, 255, 99, 99, 99, 99, 99, 99, 99, 99, 99];
        for (i, w) in words.iter().enumerate() {
            bytes[56 + 2 * i..58 + 2 * i].copy_from_slice(&w.to_be_bytes());
        }
        bytes[56 + 24] = 0;
        bytes[111] = 0xff;
        let table = OddTable::parse(&bytes).unwrap();
        let last = table.get(1).unwrap();
        assert_eq!(last.properties().total, 255);
        assert_eq!(last.properties().skills, [99; 8]);
        assert_eq!(last.title_bytes()[0], 0);
        assert_eq!(last.title_bytes()[31], 0xff);
        assert!(last.title().is_err());
        assert_eq!(table.get(0).unwrap().title().unwrap().trim(), "ODD");
    }
}
