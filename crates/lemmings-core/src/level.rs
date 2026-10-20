use core::fmt;

/// The four difficulty groups in the original DOS level catalog, in display order.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Difficulty {
    Fun,
    Tricky,
    Taxing,
    Mayhem,
}

impl Difficulty {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Fun => "fun",
            Self::Tricky => "tricky",
            Self::Taxing => "taxing",
            Self::Mayhem => "mayhem",
        }
    }
    /// Returns the zero-based group index used by the compatibility catalog.
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidDifficulty;
impl fmt::Display for InvalidDifficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("difficulty must be fun, tricky, taxing or mayhem")
    }
}
impl core::error::Error for InvalidDifficulty {}
impl core::str::FromStr for Difficulty {
    type Err = InvalidDifficulty;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        [Self::Fun, Self::Tricky, Self::Taxing, Self::Mayhem]
            .into_iter()
            .find(|d| value.eq_ignore_ascii_case(d.name()))
            .ok_or(InvalidDifficulty)
    }
}
impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A level in the original DOS catalog, with a one-based displayed number.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LevelId {
    difficulty: Difficulty,
    number: u8,
}

impl LevelId {
    /// Creates an identifier with a displayed number in `1..=30`.
    pub const fn new(difficulty: Difficulty, number: u8) -> Result<Self, InvalidLevelNumber> {
        match number {
            1..=30 => Ok(Self { difficulty, number }),
            _ => Err(InvalidLevelNumber { number }),
        }
    }

    pub const fn difficulty(self) -> Difficulty {
        self.difficulty
    }

    pub const fn number(self) -> u8 {
        self.number
    }

    /// Returns the zero-based index within the difficulty group.
    pub const fn index(self) -> usize {
        self.number as usize - 1
    }

    /// Returns the zero-based displayed catalog index in `0..120`.
    ///
    /// This index is not a DAT section number or an ODDTABLE record index.
    pub const fn global_index(self) -> usize {
        self.difficulty.index() * 30 + self.index()
    }

    /// Catalog successor, including rank boundaries; Mayhem 30 is terminal.
    /// Pinned Lemmix Styles.Base.TSection.DoAddLevelInfo links adjacent sections.
    pub const fn next(self) -> Option<Self> {
        if self.number < 30 {
            Some(Self {
                difficulty: self.difficulty,
                number: self.number + 1,
            })
        } else {
            let difficulty = match self.difficulty {
                Difficulty::Fun => Difficulty::Tricky,
                Difficulty::Tricky => Difficulty::Taxing,
                Difficulty::Taxing => Difficulty::Mayhem,
                Difficulty::Mayhem => return None,
            };
            Some(Self {
                difficulty,
                number: 1,
            })
        }
    }
}

/// A displayed level number outside the supported `1..=30` range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidLevelNumber {
    pub number: u8,
}

impl fmt::Display for InvalidLevelNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "level number {} is outside 1..=30", self.number)
    }
}

impl core::error::Error for InvalidLevelNumber {}

#[cfg(test)]
mod tests {
    use super::{Difficulty, InvalidLevelNumber, LevelId};

    const GROUPS: [Difficulty; 4] = [
        Difficulty::Fun,
        Difficulty::Tricky,
        Difficulty::Taxing,
        Difficulty::Mayhem,
    ];

    #[test]
    fn rejects_every_number_outside_the_original_catalog() {
        for difficulty in GROUPS {
            for number in 0..=u8::MAX {
                let result = LevelId::new(difficulty, number);
                if (1..=30).contains(&number) {
                    assert!(result.is_ok());
                } else {
                    assert_eq!(result, Err(InvalidLevelNumber { number }));
                }
            }
        }
    }

    #[test]
    fn maps_all_120_displayed_levels_to_distinct_zero_based_indices() {
        let mut expected_global_index = 0;
        for (group_index, difficulty) in GROUPS.into_iter().enumerate() {
            for (expected_index, number) in (1..=30).enumerate() {
                let level = LevelId::new(difficulty, number).unwrap();
                assert_eq!(level.difficulty(), difficulty);
                assert_eq!(level.number(), number);
                assert_eq!(difficulty.index(), group_index);
                assert_eq!(level.index(), expected_index);
                assert_eq!(level.global_index(), expected_global_index);
                expected_global_index += 1;
            }
        }
        assert_eq!(expected_global_index, 120);
    }

    #[test]
    fn named_difficulties_and_catalog_successors_keep_rank_identity() {
        for (i, name) in ["fun", "tricky", "taxing", "mayhem"]
            .into_iter()
            .enumerate()
        {
            let difficulty: Difficulty = name.parse().unwrap();
            assert_eq!(difficulty.index(), i);
            assert_eq!(difficulty.name(), name);
            let first = LevelId::new(difficulty, 1).unwrap();
            assert_eq!(first.next().unwrap().number(), 2);
            let last = LevelId::new(difficulty, 30).unwrap();
            assert_eq!(
                last.next().map(|n| n.global_index()),
                (i < 3).then_some((i + 1) * 30)
            );
        }
        for name in ["", "easy", "4", " fun", "FUN "] {
            assert!(name.parse::<Difficulty>().is_err(), "{name}");
        }
        assert_eq!("Tricky".parse::<Difficulty>().unwrap(), Difficulty::Tricky);
    }
}
