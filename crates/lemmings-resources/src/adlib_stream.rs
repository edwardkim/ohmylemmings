//! Bounded port of the DOS-original ADLIB driver, not an OPL synthesizer.
//! Instruction offsets and intentional TS differences: issue-74_adlib_stream.md.
use crate::{AdlibError, AdlibImage};
use core::fmt;

/// Hard ceiling per driver call, also when a caller supplies a larger budget.
pub const ADLIB_COMMAND_BUDGET: usize = 4096;
const CHANNELS: usize = 9;
const BASE: usize = 2926;
const OPS: [(u8, u8); CHANNELS] = [
    (0, 3),
    (1, 4),
    (2, 5),
    (8, 11),
    (9, 12),
    (10, 13),
    (16, 19),
    (17, 20),
    (18, 21),
];

/// One ordered OPL2 write. `audio_step` counts driver calls, not game ticks or samples.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AdlibWrite {
    pub audio_step: u64,
    pub register: u8,
    pub value: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdlibStreamError {
    Resource(AdlibError),
    Budget,
    Capacity { required: usize },
    Address { offset: usize, count: usize },
    Opcode(u8),
    Instrument(u8),
    MissingInstrument,
    StepOverflow,
}
impl fmt::Display for AdlibStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid AdLib stream: {self:?}")
    }
}
impl core::error::Error for AdlibStreamError {}
impl From<AdlibError> for AdlibStreamError {
    fn from(e: AdlibError) -> Self {
        Self::Resource(e)
    }
}

/// Raw 20-byte DOS channel records plus all persistent VM state.
/// Channel state byte 16: 0 inactive, 1 music, 2 effect. Addresses are image offsets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdlibState {
    pub channels: [[u8; 20]; CHANNELS],
    pub sample_rate_factor: u16,
    pub instrument_offset: usize,
    pub wait_cycles: u8,
    pub cycle: u8,
    pub music_channels: u8,
    pub pending_music: u8,
    pub playing_music: u8,
    pub deferred_music: u8,
    pub pending_effect: u8,
    pub pending_stop: bool,
    pub audio_step: u64,
    pub registers: [u8; 256],
}
#[derive(Clone, Copy, Debug)]
pub struct AdlibPlayer<'a> {
    image: AdlibImage<'a>,
    state: AdlibState,
}
struct Output<'a> {
    target: Option<&'a mut [AdlibWrite]>,
    count: usize,
    step: u64,
}
impl Output<'_> {
    fn write(&mut self, state: &mut AdlibState, register: u8, value: u8) {
        if let Some(target) = self.target.as_deref_mut() {
            target[self.count] = AdlibWrite {
                audio_step: self.step,
                register,
                value,
            };
        }
        self.count += 1;
        state.registers[usize::from(register)] = value;
    }
}
impl<'a> AdlibPlayer<'a> {
    pub fn new(image: AdlibImage<'a>) -> Self {
        let mut channels = [[0; 20]; CHANNELS];
        for (i, c) in channels.iter_mut().enumerate() {
            c[5] = OPS[i].0;
            c[6] = OPS[i].1;
            c[7] = i as u8;
            c[8] = 0xb0 + i as u8;
        }
        Self {
            image,
            state: AdlibState {
                channels,
                sample_rate_factor: 0x4000,
                instrument_offset: 0,
                wait_cycles: 9,
                cycle: 0,
                music_channels: 1,
                pending_music: 0,
                playing_music: 0,
                deferred_music: 0,
                pending_effect: 0,
                pending_stop: false,
                audio_step: 0,
                registers: [0; 256],
            },
        }
    }
    pub const fn state(&self) -> AdlibState {
        self.state
    }
    /// Start/restart on the next successful step, preserving the DOS saved key-off values.
    /// Corresponds to clearing the playing flag (AH=5), then queuing AH=3, AL=index+1.
    pub fn request_music(&mut self, index: usize) -> Result<(), AdlibStreamError> {
        self.image.music(index)?;
        self.state.playing_music = 0;
        self.state.pending_music = (index + 1) as u8;
        self.state.pending_stop = false;
        Ok(())
    }
    /// Queue an effect (DOS AH=4). A full four-slot pool retains the last request.
    pub fn request_effect(&mut self, index: usize) -> Result<(), AdlibStreamError> {
        self.image.effect_offset(index)?;
        self.state.pending_effect = (index + 1) as u8;
        Ok(())
    }
    /// Queue DOS AH=2 stop/reset before the next step. Also cancel pending requests.
    pub fn request_stop(&mut self) {
        self.state.pending_stop = true;
        self.state.pending_music = 0;
        self.state.deferred_music = 0;
        self.state.pending_effect = 0;
    }
    /// Execute one driver call with a shared command budget for all nine channels.
    /// Errors preserve the complete player and caller's output. The immutable image
    /// permits a validating/counting pass followed by an identical writing pass.
    pub fn step(
        &mut self,
        writes: &mut [AdlibWrite],
        command_budget: usize,
    ) -> Result<usize, AdlibStreamError> {
        let mut next = *self;
        let mut count = Output {
            target: None,
            count: 0,
            step: self.state.audio_step,
        };
        next.run(&mut count, command_budget.min(ADLIB_COMMAND_BUDGET))?;
        if writes.len() < count.count {
            return Err(AdlibStreamError::Capacity {
                required: count.count,
            });
        }
        let mut output = Output {
            target: Some(writes),
            count: 0,
            step: self.state.audio_step,
        };
        let mut replay = *self;
        replay.run(&mut output, command_budget.min(ADLIB_COMMAND_BUDGET))?;
        *self = next;
        Ok(output.count)
    }
    fn run(&mut self, out: &mut Output<'_>, mut budget: usize) -> Result<(), AdlibStreamError> {
        let next_step = self
            .state
            .audio_step
            .checked_add(1)
            .ok_or(AdlibStreamError::StepOverflow)?;
        if self.state.pending_stop {
            self.stop_music(out);
            // DOS 0x6d..0xa3: release operators and key off all nine channels.
            let reset_ops = [
                (0, 11),
                (1, 12),
                (2, 13),
                (3, 16),
                (4, 17),
                (5, 18),
                (8, 19),
                (9, 20),
                (10, 21),
            ];
            for (i, (a, b)) in reset_ops.iter().enumerate() {
                out.write(&mut self.state, 0x80 + a, 15);
                out.write(&mut self.state, 0x80 + b, 15);
                out.write(&mut self.state, 0xb0 + i as u8, 0);
            }
            for c in &mut self.state.channels {
                c[16] = 0;
            }
            self.state.pending_stop = false;
        }
        if self.state.playing_music == 0 {
            self.state.deferred_music = self.state.pending_music;
        }
        if self.state.deferred_music != 0 {
            let m = self
                .image
                .music(usize::from(self.state.deferred_music - 1))?;
            self.state.playing_music = self.state.deferred_music;
            self.state.pending_music = 0;
            self.state.deferred_music = 0;
            for c in &mut self.state.channels {
                c[16] = 0;
            }
            self.state.sample_rate_factor = m.sample_rate_factor;
            self.state.instrument_offset = m.instrument_offset;
            self.state.wait_cycles = m.wait_cycles;
            self.state.music_channels = m.channels().len() as u8;
            for (i, source) in m.channels().iter().enumerate() {
                let c = &mut self.state.channels[i];
                set_word(c, 10, source.program_offset + 2);
                set_word(c, 12, source.first_bar_offset);
                c[1] = 1;
                c[16] = 1;
            }
            for i in 0..CHANNELS {
                self.key_off(i, out);
            }
            for (r, v) in [(1, 0x20), (0xbd, 0xc0), (8, 0), (4, 0x21)] {
                out.write(&mut self.state, r, v);
            }
            self.state.cycle = 1;
        }
        // DX is caller-owned in the DOS ABI. We define DX=0 on entry; envelope
        // writes replace it, volume commands retain it, matching the driver.
        let mut operators = (0, 0);
        if self.state.pending_effect != 0
            && let Some(i) = (5..9).rev().find(|&i| self.state.channels[i][16] == 0)
        {
            let p = self
                .image
                .effect_offset(usize::from(self.state.pending_effect - 1))?;
            self.state.pending_effect = 0;
            self.state.channels[i][16] = 2;
            self.state.channels[i][19] = 0;
            self.bar(i, p, &mut operators, out, &mut budget)?;
        }
        self.state.cycle = self.state.cycle.wrapping_sub(1);
        for i in 0..CHANNELS {
            if self.state.channels[i][16] == 0
                || (self.state.channels[i][16] == 1 && self.state.cycle != 0)
            {
                continue;
            }
            self.state.channels[i][1] = self.state.channels[i][1].wrapping_sub(1);
            let p = word(&self.state.channels[i], 12);
            if self.state.channels[i][1] == 0 {
                self.bar(i, p, &mut operators, out, &mut budget)?;
            } else {
                let slide = self.state.channels[i][19];
                if slide != 0 {
                    self.state.channels[i][0] = self.state.channels[i][0].wrapping_add(slide);
                    self.frequency(i, out)?;
                }
                if self.read_stream(i, p, 1)?[0] != 0x82 {
                    let inst = if word(&self.state.channels[i], 2) == 0 {
                        0
                    } else {
                        self.instrument(i)?
                    };
                    if self.image.as_bytes()[inst + 14] == self.state.channels[i][1] {
                        self.key_off(i, out);
                        self.state.channels[i][19] = 0;
                    }
                }
            }
        }
        if self.state.cycle == 0 {
            self.state.cycle = self.state.wait_cycles;
        }
        self.state.audio_step = next_step;
        Ok(())
    }
    fn stop_music(&mut self, out: &mut Output<'_>) {
        for i in 0..usize::from(self.state.music_channels) {
            self.key_off(i, out);
            self.state.channels[i][16] = 0;
        }
        self.state.pending_music = 0;
        self.state.playing_music = 0;
    }
    fn key_off(&mut self, i: usize, out: &mut Output<'_>) {
        let c = self.state.channels[i];
        out.write(&mut self.state, c[8], c[9]);
    }
    fn read_stream(&self, i: usize, p: usize, n: usize) -> Result<&'a [u8], AdlibStreamError> {
        let (start, end) = if self.state.channels[i][16] == 2 {
            (22025, 22125)
        } else {
            (2968, 21731)
        };
        self.read(p, n, start, end)
    }
    fn read(
        &self,
        p: usize,
        n: usize,
        start: usize,
        end: usize,
    ) -> Result<&'a [u8], AdlibStreamError> {
        if p < start || p.checked_add(n).is_none_or(|v| v > end) {
            return Err(AdlibStreamError::Address {
                offset: p,
                count: n,
            });
        }
        Ok(&self.image.as_bytes()[p..p + n])
    }
    fn instrument(&self, i: usize) -> Result<usize, AdlibStreamError> {
        let p = word(&self.state.channels[i], 2);
        if p == 0 {
            return Err(AdlibStreamError::MissingInstrument);
        }
        let (start, end) = if i >= 4 {
            (21731, 21989)
        } else {
            (2968, 21731)
        };
        self.read(p, 16, start, end)?;
        Ok(p)
    }
    fn bar(
        &mut self,
        i: usize,
        mut p: usize,
        ops: &mut (u8, u8),
        out: &mut Output<'_>,
        budget: &mut usize,
    ) -> Result<(), AdlibStreamError> {
        loop {
            if *budget == 0 {
                return Err(AdlibStreamError::Budget);
            }
            *budget -= 1;
            let cmd = self.read_stream(i, p, 1)?[0];
            p += 1;
            match cmd {
                0..=0x7f => {
                    self.state.channels[i][0] = cmd;
                    self.key_off(i, out);
                    self.frequency(i, out)?;
                    self.suspend(i, p);
                    return Ok(());
                }
                0xe0..=0xff => self.state.channels[i][17] = cmd - 0xdf,
                0xc0..=0xdf => {
                    let id = cmd - 0xc0;
                    if id == 0 {
                        return Err(AdlibStreamError::Instrument(id));
                    }
                    self.state.channels[i][4] = id;
                    let base = if i >= 4 {
                        21731
                    } else {
                        self.state.instrument_offset
                    };
                    let inst = base + usize::from(id - 1) * 16;
                    set_word(&mut self.state.channels[i], 2, inst);
                    self.instrument(i)?;
                    let data = &self.image.as_bytes()[inst..inst + 16];
                    *ops = (self.state.channels[i][5], self.state.channels[i][6]);
                    for (r, v) in [
                        (ops.0 + 0x60, data[0]),
                        (ops.1 + 0x60, data[1]),
                        (ops.0 + 0x80, data[2]),
                        (ops.1 + 0x80, data[3]),
                        (ops.0 + 0xe0, data[6]),
                        (ops.1 + 0xe0, data[7]),
                        (self.state.channels[i][7] + 0xc0, data[9]),
                        (ops.0 + 0x20, data[4]),
                        (ops.1 + 0x20, data[5]),
                    ] {
                        out.write(&mut self.state, r, v);
                    }
                    self.state.channels[i][18] = data[8];
                    self.state.channels[i][15] = data[11];
                    self.level(i, data[10], *ops, out)?;
                }
                0xb0..=0xbf => {
                    let level = self.read_stream(i, p, 1)?[0];
                    p += 1;
                    self.level(i, level, *ops, out)?;
                }
                0x80 => {
                    let pp = word(&self.state.channels[i], 10);
                    let bytes = self.read(pp, 2, 2968, 21731)?;
                    let bar = word(bytes, 0);
                    let (pp, bar) = if bar == 0 {
                        let lp = BASE + word(self.read(pp + 2, 2, 2968, 21731)?, 0);
                        let bar = BASE + word(self.read(lp, 2, 2968, 21731)?, 0);
                        (lp + 2, bar)
                    } else {
                        (pp + 2, BASE + bar)
                    };
                    self.read_stream(i, bar, 1)?;
                    set_word(&mut self.state.channels[i], 10, pp);
                    set_word(&mut self.state.channels[i], 12, bar);
                    p = bar;
                }
                0x81 => {
                    self.key_off(i, out);
                    self.state.channels[i][19] = 0;
                    self.suspend(i, p);
                    return Ok(());
                }
                0x82 => {
                    self.suspend(i, p);
                    return Ok(());
                }
                0x83 => {
                    self.stop_music(out);
                    return Ok(());
                }
                0x84 => {
                    self.state.channels[i][18] = self.read_stream(i, p, 1)?[0];
                    p += 1;
                }
                0x85 => {
                    self.key_off(i, out);
                    self.state.channels[i][16] = 0;
                    return Ok(());
                }
                0x86 => self.state.channels[i][19] = 1,
                0x87 => self.state.channels[i][19] = 255,
                _ => return Err(AdlibStreamError::Opcode(cmd)),
            }
        }
    }
    fn suspend(&mut self, i: usize, p: usize) {
        set_word(&mut self.state.channels[i], 12, p);
        self.state.channels[i][1] = self.state.channels[i][17];
    }
    fn level(
        &mut self,
        i: usize,
        value: u8,
        ops: (u8, u8),
        out: &mut Output<'_>,
    ) -> Result<(), AdlibStreamError> {
        let inst = self.instrument(i)?;
        let data = self.image.as_bytes();
        self.state.channels[i][14] = value;
        let a = data[2215 + usize::from(value & 127)] | ((data[inst + 12] << 2) & 0xc0);
        let v = self.state.channels[i][15].wrapping_add(data[inst + 10]) & 127;
        let b = data[2215 + usize::from(v)] | (data[inst + 12].rotate_right(2) & 0xc0);
        out.write(&mut self.state, ops.0 + 0x40, a);
        out.write(&mut self.state, ops.1 + 0x40, b);
        Ok(())
    }
    fn frequency(&mut self, i: usize, out: &mut Output<'_>) -> Result<(), AdlibStreamError> {
        let c = self.state.channels[i];
        let index = c[0].wrapping_add(c[18]).wrapping_add(4);
        let data = self.image.as_bytes();
        let mut octave = data[2727 + usize::from(index)].wrapping_sub(1);
        // The original uses four AL shifts then doubles AX: only the low
        // nibble survives. Effect 14 deliberately reads beyond nominal tables.
        let table = data[2823 + usize::from(index)] & 15;
        let mut frequency = word(data, 2343 + usize::from(table) * 32) as u16;
        if frequency & 0x8000 != 0 {
            octave = octave.wrapping_add(1);
        }
        if octave & 0x80 != 0 {
            octave = octave.wrapping_add(1);
            frequency = ((frequency as i16) >> 1) as u16;
        }
        out.write(&mut self.state, c[7] + 0xa0, frequency as u8);
        let high = ((frequency >> 8) as u8 & 3) | octave.wrapping_mul(4);
        self.state.channels[i][8] = c[7] + 0xb0;
        self.state.channels[i][9] = high;
        out.write(&mut self.state, c[7] + 0xb0, high | 0x20);
        Ok(())
    }
}
fn word(b: &[u8], p: usize) -> usize {
    usize::from(u16::from_le_bytes([b[p], b[p + 1]]))
}
fn set_word(b: &mut [u8], p: usize, v: usize) {
    b[p..p + 2].copy_from_slice(&(v as u16).to_le_bytes());
}
