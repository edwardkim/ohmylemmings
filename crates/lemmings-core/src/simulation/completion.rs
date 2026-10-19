//! DOS result percentages and post-actor completion checks. See #52's contract.
use core::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndReason {
    PopulationExhausted,
    TimeExpired,
}

/// A frozen result. Target uses original total; done uses the current Nuke total.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LevelResult {
    pub reason: EndReason,
    pub lemming_count: u32,
    pub required: u32,
    pub rescued: u32,
    pub target_percent: u32,
    pub rescued_percent: u32,
    pub success: bool,
}

/// Counts after actor/particle updates, with time from before this tick. DOS
/// checks the displayed clock before decrementing it; a zero clock still allows
/// that boundary's actor update. Paused calls can check without advancing time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompletionProgress {
    pub seconds_before_tick: u32,
    pub released: u32,
    pub removed: u32,
    pub rescued: u32,
    pub max_lemmings: u32,
    pub nuked: bool,
    /// Maximum remaining particle timer across stable actor records (0..=52).
    pub particle_timer: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionError {
    RequiredExceedsTotal,
    InvalidProgress,
}
impl fmt::Display for CompletionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::RequiredExceedsTotal => "required rescues exceed original population",
            Self::InvalidProgress => "invalid completion counter subsets or particle timer",
        })
    }
}
impl core::error::Error for CompletionError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompletionRules {
    total: u32,
    required: u32,
}
impl CompletionRules {
    pub const fn new(total: u32, required: u32) -> Result<Self, CompletionError> {
        if required > total {
            Err(CompletionError::RequiredExceedsTotal)
        } else {
            Ok(Self { total, required })
        }
    }
    pub const fn target_percent(self) -> u32 {
        percentage(self.required, self.total) as u32
    }
    pub const fn total(self) -> u32 {
        self.total
    }
    pub const fn required(self) -> u32 {
        self.required
    }
    /// DOS checks exhausted population (including explosion display lifetime),
    /// then time. Meeting the quota alone never finishes a running level.
    /// This pure operation allocates nothing; callers retain the first result.
    pub fn evaluate(self, p: CompletionProgress) -> Result<Option<LevelResult>, CompletionError> {
        if p.released > self.total
            || p.removed > p.released
            || p.rescued > p.removed
            || p.max_lemmings > self.total
            || p.max_lemmings < p.released
            || (!p.nuked && p.max_lemmings != self.total)
            || p.particle_timer > 52
        {
            return Err(CompletionError::InvalidProgress);
        }
        let exhausted = p.particle_timer == 0
            && (p.removed >= p.max_lemmings || (p.nuked && p.removed == p.released));
        let reason = if exhausted {
            EndReason::PopulationExhausted
        } else if p.seconds_before_tick == 0 {
            EndReason::TimeExpired
        } else {
            return Ok(None);
        };
        let rescued_percent = percentage(p.rescued, p.max_lemmings) as u32;
        Ok(Some(LevelResult {
            reason,
            lemming_count: p.max_lemmings,
            required: self.required,
            rescued: p.rescued,
            target_percent: self.target_percent(),
            rescued_percent,
            success: rescued_percent >= self.target_percent(),
        }))
    }
}

/// Floor after wide multiplication, including ratios above 100 for general callers.
pub const fn percentage(count: u32, total: u32) -> u64 {
    if total == 0 {
        0
    } else {
        count as u64 * 100 / total as u64
    }
}
