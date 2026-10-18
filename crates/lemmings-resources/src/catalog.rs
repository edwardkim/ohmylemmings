use crate::{
    DatError, DatSection, DatSections, DecompressError, LEVEL_RECORD_SIZE, LevelError, LevelHeader,
    LevelRecord, ObjectPlacement, OddError, OddTable, SteelArea, TerrainPlacement,
};
use core::{fmt, iter::FusedIterator, str::Utf8Error};
use lemmings_core::LevelId;

// Lemmix SectionTable at 40e9bc3; subtract one before splitting parity/index.
// Provenance, independent config comparison and loading policy:
// mydocs/tech_insights/issue-11_level_catalog.md.
const ORDER: [[u8; 30]; 4] = [
    [
        147, 155, 157, 149, 151, 153, 159, 14, 22, 54, 70, 16, 29, 32, 38, 42, 48, 72, 84, 104,
        138, 23, 68, 96, 98, 116, 78, 100, 108, 134,
    ],
    [
        1, 30, 36, 50, 52, 56, 58, 80, 102, 120, 128, 130, 136, 5, 148, 152, 154, 156, 160, 7, 11,
        13, 15, 17, 19, 21, 25, 27, 33, 31,
    ],
    [
        37, 39, 41, 43, 45, 47, 49, 51, 53, 55, 57, 59, 61, 63, 3, 65, 67, 69, 71, 73, 75, 77, 79,
        81, 83, 85, 87, 89, 35, 111,
    ],
    [
        91, 93, 95, 97, 99, 101, 103, 105, 107, 109, 112, 113, 115, 117, 119, 121, 123, 125, 127,
        150, 129, 9, 131, 133, 135, 137, 139, 141, 143, 145,
    ],
];

/// Caller-tagged LEVEL000..009.DAT with all eight sections validated.
/// The tag identifies the supplied file; it does not authenticate its contents.
#[derive(Clone, Copy, Debug)]
pub struct DosLevelFile<'a> {
    index: u8,
    sections: [DatSection<'a>; 8],
}
impl<'a> DosLevelFile<'a> {
    pub fn parse(index: u8, bytes: &'a [u8]) -> Result<Self, CatalogError> {
        if index > 9 {
            return Err(CatalogError::FileIndex { found: index });
        }
        let check = |part, result: Result<DatSection<'a>, DatError>| {
            let section = result.map_err(|error| CatalogError::Dat {
                file: index,
                part,
                error,
            })?;
            if usize::from(section.header().decompressed_size) != LEVEL_RECORD_SIZE {
                return Err(CatalogError::SectionSize {
                    file: index,
                    part,
                    actual: section.header().decompressed_size,
                });
            }
            Ok(section)
        };
        let mut iterator = DatSections::new(bytes);
        let first = check(
            0,
            iterator.next().ok_or(CatalogError::SectionCount {
                file: index,
                actual: 0,
            })?,
        )?;
        let mut sections = [first; 8];
        let mut count = 1;
        for result in iterator {
            let section = check(count, result)?;
            if count < sections.len() {
                sections[count] = section;
            }
            count += 1;
        }
        if count != 8 {
            return Err(CatalogError::SectionCount {
                file: index,
                actual: count,
            });
        }
        Ok(Self { index, sections })
    }
    pub const fn index(&self) -> u8 {
        self.index
    }
}

/// Original catalog routing from a validated, displayed core LevelId.
///
/// ```
/// use lemmings_core::{Difficulty, LevelId};
/// use lemmings_resources::CatalogEntry;
///
/// let entry = CatalogEntry::for_level(LevelId::new(Difficulty::Fun, 8)?);
/// assert_eq!((entry.file_index(), entry.part_index()), (0, 6));
/// assert_eq!(entry.odd_index(), Some(6)); // Not displayed global index 7.
/// # Ok::<(), lemmings_core::InvalidLevelNumber>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogEntry {
    id: LevelId,
    file: u8,
    part: u8,
    odd_index: Option<usize>,
}
impl CatalogEntry {
    pub const fn for_level(id: LevelId) -> Self {
        let encoded = ORDER[id.difficulty().index()][id.index()] - 1;
        let base = encoded / 2;
        Self {
            id,
            file: base / 8,
            part: base % 8,
            odd_index: if encoded & 1 == 1 {
                Some(base as usize)
            } else {
                None
            },
        }
    }
    pub const fn level_id(self) -> LevelId {
        self.id
    }
    pub const fn file_index(self) -> u8 {
        self.file
    }
    pub const fn part_index(self) -> u8 {
        self.part
    }
    pub const fn odd_index(self) -> Option<usize> {
        self.odd_index
    }
    /// Loads into caller-owned memory, preserving base bytes and all placements.
    /// File binding and required ODD data are checked before writing output.
    /// Decompression or base-parse errors may change output: discard it on error.
    pub fn load<'a>(
        &self,
        file: &DosLevelFile<'_>,
        odd: Option<OddTable<'a>>,
        output: &'a mut [u8; LEVEL_RECORD_SIZE],
    ) -> Result<ResolvedLevel<'a>, CatalogError> {
        if file.index != self.file {
            return Err(CatalogError::WrongFile {
                expected: self.file,
                actual: file.index,
            });
        }
        let variant = match self.odd_index {
            Some(index) => Some(
                odd.ok_or(CatalogError::MissingOdd { index })?
                    .get(index)
                    .map_err(|error| CatalogError::Odd { index, error })?,
            ),
            None => None,
        };
        file.sections[usize::from(self.part)]
            .decompress_into(output)
            .map_err(|error| CatalogError::Decompress {
                file: self.file,
                part: self.part,
                error,
            })?;
        let base = LevelRecord::parse(output).map_err(|error| CatalogError::Level {
            file: self.file,
            part: self.part,
            error,
        })?;
        let (header, title) = match variant {
            Some(row) => (row.properties().apply_to(base.header()), row.title_bytes()),
            None => (base.header(), base.title_bytes()),
        };
        Ok(ResolvedLevel {
            entry: *self,
            base,
            header,
            title,
        })
    }
}

/// Effective properties/title plus unchanged borrowed base geometry. This is
/// resource selection only; graphics reconstruction and simulation are separate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedLevel<'a> {
    entry: CatalogEntry,
    base: LevelRecord<'a>,
    header: LevelHeader,
    title: &'a [u8],
}
impl<'a> ResolvedLevel<'a> {
    /// Validated base geometry/header before ODD overrides; graphics selectors
    /// and placements are identical to the effective level. Borrowed, without reparse.
    pub const fn base_record(&self) -> LevelRecord<'a> {
        self.base
    }
    pub const fn entry(&self) -> CatalogEntry {
        self.entry
    }
    pub const fn header(&self) -> LevelHeader {
        self.header
    }
    pub const fn title_bytes(&self) -> &'a [u8] {
        self.title
    }
    pub fn title(&self) -> Result<&'a str, Utf8Error> {
        core::str::from_utf8(self.title)
    }
    pub const fn base_bytes(&self) -> &'a [u8; LEVEL_RECORD_SIZE] {
        self.base.as_bytes()
    }
    pub fn objects(&self) -> impl FusedIterator<Item = ObjectPlacement> + '_ {
        self.base.objects()
    }
    pub fn terrains(&self) -> impl FusedIterator<Item = TerrainPlacement> + '_ {
        self.base.terrains()
    }
    pub fn steel(&self) -> impl FusedIterator<Item = SteelArea> + '_ {
        self.base.steel()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogError {
    FileIndex {
        found: u8,
    },
    SectionCount {
        file: u8,
        actual: usize,
    },
    SectionSize {
        file: u8,
        part: usize,
        actual: u16,
    },
    WrongFile {
        expected: u8,
        actual: u8,
    },
    MissingOdd {
        index: usize,
    },
    Odd {
        index: usize,
        error: OddError,
    },
    Dat {
        file: u8,
        part: usize,
        error: DatError,
    },
    Decompress {
        file: u8,
        part: u8,
        error: DecompressError,
    },
    Level {
        file: u8,
        part: u8,
        error: LevelError,
    },
}
impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileIndex { found } => write!(f, "DOS level file index {found} is outside 0..=9"),
            Self::SectionCount { file, actual } => {
                write!(f, "LEVEL{file:03}.DAT has {actual} sections, expected 8")
            }
            Self::SectionSize { file, part, actual } => write!(
                f,
                "LEVEL{file:03}.DAT part {part} declares {actual} output bytes, expected {LEVEL_RECORD_SIZE}"
            ),
            Self::WrongFile { expected, actual } => write!(
                f,
                "catalog needs LEVEL{expected:03}.DAT, got file index {actual}"
            ),
            Self::MissingOdd { index } => write!(
                f,
                "catalog needs ODD record {index}, but no table was supplied"
            ),
            Self::Odd { index, error } => write!(f, "catalog ODD record {index}: {error}"),
            Self::Dat { file, part, error } => write!(f, "LEVEL{file:03}.DAT part {part}: {error}"),
            Self::Decompress { file, part, error } => {
                write!(f, "LEVEL{file:03}.DAT part {part}: {error}")
            }
            Self::Level { file, part, error } => {
                write!(f, "LEVEL{file:03}.DAT part {part}: {error}")
            }
        }
    }
}
impl core::error::Error for CatalogError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Odd { error, .. } => Some(error),
            Self::Dat { error, .. } => Some(error),
            Self::Decompress { error, .. } => Some(error),
            Self::Level { error, .. } => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use lemmings_core::Difficulty;
    use std::{format, string::String, vec, vec::Vec};

    const GROUPS: [Difficulty; 4] = [
        Difficulty::Fun,
        Difficulty::Tricky,
        Difficulty::Taxing,
        Difficulty::Mayhem,
    ];

    fn entry(group: Difficulty, number: u8) -> CatalogEntry {
        CatalogEntry::for_level(LevelId::new(group, number).unwrap())
    }

    fn base(part: u8) -> [u8; 2048] {
        let mut b = [0; 2048];
        for (i, w) in [40u16, 50, 25, 7, 1, 2, 3, 4, 5, 6, 7, 8, 624, 4, 3, 0xffff]
            .iter()
            .enumerate()
        {
            b[2 * i..2 * i + 2].copy_from_slice(&w.to_be_bytes());
        }
        b[288..1888].fill(0xff);
        b[2016..].fill(b' ');
        b[2016..2020].copy_from_slice(b"BASE");
        b[2020] = b'0' + part;
        b[39] = 0x0f;
        b[288..292].fill(0);
        b[1888..1892].copy_from_slice(&[0x12, 0x83, 0x25, 0]);
        b
    }

    fn packed(data: &[u8]) -> Vec<u8> {
        let mut bits = String::new();
        for chunk in data.rchunks(264) {
            bits += &if chunk.len() <= 8 {
                format!("00{:03b}", chunk.len() - 1)
            } else {
                format!("111{:08b}", chunk.len() - 9)
            };
            for byte in chunk.iter().rev() {
                bits += &format!("{byte:08b}");
            }
        }
        let payload: Vec<u8> = bits
            .as_bytes()
            .rchunks(8)
            .map(|chunk| {
                chunk
                    .iter()
                    .enumerate()
                    .fold(0, |value, (i, b)| value | ((b - b'0') << i))
            })
            .collect();
        let checksum = payload.iter().fold(0, |a, b| a ^ b);
        let mut bytes = vec![((bits.len() - 1) % 8 + 1) as u8, checksum, 0, 0];
        bytes.extend((data.len() as u16).to_be_bytes());
        bytes.extend([0, 0]);
        bytes.extend(((payload.len() + 10) as u16).to_be_bytes());
        bytes.extend(payload);
        bytes
    }

    fn dat(count: u8) -> Vec<u8> {
        (0..count).flat_map(|part| packed(&base(part))).collect()
    }
    fn odd(count: usize) -> Vec<u8> {
        let mut bytes = vec![0; count * 56];
        for (index, b) in bytes.chunks_exact_mut(56).enumerate() {
            for (i, w) in [88u16, 80, 76, 5, 20, 20, 20, 20, 20, 20, 20, 20]
                .iter()
                .enumerate()
            {
                b[2 * i..2 * i + 2].copy_from_slice(&w.to_be_bytes());
            }
            b[24..].fill(b' ');
            b[24..27].copy_from_slice(b"ODD");
            b[27] = index as u8;
        }
        bytes
    }

    #[test]
    fn all_120_routes_match_the_independent_signed_decimal_config() {
        // Pinned TypeScript config, distinct encoding from Lemmix's production table.
        const CONFIG: [[i16; 30]; 4] = [
            [
                91, 95, 96, 92, 93, 94, 97, -6, -12, -32, -42, -7, 16, -17, -22, -24, -27, -43,
                -51, -63, -84, 13, -41, -57, -60, -71, -46, -61, -65, -82,
            ],
            [
                0, -16, -21, -30, -31, -33, -34, -47, -62, -73, -77, -80, -83, 2, -91, -93, -94,
                -95, -97, 3, 5, 6, 7, 10, 11, 12, 14, 15, 20, 17,
            ],
            [
                22, 23, 24, 25, 26, 27, 30, 31, 32, 33, 34, 35, 36, 37, 1, 40, 41, 42, 43, 44, 45,
                46, 47, 50, 51, 52, 53, 54, 21, 67,
            ],
            [
                55, 56, 57, 60, 61, 62, 63, 64, 65, 66, -67, 70, 71, 72, 73, 74, 75, 76, 77, -92,
                80, 4, 81, 82, 83, 84, 85, 86, 87, 90,
            ],
        ];
        let mut odd_count = 0;
        for (group_index, group) in GROUPS.into_iter().enumerate() {
            for number in 1..=30 {
                let route = entry(group, number);
                let expected = CONFIG[group_index][number as usize - 1];
                let file = (expected.unsigned_abs() / 10) as u8;
                let part = (expected.unsigned_abs() % 10) as u8;
                assert_eq!((route.file_index(), route.part_index()), (file, part));
                assert_eq!(
                    route.level_id().global_index(),
                    group_index * 30 + number as usize - 1
                );
                assert_eq!(
                    route.odd_index(),
                    if expected < 0 {
                        odd_count += 1;
                        Some(file as usize * 8 + part as usize)
                    } else {
                        None
                    }
                );
            }
        }
        assert_eq!(odd_count, 40);
    }

    #[test]
    fn fun_eight_uses_base_index_six_not_display_index_seven() {
        let route = entry(Difficulty::Fun, 8);
        assert_eq!(
            (route.file_index(), route.part_index(), route.odd_index()),
            (0, 6, Some(6))
        );
        assert_eq!(route.level_id().global_index(), 7);
    }

    #[test]
    fn file_indexes_and_every_container_section_are_checked() {
        for found in [10, 255] {
            assert!(
                matches!(DosLevelFile::parse(found,&[]),Err(CatalogError::FileIndex { found: n }) if n==found)
            );
        }
        for count in [0, 1, 7, 9] {
            assert!(
                matches!(DosLevelFile::parse(0,&dat(count)),Err(CatalogError::SectionCount { file:0,actual }) if actual==count as usize)
            );
        }
        let bytes = dat(8);
        let file = DosLevelFile::parse(9, &bytes).unwrap();
        assert_eq!(file.index(), 9);
    }

    #[test]
    fn trailing_corruption_and_wrong_decoded_section_size_reject_the_file() {
        let mut bytes = dat(8);
        bytes.push(0);
        assert!(matches!(
            DosLevelFile::parse(0, &bytes),
            Err(CatalogError::Dat {
                file: 0,
                part: 8,
                error: DatError::TruncatedHeader { .. }
            })
        ));
        let mut bytes = dat(8);
        bytes[4..6].copy_from_slice(&2047u16.to_be_bytes());
        assert!(matches!(
            DosLevelFile::parse(0, &bytes),
            Err(CatalogError::SectionSize {
                file: 0,
                part: 0,
                actual: 2047
            })
        ));
    }

    #[test]
    fn unmodified_levels_need_no_odd_table_and_keep_base_properties() {
        let bytes = dat(8);
        let file = DosLevelFile::parse(9, &bytes).unwrap();
        let mut out = [0xcc; 2048];
        let selected = entry(Difficulty::Fun, 1)
            .load(&file, None, &mut out)
            .unwrap();
        assert_eq!(
            selected.header(),
            LevelRecord::parse(&base(1)).unwrap().header()
        );
        assert_eq!(&selected.title_bytes()[..5], b"BASE1");
        assert_eq!(selected.base_bytes(), &base(1));
        assert_eq!(selected.entry(), entry(Difficulty::Fun, 1));
    }

    #[test]
    fn variants_replace_only_initial_properties_and_title() {
        let bytes = dat(8);
        let file = DosLevelFile::parse(0, &bytes).unwrap();
        let odd_bytes = odd(80);
        let table = OddTable::parse(&odd_bytes).unwrap();
        let mut out = [0xcc; 2048];
        let selected = entry(Difficulty::Fun, 8)
            .load(&file, Some(table), &mut out)
            .unwrap();
        assert_eq!(
            selected.header().properties(),
            table.get(6).unwrap().properties()
        );
        assert_eq!(
            selected.title_bytes().as_ptr(),
            odd_bytes[6 * 56 + 24..].as_ptr()
        );
        assert_eq!(selected.title_bytes()[3], 6);
        assert_eq!(
            (
                selected.header().screen_x,
                selected.header().graphics,
                selected.header().special,
                selected.header().superlemming
            ),
            (624, 4, 3, true)
        );
        assert_eq!(selected.base_bytes(), &base(6));
        let base_bytes = base(6);
        let raw = LevelRecord::parse(&base_bytes).unwrap();
        assert_eq!(
            selected.objects().collect::<Vec<_>>(),
            raw.objects().collect::<Vec<_>>()
        );
        assert_eq!(
            selected.terrains().collect::<Vec<_>>(),
            raw.terrains().collect::<Vec<_>>()
        );
        assert_eq!(
            selected.steel().collect::<Vec<_>>(),
            raw.steel().collect::<Vec<_>>()
        );
        assert!(selected.title().is_ok());
    }

    #[test]
    fn wrong_files_and_missing_odd_data_fail_before_writing_output() {
        let bytes = dat(8);
        let file = DosLevelFile::parse(0, &bytes).unwrap();
        let mut out = [0xcc; 2048];
        assert_eq!(
            entry(Difficulty::Fun, 1).load(&file, None, &mut out),
            Err(CatalogError::WrongFile {
                expected: 9,
                actual: 0
            })
        );
        assert_eq!(out, [0xcc; 2048]);
        assert_eq!(
            entry(Difficulty::Fun, 8).load(&file, None, &mut out),
            Err(CatalogError::MissingOdd { index: 6 })
        );
        assert_eq!(out, [0xcc; 2048]);
        let odd_bytes = odd(6);
        let table = OddTable::parse(&odd_bytes).unwrap();
        assert_eq!(
            entry(Difficulty::Fun, 8).load(&file, Some(table), &mut out),
            Err(CatalogError::Odd {
                index: 6,
                error: OddError::Index { index: 6, count: 6 }
            })
        );
        assert_eq!(out, [0xcc; 2048]);
    }

    #[test]
    fn selected_decompression_and_base_level_errors_have_file_and_part_context() {
        let mut bytes = vec![0, 0, 0, 0, 8, 0, 0, 0, 0, 11, 0];
        bytes.extend(dat(7));
        let file = DosLevelFile::parse(0, &bytes).unwrap();
        let mut out = [0xcc; 2048];
        assert!(matches!(
            entry(Difficulty::Tricky, 1).load(&file, None, &mut out),
            Err(CatalogError::Decompress {
                file: 0,
                part: 0,
                error: DecompressError::UnexpectedEnd { .. }
            })
        ));
        let mut invalid = base(0);
        invalid[0..2].fill(0);
        let mut bytes = packed(&invalid);
        bytes.extend(dat(7));
        let file = DosLevelFile::parse(0, &bytes).unwrap();
        assert!(matches!(
            entry(Difficulty::Tricky, 1).load(&file, None, &mut out),
            Err(CatalogError::Level {
                file: 0,
                part: 0,
                error: LevelError::InvalidValue { .. }
            })
        ));
    }
}
