// Modified for ohmylemmings on 2026-10-20: freeze stereo-ext off,
// remove unreachable floating-point panning; preserve the integer OPL core.
// Pure-Rust port of Nuked-OPL3 1.8.
//
// Copyright (C) 2013-2020 Nuke.YKT
// Copyright (C) 2026 Tony Gies (Rust port)
//
// This file is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as
// published by the Free Software Foundation, either version 2.1
// of the License, or (at your option) any later version.
//
// This file is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this file. If not, see <https://www.gnu.org/licenses/>.
//
// The original C implementation stores self-referential pointers between the
// chip, channels, slots, modulation inputs, and channel outputs. This module
// keeps the same state machine and sample ordering, but represents those links
// as stable indices so the chip can be moved safely in Rust.

const WRITEBUF_SIZE: usize = 1024;
const WRITEBUF_DELAY: u64 = 2;
const RSM_FRAC: u32 = 10;
const NATIVE_RATE: u32 = 49_716;
const CHANNEL_SAMPLE_DELAY: bool = !false;

const CH_2OP: u8 = 0;
const CH_4OP: u8 = 1;
const CH_4OP2: u8 = 2;
const CH_DRUM: u8 = 3;

const EGK_NORM: u8 = 0x01;
const EGK_DRUM: u8 = 0x02;

const ENV_ATTACK: u8 = 0;
const ENV_DECAY: u8 = 1;
const ENV_SUSTAIN: u8 = 2;
const ENV_RELEASE: u8 = 3;

const LOGSINROM: [u16; 256] = [
    0x859, 0x6c3, 0x607, 0x58b, 0x52e, 0x4e4, 0x4a6, 0x471, 0x443, 0x41a, 0x3f5, 0x3d3, 0x3b5,
    0x398, 0x37e, 0x365, 0x34e, 0x339, 0x324, 0x311, 0x2ff, 0x2ed, 0x2dc, 0x2cd, 0x2bd, 0x2af,
    0x2a0, 0x293, 0x286, 0x279, 0x26d, 0x261, 0x256, 0x24b, 0x240, 0x236, 0x22c, 0x222, 0x218,
    0x20f, 0x206, 0x1fd, 0x1f5, 0x1ec, 0x1e4, 0x1dc, 0x1d4, 0x1cd, 0x1c5, 0x1be, 0x1b7, 0x1b0,
    0x1a9, 0x1a2, 0x19b, 0x195, 0x18f, 0x188, 0x182, 0x17c, 0x177, 0x171, 0x16b, 0x166, 0x160,
    0x15b, 0x155, 0x150, 0x14b, 0x146, 0x141, 0x13c, 0x137, 0x133, 0x12e, 0x129, 0x125, 0x121,
    0x11c, 0x118, 0x114, 0x10f, 0x10b, 0x107, 0x103, 0x0ff, 0x0fb, 0x0f8, 0x0f4, 0x0f0, 0x0ec,
    0x0e9, 0x0e5, 0x0e2, 0x0de, 0x0db, 0x0d7, 0x0d4, 0x0d1, 0x0cd, 0x0ca, 0x0c7, 0x0c4, 0x0c1,
    0x0be, 0x0bb, 0x0b8, 0x0b5, 0x0b2, 0x0af, 0x0ac, 0x0a9, 0x0a7, 0x0a4, 0x0a1, 0x09f, 0x09c,
    0x099, 0x097, 0x094, 0x092, 0x08f, 0x08d, 0x08a, 0x088, 0x086, 0x083, 0x081, 0x07f, 0x07d,
    0x07a, 0x078, 0x076, 0x074, 0x072, 0x070, 0x06e, 0x06c, 0x06a, 0x068, 0x066, 0x064, 0x062,
    0x060, 0x05e, 0x05c, 0x05b, 0x059, 0x057, 0x055, 0x053, 0x052, 0x050, 0x04e, 0x04d, 0x04b,
    0x04a, 0x048, 0x046, 0x045, 0x043, 0x042, 0x040, 0x03f, 0x03e, 0x03c, 0x03b, 0x039, 0x038,
    0x037, 0x035, 0x034, 0x033, 0x031, 0x030, 0x02f, 0x02e, 0x02d, 0x02b, 0x02a, 0x029, 0x028,
    0x027, 0x026, 0x025, 0x024, 0x023, 0x022, 0x021, 0x020, 0x01f, 0x01e, 0x01d, 0x01c, 0x01b,
    0x01a, 0x019, 0x018, 0x017, 0x017, 0x016, 0x015, 0x014, 0x014, 0x013, 0x012, 0x011, 0x011,
    0x010, 0x00f, 0x00f, 0x00e, 0x00d, 0x00d, 0x00c, 0x00c, 0x00b, 0x00a, 0x00a, 0x009, 0x009,
    0x008, 0x008, 0x007, 0x007, 0x007, 0x006, 0x006, 0x005, 0x005, 0x005, 0x004, 0x004, 0x004,
    0x003, 0x003, 0x003, 0x002, 0x002, 0x002, 0x002, 0x001, 0x001, 0x001, 0x001, 0x001, 0x001,
    0x001, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000, 0x000,
];

// Pre-shifted by 1: stores `original_exprom[i] << 1` so the runtime `<< 1` in
// envelope_calc_exp can be dropped. Max value 0x7fa << 1 = 0xff4, still fits in u16.
const EXPROM: [u16; 256] = [
    0xff4, 0xfea, 0xfde, 0xfd4, 0xfc8, 0xfbe, 0xfb4, 0xfa8, 0xf9e, 0xf92, 0xf88, 0xf7e, 0xf72,
    0xf68, 0xf5c, 0xf52, 0xf48, 0xf3e, 0xf32, 0xf28, 0xf1e, 0xf14, 0xf08, 0xefe, 0xef4, 0xeea,
    0xee0, 0xed4, 0xeca, 0xec0, 0xeb6, 0xeac, 0xea2, 0xe98, 0xe8e, 0xe84, 0xe7a, 0xe70, 0xe66,
    0xe5c, 0xe52, 0xe48, 0xe3e, 0xe34, 0xe2a, 0xe20, 0xe16, 0xe0c, 0xe04, 0xdfa, 0xdf0, 0xde6,
    0xddc, 0xdd2, 0xdca, 0xdc0, 0xdb6, 0xdac, 0xda4, 0xd9a, 0xd90, 0xd88, 0xd7e, 0xd74, 0xd6a,
    0xd62, 0xd58, 0xd50, 0xd46, 0xd3c, 0xd34, 0xd2a, 0xd22, 0xd18, 0xd10, 0xd06, 0xcfe, 0xcf4,
    0xcec, 0xce2, 0xcda, 0xcd0, 0xcc8, 0xcbe, 0xcb6, 0xcae, 0xca4, 0xc9c, 0xc92, 0xc8a, 0xc82,
    0xc78, 0xc70, 0xc68, 0xc60, 0xc56, 0xc4e, 0xc46, 0xc3c, 0xc34, 0xc2c, 0xc24, 0xc1c, 0xc12,
    0xc0a, 0xc02, 0xbfa, 0xbf2, 0xbea, 0xbe0, 0xbd8, 0xbd0, 0xbc8, 0xbc0, 0xbb8, 0xbb0, 0xba8,
    0xba0, 0xb98, 0xb90, 0xb88, 0xb80, 0xb78, 0xb70, 0xb68, 0xb60, 0xb58, 0xb50, 0xb48, 0xb40,
    0xb38, 0xb32, 0xb2a, 0xb22, 0xb1a, 0xb12, 0xb0a, 0xb02, 0xafc, 0xaf4, 0xaec, 0xae4, 0xade,
    0xad6, 0xace, 0xac6, 0xac0, 0xab8, 0xab0, 0xaa8, 0xaa2, 0xa9a, 0xa92, 0xa8c, 0xa84, 0xa7c,
    0xa76, 0xa6e, 0xa68, 0xa60, 0xa58, 0xa52, 0xa4a, 0xa44, 0xa3c, 0xa36, 0xa2e, 0xa28, 0xa20,
    0xa18, 0xa12, 0xa0c, 0xa04, 0x9fe, 0x9f6, 0x9f0, 0x9e8, 0x9e2, 0x9da, 0x9d4, 0x9ce, 0x9c6,
    0x9c0, 0x9b8, 0x9b2, 0x9ac, 0x9a4, 0x99e, 0x998, 0x990, 0x98a, 0x984, 0x97c, 0x976, 0x970,
    0x96a, 0x962, 0x95c, 0x956, 0x950, 0x948, 0x942, 0x93c, 0x936, 0x930, 0x928, 0x922, 0x91c,
    0x916, 0x910, 0x90a, 0x904, 0x8fc, 0x8f6, 0x8f0, 0x8ea, 0x8e4, 0x8de, 0x8d8, 0x8d2, 0x8cc,
    0x8c6, 0x8c0, 0x8ba, 0x8b4, 0x8ae, 0x8a8, 0x8a2, 0x89c, 0x896, 0x890, 0x88a, 0x884, 0x87e,
    0x878, 0x872, 0x86c, 0x866, 0x860, 0x85a, 0x854, 0x850, 0x84a, 0x844, 0x83e, 0x838, 0x832,
    0x82c, 0x828, 0x822, 0x81c, 0x816, 0x810, 0x80c, 0x806, 0x800,
];

const MT: [u8; 16] = [1, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 20, 24, 24, 30, 30];
const KSLROM: [u8; 16] = [
    0, 32, 40, 45, 48, 51, 53, 55, 56, 58, 59, 60, 61, 62, 63, 64,
];
const KSLSHIFT: [u8; 4] = [8, 1, 2, 0];
const EG_INCSTEP: [[u8; 4]; 4] = [[0, 0, 0, 0], [1, 0, 0, 0], [1, 0, 1, 0], [1, 1, 1, 0]];
const AD_SLOT: [i8; 0x20] = [
    0, 1, 2, 3, 4, 5, -1, -1, 6, 7, 8, 9, 10, 11, -1, -1, 12, 13, 14, 15, 16, 17, -1, -1, -1, -1,
    -1, -1, -1, -1, -1, -1,
];
const CH_SLOT: [usize; 18] = [
    0, 1, 2, 6, 7, 8, 12, 13, 14, 18, 19, 20, 24, 25, 26, 30, 31, 32,
];
#[derive(Clone, Copy, Debug)]
struct Slot {
    channel: usize,
    mod_idx: usize,
    prout: i16,
    eg_rout: u16,
    eg_out: u16,
    eg_gen: u8,
    eg_ksl: u8,
    eg_tl_ksl: u16,
    eg_ks: u8,
    // Per-eg_gen cached rate fields. Index into these by eg_gen (or 0 when in
    // the key-on/release reset case). Refreshed by refresh_eg_rates whenever
    // any contributing input changes (eg_ks, reg_ar/dr/rr, reg_type).
    eg_rate_hi: [u8; 4],
    eg_rate_lo: [u8; 4],
    eg_rate_nonzero_mask: u8,
    trem_enabled: bool,
    reg_vib: u8,
    reg_type: u8,
    reg_ksr: u8,
    reg_mult: u8,
    reg_ksl: u8,
    reg_tl: u8,
    reg_ar: u8,
    reg_dr: u8,
    reg_sl: u8,
    reg_rr: u8,
    reg_wf: u8,
    key: u8,
    pg_reset: u32,
    pg_phase: u32,
    pg_inc: u32,
    pg_inc_vib: [u32; 8],
    pg_phase_out: u16,
    slot_num: usize,
}

impl Default for Slot {
    fn default() -> Self {
        Self {
            channel: 0,
            mod_idx: 72,
            prout: 0,
            eg_rout: 0,
            eg_out: 0,
            eg_gen: 0,
            eg_ksl: 0,
            eg_tl_ksl: 0,
            eg_ks: 0,
            eg_rate_hi: [0; 4],
            eg_rate_lo: [0; 4],
            eg_rate_nonzero_mask: 0,
            trem_enabled: false,
            reg_vib: 0,
            reg_type: 0,
            reg_ksr: 0,
            reg_mult: 0,
            reg_ksl: 0,
            reg_tl: 0,
            reg_ar: 0,
            reg_dr: 0,
            reg_sl: 0,
            reg_rr: 0,
            reg_wf: 0,
            key: 0,
            pg_reset: 0,
            pg_phase: 0,
            pg_inc: 0,
            pg_inc_vib: [0; 8],
            pg_phase_out: 0,
            slot_num: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Channel {
    slotz: [usize; 2],
    pair: Option<usize>,
    out: [usize; 4],
    chtype: u8,
    f_num: u16,
    block: u8,
    fb: u8,
    con: u8,
    alg: u8,
    ksv: u8,
    cha: u16,
    chb: u16,
    chc: u16,
    chd: u16,
    mix_mask: u8,
    leftpan: i32,
    rightpan: i32,
    ch_num: usize,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            slotz: [0, 0],
            pair: None,
            out: [72; 4],
            chtype: CH_2OP,
            f_num: 0,
            block: 0,
            fb: 0,
            con: 0,
            alg: 0,
            ksv: 0,
            cha: 0,
            chb: 0,
            chc: 0,
            chd: 0,
            mix_mask: 0,
            leftpan: 0,
            rightpan: 0,
            ch_num: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct WriteBuf {
    time: u64,
    reg: u16,
    data: u8,
}

#[derive(Clone, Debug)]
pub(crate) struct Chip {
    channels: [Channel; 18],
    slots: [Slot; 36],
    outputs: [i16; 74],
    timer: u16,
    eg_timer: u64,
    eg_timerrem: u8,
    eg_state: u8,
    eg_add: u8,
    eg_timer_lo: u8,
    newm: u8,
    nts: u8,
    rhy: u8,
    vibpos: u8,
    vibshift: u8,
    tremolo: u8,
    tremolopos: u8,
    tremoloshift: u8,
    noise: u32,
    noise_hh: u8,
    noise_sd: u8,
    mixbuff: [i32; 4],
    rm_hh_bit2: u8,
    rm_hh_bit3: u8,
    rm_hh_bit7: u8,
    rm_hh_bit8: u8,
    rm_tc_bit3: u8,
    rm_tc_bit5: u8,
    stereoext: u8,
    rateratio: i32,
    samplecnt: i32,
    oldsamples: [i16; 4],
    samples: [i16; 4],
    writebuf_samplecnt: u64,
    writebuf_cur: usize,
    writebuf_last: usize,
    writebuf_lasttime: u64,
    writebuf: [WriteBuf; WRITEBUF_SIZE],
}

impl Default for Chip {
    fn default() -> Self {
        Self {
            channels: [Channel::default(); 18],
            slots: [Slot::default(); 36],
            outputs: [0; 74],
            timer: 0,
            eg_timer: 0,
            eg_timerrem: 0,
            eg_state: 0,
            eg_add: 0,
            eg_timer_lo: 0,
            newm: 0,
            nts: 0,
            rhy: 0,
            vibpos: 0,
            vibshift: 0,
            tremolo: 0,
            tremolopos: 0,
            tremoloshift: 0,
            noise: 0,
            noise_hh: 0,
            noise_sd: 0,
            mixbuff: [0; 4],
            rm_hh_bit2: 0,
            rm_hh_bit3: 0,
            rm_hh_bit7: 0,
            rm_hh_bit8: 0,
            rm_tc_bit3: 0,
            rm_tc_bit5: 0,
            stereoext: 0,
            rateratio: 0,
            samplecnt: 0,
            oldsamples: [0; 4],
            samples: [0; 4],
            writebuf_samplecnt: 0,
            writebuf_cur: 0,
            writebuf_last: 0,
            writebuf_lasttime: 0,
            writebuf: [WriteBuf::default(); WRITEBUF_SIZE],
        }
    }
}

impl Chip {
    pub(crate) fn new(sample_rate: u32) -> Self {
        let mut chip = Self::default();
        chip.reset(sample_rate);
        chip
    }

    pub(crate) fn reset(&mut self, sample_rate: u32) {
        *self = Self::default();

        for slotnum in 0..36 {
            self.slots[slotnum] = Slot {
                eg_rout: 0x1ff,
                eg_out: 0x1ff,
                eg_gen: ENV_RELEASE,
                slot_num: slotnum,
                ..Slot::default()
            };
        }

        for (channum, &local_ch_slot) in CH_SLOT.iter().enumerate() {
            self.channels[channum] = Channel {
                slotz: [local_ch_slot, local_ch_slot + 3],
                pair: if (channum % 9) < 3 {
                    Some(channum + 3)
                } else if (channum % 9) < 6 {
                    Some(channum - 3)
                } else {
                    None
                },
                chtype: CH_2OP,
                cha: 0xffff,
                chb: 0xffff,
                mix_mask: 0x03,
                leftpan: 0x10000,
                rightpan: 0x10000,
                ch_num: channum,
                ..Channel::default()
            };
            self.slots[local_ch_slot].channel = channum;
            self.slots[local_ch_slot + 3].channel = channum;
            self.channel_setup_alg(channum);
        }

        self.noise = 1;
        self.rateratio = ((sample_rate << RSM_FRAC) / NATIVE_RATE) as i32;
        self.tremoloshift = 4;
        self.vibshift = 1;
    }

    pub(crate) fn generate(&mut self, buf: &mut [i16]) {
        let mut samples = [0; 4];
        self.generate_4ch(&mut samples);
        buf[0] = samples[0];
        buf[1] = samples[1];
    }

    pub(crate) fn generate_resampled(&mut self, buf: &mut [i16]) {
        let mut samples = [0; 4];
        self.generate_4ch_resampled(&mut samples);
        buf[0] = samples[0];
        buf[1] = samples[1];
    }

    pub(crate) fn generate_stream(&mut self, buf: &mut [i16], numsamples: usize) {
        for frame in buf.chunks_exact_mut(2).take(numsamples) {
            self.generate_resampled(frame);
        }
    }

    pub(crate) fn generate_4ch(&mut self, buf4: &mut [i16]) {
        buf4[1] = clip_sample(self.mixbuff[1]);
        buf4[3] = clip_sample(self.mixbuff[3]);

        {
            let s = self.noise;
            let f0_8 = (s ^ (s >> 14)) & 0x1ff;
            let f9_17 = ((s >> 9) ^ f0_8) & 0x1ff;
            let f18_22 = ((s >> 18) ^ f9_17) & 0x1f;
            let f23_31 = f0_8 ^ ((f9_17 >> 5) | (f18_22 << 4));
            let f32_35 = (f9_17 ^ f23_31) & 0x0f;
            self.noise_hh = ((s >> 13) & 1) as u8;
            self.noise_sd = ((s >> 16) & 1) as u8;
            self.noise = ((f9_17 >> 4) & 0x1f) | (f18_22 << 5) | (f23_31 << 10) | (f32_35 << 19);
        }

        if CHANNEL_SAMPLE_DELAY {
            for slot in 0..15 {
                self.process_slot(slot);
            }

            let mix = self.mix(false);
            self.mixbuff[0] = mix[0];
            self.mixbuff[2] = mix[1];

            for slot in 15..18 {
                self.process_slot(slot);
            }

            buf4[0] = clip_sample(self.mixbuff[0]);
            buf4[2] = clip_sample(self.mixbuff[2]);

            for slot in 18..33 {
                self.process_slot(slot);
            }

            let mix = self.mix(true);
            self.mixbuff[1] = mix[0];
            self.mixbuff[3] = mix[1];

            for slot in 33..36 {
                self.process_slot(slot);
            }
        } else {
            for slot in 0..36 {
                self.process_slot(slot);
            }

            let mix = self.mix(false);
            self.mixbuff[0] = mix[0];
            self.mixbuff[2] = mix[1];

            buf4[0] = clip_sample(self.mixbuff[0]);
            buf4[2] = clip_sample(self.mixbuff[2]);

            let mix = self.mix(true);
            self.mixbuff[1] = mix[0];
            self.mixbuff[3] = mix[1];
        }

        self.update_timers_and_lfo();
        self.drain_writebuf();
    }

    pub(crate) fn generate_4ch_resampled(&mut self, buf4: &mut [i16]) {
        while self.samplecnt >= self.rateratio {
            self.oldsamples = self.samples;
            let mut samples = [0; 4];
            self.generate_4ch(&mut samples);
            self.samples = samples;
            self.samplecnt -= self.rateratio;
        }

        for (ii, out) in buf4.iter_mut().enumerate() {
            *out = ((self.oldsamples[ii] as i32 * (self.rateratio - self.samplecnt)
                + self.samples[ii] as i32 * self.samplecnt)
                / self.rateratio) as i16;
        }
        self.samplecnt += 1 << RSM_FRAC;
    }

    pub(crate) fn generate_4ch_stream(
        &mut self,
        buf1: &mut [i16],
        buf2: &mut [i16],
        numsamples: usize,
    ) {
        let mut samples = [0; 4];
        for frame in 0..numsamples {
            self.generate_4ch_resampled(&mut samples);
            let idx = frame * 2;
            buf1[idx] = samples[0];
            buf1[idx + 1] = samples[1];
            buf2[idx] = samples[2];
            buf2[idx + 1] = samples[3];
        }
    }

    pub(crate) fn active_voice_count(&self) -> usize {
        self.slots[..36]
            .iter()
            .filter(|slot| !(slot.key == 0 && slot.eg_rout == 0x1ff && slot.eg_gen == ENV_RELEASE))
            .count()
    }

    pub(crate) fn write_reg_buffered(&mut self, reg: u16, data: u8) {
        let writebuf_last = self.writebuf_last;

        if (self.writebuf[writebuf_last].reg & 0x200) != 0 {
            let reg = self.writebuf[writebuf_last].reg & 0x1ff;
            let data = self.writebuf[writebuf_last].data;
            let time = self.writebuf[writebuf_last].time;
            self.write_reg(reg, data);
            self.writebuf_cur = (writebuf_last + 1) % WRITEBUF_SIZE;
            self.writebuf_samplecnt = time;
        }

        self.writebuf[writebuf_last].reg = reg | 0x200;
        self.writebuf[writebuf_last].data = data;
        let mut time = self.writebuf_lasttime.wrapping_add(WRITEBUF_DELAY);
        if time < self.writebuf_samplecnt {
            time = self.writebuf_samplecnt;
        }

        self.writebuf[writebuf_last].time = time;
        self.writebuf_lasttime = time;
        self.writebuf_last = (writebuf_last + 1) % WRITEBUF_SIZE;
    }

    pub(crate) fn write_reg(&mut self, reg: u16, value: u8) {
        let high = ((reg >> 8) & 0x01) as usize;
        let regm = (reg & 0xff) as u8;

        match regm & 0xf0 {
            0x00 => {
                if high != 0 {
                    match regm & 0x0f {
                        0x04 => self.channel_set_4op(value),
                        0x05 => {
                            self.newm = value & 0x01;
                            if false {
                                self.stereoext = (value >> 1) & 0x01;
                            }
                        }
                        _ => {}
                    }
                } else if (regm & 0x0f) == 0x08 {
                    self.nts = (value >> 6) & 0x01;
                }
            }
            0x20 | 0x30 => {
                if let Some(slot) = decoded_slot(high, regm) {
                    self.slot_write_20(slot, value);
                }
            }
            0x40 | 0x50 => {
                if let Some(slot) = decoded_slot(high, regm) {
                    self.slot_write_40(slot, value);
                }
            }
            0x60 | 0x70 => {
                if let Some(slot) = decoded_slot(high, regm) {
                    self.slot_write_60(slot, value);
                }
            }
            0x80 | 0x90 => {
                if let Some(slot) = decoded_slot(high, regm) {
                    self.slot_write_80(slot, value);
                }
            }
            0xe0 | 0xf0 => {
                if let Some(slot) = decoded_slot(high, regm) {
                    self.slot_write_e0(slot, value);
                }
            }
            0xa0 => {
                if (regm & 0x0f) < 9 {
                    self.channel_write_a0(9 * high + (regm & 0x0f) as usize, value);
                }
            }
            0xb0 => {
                if regm == 0xbd && high == 0 {
                    self.tremoloshift = (((value >> 7) ^ 1) << 1) + 2;
                    let vibshift = ((value >> 6) & 0x01) ^ 1;
                    if self.vibshift != vibshift {
                        self.vibshift = vibshift;
                        for ii in 0..36 {
                            self.phase_update_inc(ii);
                        }
                    }
                    self.channel_update_rhythm(value);
                } else if (regm & 0x0f) < 9 {
                    let channel = 9 * high + (regm & 0x0f) as usize;
                    self.channel_write_b0(channel, value);
                    if (value & 0x20) != 0 {
                        self.channel_key_on(channel);
                    } else {
                        self.channel_key_off(channel);
                    }
                }
            }
            0xc0 => {
                if (regm & 0x0f) < 9 {
                    self.channel_write_c0(9 * high + (regm & 0x0f) as usize, value);
                }
            }
            0xd0 if false && (regm & 0x0f) < 9 => {
                self.channel_write_d0(9 * high + (regm & 0x0f) as usize, value);
            }
            _ => {}
        }
    }

    fn process_slot(&mut self, slot: usize) {
        self.slot_calc_fb(slot);

        // Two fast paths skip the full envelope_calc + phase_generate work:
        //
        //   * Dead-slot path:     key off and eg_rout already at full attenuation.
        //   * Sustain-rate-zero:  key on, in sustain phase, with the cached sustain
        //                         rate equal to zero (the common "hold forever" case).
        //
        // Both still need slot_calc_fb above and slot_generate below so the
        // waveform's sign bit can flip slot.out to -1 even at full attenuation
        // (used as a modulator). Rhythm slots 13/16/17 are excluded so their
        // rm_hh_bit*/rm_tc_bit* updates and rhythm-mode phase overrides keep
        // running.
        let slot_num = self.slots[slot].slot_num;
        let is_rhythm = matches!(slot_num, 13 | 16 | 17);
        if !is_rhythm {
            let key = self.slots[slot].key;
            let eg_rout = self.slots[slot].eg_rout;
            let eg_gen = self.slots[slot].eg_gen;
            let dead = key == 0 && eg_rout == 0x1ff;
            let sustain_zero = key != 0
                && eg_gen == ENV_SUSTAIN
                && (self.slots[slot].eg_rate_nonzero_mask & (1 << ENV_SUSTAIN)) == 0;

            if dead || sustain_zero {
                // Mirror envelope_calc's tail: a key-off forces eg_gen back to
                // RELEASE. Without this the next key-on can't tell that the
                // slot is fully released and skips the attack-reset branch.
                if dead {
                    self.slots[slot].eg_gen = ENV_RELEASE;
                }
                // Mirror the eg_off clamp in envelope_calc so eg_rout is held
                // at 0x1ff once any of bits 3..8 latch high (and we're not in
                // attack, where the clamp is suppressed).
                if self.slots[slot].eg_gen != ENV_ATTACK && (eg_rout & 0x1f8) == 0x1f8 {
                    self.slots[slot].eg_rout = 0x1ff;
                }
                let trem = if self.slots[slot].trem_enabled {
                    self.tremolo
                } else {
                    0
                };
                self.slots[slot].eg_out =
                    self.slots[slot].eg_rout + self.slots[slot].eg_tl_ksl + u16::from(trem);
                self.slots[slot].pg_reset = 0;

                if self.slots[slot].reg_vib == 0 {
                    let phase = (self.slots[slot].pg_phase >> 9) as u16;
                    self.slots[slot].pg_phase_out = phase;
                    self.slots[slot].pg_phase = self.slots[slot]
                        .pg_phase
                        .wrapping_add(self.slots[slot].pg_inc);
                } else {
                    // Vibrato is rare for held notes; fall through to the
                    // standard phase generator.
                    self.phase_generate(slot);
                }

                // On the dead path, eg_rout was 0x1ff so eg_out is also
                // >= 0x1ff and the silent-regime shortcut applies. On the
                // sustain-zero path, eg_rout sits wherever sustain landed,
                // so the full path is needed.
                if dead {
                    self.slot_generate_silent(slot);
                } else {
                    self.slot_generate(slot);
                }
                return;
            }
        }

        self.envelope_calc(slot);
        self.phase_generate(slot);
        self.slot_generate(slot);
    }

    #[inline(always)]
    fn slot_calc_fb(&mut self, slot: usize) {
        let channel = self.slots[slot].channel;
        let fb = self.channels[channel].fb;
        self.outputs[slot * 2 + 1] = if fb != 0 {
            (self.slots[slot].prout.wrapping_add(self.outputs[slot * 2])) >> (0x09 - fb)
        } else {
            0
        };
        self.slots[slot].prout = self.outputs[slot * 2];
    }

    fn envelope_calc(&mut self, slot: usize) {
        let trem = if self.slots[slot].trem_enabled {
            self.tremolo
        } else {
            0
        };

        self.slots[slot].eg_out =
            self.slots[slot].eg_rout + self.slots[slot].eg_tl_ksl + trem as u16;

        let mut reset = 0;
        // Pick the rate cache slot. The key-on/release reset case reuses the
        // attack rate (index 0), matching the original branch in the C code.
        let rate_idx = if self.slots[slot].key != 0 && self.slots[slot].eg_gen == ENV_RELEASE {
            reset = 1;
            0
        } else {
            (self.slots[slot].eg_gen & 0x03) as usize
        };
        self.slots[slot].pg_reset = reset;

        let rate_hi = self.slots[slot].eg_rate_hi[rate_idx];
        let rate_lo = self.slots[slot].eg_rate_lo[rate_idx];
        let nonzero = ((self.slots[slot].eg_rate_nonzero_mask >> rate_idx) & 1) != 0;

        let eg_shift = rate_hi + self.eg_add;
        let mut shift = 0;
        if nonzero {
            if rate_hi < 12 {
                if self.eg_state != 0 {
                    shift = match eg_shift {
                        12 => 1,
                        13 => (rate_lo >> 1) & 0x01,
                        14 => rate_lo & 0x01,
                        _ => 0,
                    };
                }
            } else {
                shift = (rate_hi & 0x03) + EG_INCSTEP[rate_lo as usize][self.eg_timer_lo as usize];
                if (shift & 0x04) != 0 {
                    shift = 0x03;
                }
                if shift == 0 {
                    shift = self.eg_state;
                }
            }
        }

        let mut eg_rout = self.slots[slot].eg_rout;
        let mut eg_inc: i32 = 0;
        let eg_off = (self.slots[slot].eg_rout & 0x1f8) == 0x1f8;

        if reset != 0 && rate_hi == 0x0f {
            eg_rout = 0;
        }
        if self.slots[slot].eg_gen != ENV_ATTACK && reset == 0 && eg_off {
            eg_rout = 0x1ff;
        }

        match self.slots[slot].eg_gen {
            ENV_ATTACK => {
                if self.slots[slot].eg_rout == 0 {
                    self.slots[slot].eg_gen = ENV_DECAY;
                } else if self.slots[slot].key != 0 && shift > 0 && rate_hi != 0x0f {
                    eg_inc = (!(self.slots[slot].eg_rout as i32)) >> (4 - shift);
                }
            }
            ENV_DECAY => {
                if (self.slots[slot].eg_rout >> 4) == self.slots[slot].reg_sl as u16 {
                    self.slots[slot].eg_gen = ENV_SUSTAIN;
                } else if !eg_off && reset == 0 && shift > 0 {
                    eg_inc = 1 << (shift - 1);
                }
            }
            ENV_SUSTAIN | ENV_RELEASE if !eg_off && reset == 0 && shift > 0 => {
                eg_inc = 1 << (shift - 1);
            }
            _ => {}
        }

        self.slots[slot].eg_rout = ((eg_rout as i32 + eg_inc) & 0x1ff) as u16;
        if reset != 0 {
            self.slots[slot].eg_gen = ENV_ATTACK;
        }
        if self.slots[slot].key == 0 {
            self.slots[slot].eg_gen = ENV_RELEASE;
        }
    }

    fn phase_generate(&mut self, slot: usize) {
        let phase_delta = if self.slots[slot].reg_vib != 0 {
            self.slots[slot].pg_inc_vib[self.vibpos as usize]
        } else {
            self.slots[slot].pg_inc
        };

        let phase = (self.slots[slot].pg_phase >> 9) as u16;
        if self.slots[slot].pg_reset != 0 {
            self.slots[slot].pg_phase = 0;
        }
        self.slots[slot].pg_phase = self.slots[slot].pg_phase.wrapping_add(phase_delta);

        let rhy_enabled = (self.rhy & 0x20) != 0;
        self.slots[slot].pg_phase_out = phase;
        match self.slots[slot].slot_num {
            13 => {
                self.rm_hh_bit2 = ((phase >> 2) & 1) as u8;
                self.rm_hh_bit3 = ((phase >> 3) & 1) as u8;
                self.rm_hh_bit7 = ((phase >> 7) & 1) as u8;
                self.rm_hh_bit8 = ((phase >> 8) & 1) as u8;
                if rhy_enabled {
                    let rm_xor = (self.rm_hh_bit2 ^ self.rm_hh_bit7)
                        | (self.rm_hh_bit3 ^ self.rm_tc_bit5)
                        | (self.rm_tc_bit3 ^ self.rm_tc_bit5);
                    self.slots[slot].pg_phase_out = (rm_xor as u16) << 9;
                    if (rm_xor ^ self.noise_hh) != 0 {
                        self.slots[slot].pg_phase_out |= 0xd0;
                    } else {
                        self.slots[slot].pg_phase_out |= 0x34;
                    }
                }
            }
            16 => {
                if rhy_enabled {
                    self.slots[slot].pg_phase_out = ((self.rm_hh_bit8 as u16) << 9)
                        | (((self.rm_hh_bit8 ^ self.noise_sd) as u16) << 8);
                }
            }
            17 if rhy_enabled => {
                self.rm_tc_bit3 = ((phase >> 3) & 1) as u8;
                self.rm_tc_bit5 = ((phase >> 5) & 1) as u8;
                let rm_xor = (self.rm_hh_bit2 ^ self.rm_hh_bit7)
                    | (self.rm_hh_bit3 ^ self.rm_tc_bit5)
                    | (self.rm_tc_bit3 ^ self.rm_tc_bit5);
                self.slots[slot].pg_phase_out = ((rm_xor as u16) << 9) | 0x80;
            }
            _ => {}
        }
    }

    fn slot_generate(&mut self, slot: usize) {
        let mod_value = self.outputs[self.slots[slot].mod_idx];
        let phase = (self.slots[slot].pg_phase_out as i32 + mod_value as i32) as u16;
        self.outputs[slot * 2] =
            envelope_calc_sin(self.slots[slot].reg_wf, phase, self.slots[slot].eg_out);
    }

    /// Silent-regime variant of slot_generate. When the caller has proven
    /// eg_out >= 0x180, the post-clamp exprom lookup in envelope_calc_exp
    /// is shifted right by at least 12 bits. The exprom max value is
    /// 0xff4, so the result is always 0 and the final output reduces to
    /// just the sign bit of the WF_ROM entry. Verified bit-exact across
    /// all 8 waveforms and all phases against the full path.
    fn slot_generate_silent(&mut self, slot: usize) {
        let mod_value = self.outputs[self.slots[slot].mod_idx];
        let phase = (self.slots[slot].pg_phase_out as i32 + mod_value as i32) as u16;
        let idx = (((self.slots[slot].reg_wf & 0x07) as usize) << 10) | ((phase & 0x3ff) as usize);
        self.outputs[slot * 2] = (WF_ROM[idx] as i16) >> 15;
    }

    fn mix(&self, right: bool) -> [i32; 2] {
        let mut mix = [0, 0];
        let active_bit = if right { 0x02 } else { 0x01 };
        for channel in &self.channels {
            if (channel.mix_mask & active_bit) == 0 {
                continue;
            }

            let sum = self.outputs[channel.out[0]] as i32
                + self.outputs[channel.out[1]] as i32
                + self.outputs[channel.out[2]] as i32
                + self.outputs[channel.out[3]] as i32;
            let accm = sum as i16;
            let main = if false {
                let pan = if right {
                    channel.rightpan
                } else {
                    channel.leftpan
                };
                ((accm as i32).wrapping_mul(pan) >> 16) as i16
            } else {
                let main_mask = if right { channel.chb } else { channel.cha };
                masked_accm(accm, main_mask)
            };
            let rear_mask = if right { channel.chd } else { channel.chc };
            mix[0] += main as i32;
            mix[1] += masked_accm(accm, rear_mask) as i32;
        }
        mix
    }

    fn update_timers_and_lfo(&mut self) {
        if (self.timer & 0x3f) == 0x3f {
            self.tremolopos = (self.tremolopos + 1) % 210;
        }
        self.tremolo = if self.tremolopos < 105 {
            self.tremolopos >> self.tremoloshift
        } else {
            (210 - self.tremolopos) >> self.tremoloshift
        };

        if (self.timer & 0x3ff) == 0x3ff {
            self.vibpos = (self.vibpos + 1) & 7;
        }

        self.timer = self.timer.wrapping_add(1);

        if self.eg_state != 0 {
            // Same shape as the C session-6 ctz path: only the low 13 bits
            // matter, narrowed to u32 so trailing_zeros maps to a single x86
            // bsf/tzcnt. eg_add stays 0 when those bits are all zero (matches
            // the original "shift > 12" guard).
            let masked = (self.eg_timer & 0x1fff) as u32;
            self.eg_add = if masked == 0 {
                0
            } else {
                (masked.trailing_zeros() + 1) as u8
            };
            self.eg_timer_lo = (self.eg_timer & 0x3) as u8;
        }

        if self.eg_timerrem != 0 || self.eg_state != 0 {
            if self.eg_timer == 0xfffffffff {
                self.eg_timer = 0;
                self.eg_timerrem = 1;
            } else {
                self.eg_timer += 1;
                self.eg_timerrem = 0;
            }
        }

        self.eg_state ^= 1;
    }

    fn drain_writebuf(&mut self) {
        loop {
            let entry = self.writebuf[self.writebuf_cur];
            if entry.time > self.writebuf_samplecnt || (entry.reg & 0x200) == 0 {
                break;
            }
            self.writebuf[self.writebuf_cur].reg &= 0x1ff;
            self.write_reg(entry.reg & 0x1ff, entry.data);
            self.writebuf_cur = (self.writebuf_cur + 1) % WRITEBUF_SIZE;
        }
        self.writebuf_samplecnt = self.writebuf_samplecnt.wrapping_add(1);
    }

    fn slot_write_20(&mut self, slot: usize, data: u8) {
        self.slots[slot].trem_enabled = ((data >> 7) & 0x01) != 0;
        self.slots[slot].reg_vib = (data >> 6) & 0x01;
        self.slots[slot].reg_type = (data >> 5) & 0x01;
        self.slots[slot].reg_ksr = (data >> 4) & 0x01;
        self.slots[slot].reg_mult = data & 0x0f;
        self.envelope_update_rate(slot);
        self.phase_update_inc(slot);
    }

    fn slot_write_40(&mut self, slot: usize, data: u8) {
        self.slots[slot].reg_ksl = (data >> 6) & 0x03;
        self.slots[slot].reg_tl = data & 0x3f;
        self.envelope_update_ksl(slot);
    }

    fn slot_write_60(&mut self, slot: usize, data: u8) {
        self.slots[slot].reg_ar = (data >> 4) & 0x0f;
        self.slots[slot].reg_dr = data & 0x0f;
        self.refresh_eg_rates(slot);
    }

    fn slot_write_80(&mut self, slot: usize, data: u8) {
        self.slots[slot].reg_sl = (data >> 4) & 0x0f;
        if self.slots[slot].reg_sl == 0x0f {
            self.slots[slot].reg_sl = 0x1f;
        }
        self.slots[slot].reg_rr = data & 0x0f;
        self.refresh_eg_rates(slot);
    }

    fn slot_write_e0(&mut self, slot: usize, data: u8) {
        self.slots[slot].reg_wf = data & 0x07;
        if self.newm == 0 {
            self.slots[slot].reg_wf &= 0x03;
        }
    }

    fn envelope_update_ksl(&mut self, slot: usize) {
        let channel = self.slots[slot].channel;
        let mut ksl = ((KSLROM[(self.channels[channel].f_num >> 6) as usize] as i16) << 2)
            - (((0x08 - self.channels[channel].block) as i16) << 5);
        if ksl < 0 {
            ksl = 0;
        }
        self.slots[slot].eg_ksl = ksl as u8;
        self.slots[slot].eg_tl_ksl = ((self.slots[slot].reg_tl as u16) << 2)
            + ((self.slots[slot].eg_ksl as u16) >> KSLSHIFT[self.slots[slot].reg_ksl as usize]);
    }

    fn envelope_update_rate(&mut self, slot: usize) {
        let channel = self.slots[slot].channel;
        self.slots[slot].eg_ks =
            self.channels[channel].ksv >> ((self.slots[slot].reg_ksr ^ 1) << 1);
        self.refresh_eg_rates(slot);
    }

    fn refresh_eg_rates(&mut self, slot: usize) {
        let s = &mut self.slots[slot];
        let ks = s.eg_ks;
        let mut nz_mask: u8 = 0;
        for idx in 0..4 {
            // Mirror the per-state reg_rate selection used by envelope_calc.
            let reg_rate = match idx {
                // ENV_ATTACK (also reused for the key-on/release reset case)
                0 => s.reg_ar,
                // ENV_DECAY
                1 => s.reg_dr,
                // ENV_SUSTAIN: percussive types hold at 0 instead of releasing
                2 => {
                    if s.reg_type == 0 {
                        s.reg_rr
                    } else {
                        0
                    }
                }
                // ENV_RELEASE
                _ => s.reg_rr,
            };
            let rate = ks + (reg_rate << 2);
            let mut rate_hi = rate >> 2;
            if (rate_hi & 0x10) != 0 {
                rate_hi = 0x0f;
            }
            s.eg_rate_hi[idx] = rate_hi;
            s.eg_rate_lo[idx] = rate & 0x03;
            if reg_rate != 0 {
                nz_mask |= 1 << idx;
            }
        }
        s.eg_rate_nonzero_mask = nz_mask;
    }

    fn phase_update_inc(&mut self, slot: usize) {
        let channel = self.slots[slot].channel;
        let basefreq = ((self.channels[channel].f_num as u32) << self.channels[channel].block) >> 1;
        self.slots[slot].pg_inc = (basefreq * MT[self.slots[slot].reg_mult as usize] as u32) >> 1;

        let reg_mult = self.slots[slot].reg_mult as usize;
        let block = self.channels[channel].block;
        for vibpos in 0..8 {
            let mut f_num = self.channels[channel].f_num;
            let mut range = ((f_num >> 7) & 7) as i8;
            if (vibpos & 3) == 0 {
                range = 0;
            } else if (vibpos & 1) != 0 {
                range >>= 1;
            }
            range >>= self.vibshift;
            if (vibpos & 4) != 0 {
                range = -range;
            }
            f_num = (f_num as i32 + range as i32) as u16;
            let basefreq_vib = ((f_num as u32) << block) >> 1;
            self.slots[slot].pg_inc_vib[vibpos] = (basefreq_vib * MT[reg_mult] as u32) >> 1;
        }
    }

    fn envelope_key_on(&mut self, slot: usize, key_type: u8) {
        self.slots[slot].key |= key_type;
    }

    fn envelope_key_off(&mut self, slot: usize, key_type: u8) {
        self.slots[slot].key &= !key_type;
    }

    fn channel_update_rhythm(&mut self, data: u8) {
        self.rhy = data & 0x3f;
        if (self.rhy & 0x20) != 0 {
            self.channels[6].out = [
                self.channels[6].slotz[1] * 2,
                self.channels[6].slotz[1] * 2,
                72,
                72,
            ];
            self.channels[7].out = [
                self.channels[7].slotz[0] * 2,
                self.channels[7].slotz[0] * 2,
                self.channels[7].slotz[1] * 2,
                self.channels[7].slotz[1] * 2,
            ];
            self.channels[8].out = [
                self.channels[8].slotz[0] * 2,
                self.channels[8].slotz[0] * 2,
                self.channels[8].slotz[1] * 2,
                self.channels[8].slotz[1] * 2,
            ];
            for channel in 6..9 {
                self.channels[channel].chtype = CH_DRUM;
            }
            for channel in 6..9 {
                self.channel_setup_alg(channel);
            }

            self.set_drum_key(7, 0, self.rhy & 0x01);
            self.set_drum_key(8, 1, self.rhy & 0x02);
            self.set_drum_key(8, 0, self.rhy & 0x04);
            self.set_drum_key(7, 1, self.rhy & 0x08);
            self.set_drum_key(6, 0, self.rhy & 0x10);
            self.set_drum_key(6, 1, self.rhy & 0x10);
        } else {
            for channel in 6..9 {
                self.channels[channel].chtype = CH_2OP;
                self.channel_setup_alg(channel);
                let slot0 = self.channels[channel].slotz[0];
                let slot1 = self.channels[channel].slotz[1];
                self.envelope_key_off(slot0, EGK_DRUM);
                self.envelope_key_off(slot1, EGK_DRUM);
            }
        }
    }

    fn set_drum_key(&mut self, channel: usize, slot_index: usize, mask: u8) {
        let slot = self.channels[channel].slotz[slot_index];
        if mask != 0 {
            self.envelope_key_on(slot, EGK_DRUM);
        } else {
            self.envelope_key_off(slot, EGK_DRUM);
        }
    }

    fn channel_write_a0(&mut self, channel: usize, data: u8) {
        if self.newm != 0 && self.channels[channel].chtype == CH_4OP2 {
            return;
        }

        self.channels[channel].f_num = (self.channels[channel].f_num & 0x300) | data as u16;
        self.channels[channel].ksv = (self.channels[channel].block << 1)
            | ((self.channels[channel].f_num >> (0x09 - self.nts)) as u8 & 0x01);
        self.update_channel_slots_frequency_dependent(channel);

        if self.newm != 0 && self.channels[channel].chtype == CH_4OP {
            let pair = self.channels[channel].pair.unwrap();
            self.channels[pair].f_num = self.channels[channel].f_num;
            self.channels[pair].ksv = self.channels[channel].ksv;
            self.update_channel_slots_frequency_dependent(pair);
        }
    }

    fn channel_write_b0(&mut self, channel: usize, data: u8) {
        if self.newm != 0 && self.channels[channel].chtype == CH_4OP2 {
            return;
        }

        self.channels[channel].f_num =
            (self.channels[channel].f_num & 0xff) | (((data & 0x03) as u16) << 8);
        self.channels[channel].block = (data >> 2) & 0x07;
        self.channels[channel].ksv = (self.channels[channel].block << 1)
            | ((self.channels[channel].f_num >> (0x09 - self.nts)) as u8 & 0x01);
        self.update_channel_slots_frequency_dependent(channel);

        if self.newm != 0 && self.channels[channel].chtype == CH_4OP {
            let pair = self.channels[channel].pair.unwrap();
            self.channels[pair].f_num = self.channels[channel].f_num;
            self.channels[pair].block = self.channels[channel].block;
            self.channels[pair].ksv = self.channels[channel].ksv;
            self.update_channel_slots_frequency_dependent(pair);
        }
    }

    fn update_channel_slots_frequency_dependent(&mut self, channel: usize) {
        let slot0 = self.channels[channel].slotz[0];
        let slot1 = self.channels[channel].slotz[1];
        self.envelope_update_ksl(slot0);
        self.envelope_update_ksl(slot1);
        self.envelope_update_rate(slot0);
        self.envelope_update_rate(slot1);
        self.phase_update_inc(slot0);
        self.phase_update_inc(slot1);
    }

    fn channel_write_c0(&mut self, channel: usize, data: u8) {
        self.channels[channel].fb = (data & 0x0e) >> 1;
        self.channels[channel].con = data & 0x01;
        self.channel_update_alg(channel);

        if self.newm != 0 {
            self.channels[channel].cha = if ((data >> 4) & 0x01) != 0 { 0xffff } else { 0 };
            self.channels[channel].chb = if ((data >> 5) & 0x01) != 0 { 0xffff } else { 0 };
            self.channels[channel].chc = if ((data >> 6) & 0x01) != 0 { 0xffff } else { 0 };
            self.channels[channel].chd = if ((data >> 7) & 0x01) != 0 { 0xffff } else { 0 };
        } else {
            self.channels[channel].cha = 0xffff;
            self.channels[channel].chb = 0xffff;
            self.channels[channel].chc = 0;
            self.channels[channel].chd = 0;
        }

        if false && self.stereoext == 0 {
            self.channels[channel].leftpan = pan_from_channel_mask(self.channels[channel].cha);
            self.channels[channel].rightpan = pan_from_channel_mask(self.channels[channel].chb);
        }
        self.refresh_channel_mix_mask(channel);
    }

    // stereo-ext is fixed off; this register cannot be reached in OPL2 mode.
    fn channel_write_d0(&mut self, _channel: usize, _data: u8) {}

    fn refresh_channel_mix_mask(&mut self, channel: usize) {
        let ch = &mut self.channels[channel];
        let left_active = if false {
            ch.leftpan != 0 || ch.chc != 0
        } else {
            ch.cha != 0 || ch.chc != 0
        };
        let right_active = if false {
            ch.rightpan != 0 || ch.chd != 0
        } else {
            ch.chb != 0 || ch.chd != 0
        };
        ch.mix_mask = u8::from(left_active) | (u8::from(right_active) << 1);
    }

    fn channel_key_on(&mut self, channel: usize) {
        if self.newm != 0 {
            match self.channels[channel].chtype {
                CH_4OP => {
                    let pair = self.channels[channel].pair.unwrap();
                    self.key_channel_slots(channel, true, EGK_NORM);
                    self.key_channel_slots(pair, true, EGK_NORM);
                }
                CH_2OP | CH_DRUM => self.key_channel_slots(channel, true, EGK_NORM),
                _ => {}
            }
        } else {
            self.key_channel_slots(channel, true, EGK_NORM);
        }
    }

    fn channel_key_off(&mut self, channel: usize) {
        if self.newm != 0 {
            match self.channels[channel].chtype {
                CH_4OP => {
                    let pair = self.channels[channel].pair.unwrap();
                    self.key_channel_slots(channel, false, EGK_NORM);
                    self.key_channel_slots(pair, false, EGK_NORM);
                }
                CH_2OP | CH_DRUM => self.key_channel_slots(channel, false, EGK_NORM),
                _ => {}
            }
        } else {
            self.key_channel_slots(channel, false, EGK_NORM);
        }
    }

    fn key_channel_slots(&mut self, channel: usize, on: bool, key_type: u8) {
        let slot0 = self.channels[channel].slotz[0];
        let slot1 = self.channels[channel].slotz[1];
        if on {
            self.envelope_key_on(slot0, key_type);
            self.envelope_key_on(slot1, key_type);
        } else {
            self.envelope_key_off(slot0, key_type);
            self.envelope_key_off(slot1, key_type);
        }
    }

    fn channel_set_4op(&mut self, data: u8) {
        for bit in 0..6 {
            let mut chnum = bit;
            if bit >= 3 {
                chnum += 9 - 3;
            }
            let chnum = chnum as usize;
            if ((data >> bit) & 0x01) != 0 {
                self.channels[chnum].chtype = CH_4OP;
                self.channels[chnum + 3].chtype = CH_4OP2;
                self.channel_update_alg(chnum);
            } else {
                self.channels[chnum].chtype = CH_2OP;
                self.channels[chnum + 3].chtype = CH_2OP;
                self.channel_update_alg(chnum);
                self.channel_update_alg(chnum + 3);
            }
        }
    }

    fn channel_update_alg(&mut self, channel: usize) {
        self.channels[channel].alg = self.channels[channel].con;
        if self.newm != 0 {
            match self.channels[channel].chtype {
                CH_4OP => {
                    let pair = self.channels[channel].pair.unwrap();
                    self.channels[pair].alg =
                        0x04 | (self.channels[channel].con << 1) | self.channels[pair].con;
                    self.channels[channel].alg = 0x08;
                    self.channel_setup_alg(pair);
                }
                CH_4OP2 => {
                    let pair = self.channels[channel].pair.unwrap();
                    self.channels[channel].alg =
                        0x04 | (self.channels[pair].con << 1) | self.channels[channel].con;
                    self.channels[pair].alg = 0x08;
                    self.channel_setup_alg(channel);
                }
                _ => self.channel_setup_alg(channel),
            }
        } else {
            self.channel_setup_alg(channel);
        }
    }

    fn channel_setup_alg(&mut self, channel: usize) {
        let slot0 = self.channels[channel].slotz[0];
        let slot1 = self.channels[channel].slotz[1];

        if self.channels[channel].chtype == CH_DRUM {
            if self.channels[channel].ch_num == 7 || self.channels[channel].ch_num == 8 {
                self.slots[slot0].mod_idx = 72;
                self.slots[slot1].mod_idx = 72;
                return;
            }
            match self.channels[channel].alg & 0x01 {
                0x00 => {
                    self.slots[slot0].mod_idx = slot0 * 2 + 1;
                    self.slots[slot1].mod_idx = slot0 * 2;
                }
                0x01 => {
                    self.slots[slot0].mod_idx = slot0 * 2 + 1;
                    self.slots[slot1].mod_idx = 72;
                }
                _ => {}
            }
            return;
        }

        if (self.channels[channel].alg & 0x08) != 0 {
            return;
        }

        if (self.channels[channel].alg & 0x04) != 0 {
            let pair = self.channels[channel].pair.unwrap();
            let pslot0 = self.channels[pair].slotz[0];
            let pslot1 = self.channels[pair].slotz[1];
            // Pair becomes the muted secondary in a 4-op voice; clear its
            // outputs and the matching count so the mix loop skips it.
            self.channels[pair].out = [72; 4];
            match self.channels[channel].alg & 0x03 {
                0x00 => {
                    self.slots[pslot0].mod_idx = pslot0 * 2 + 1;
                    self.slots[pslot1].mod_idx = pslot0 * 2;
                    self.slots[slot0].mod_idx = pslot1 * 2;
                    self.slots[slot1].mod_idx = slot0 * 2;
                    self.channels[channel].out = [slot1 * 2, 72, 72, 72];
                }
                0x01 => {
                    self.slots[pslot0].mod_idx = pslot0 * 2 + 1;
                    self.slots[pslot1].mod_idx = pslot0 * 2;
                    self.slots[slot0].mod_idx = 72;
                    self.slots[slot1].mod_idx = slot0 * 2;
                    self.channels[channel].out = [pslot1 * 2, slot1 * 2, 72, 72];
                }
                0x02 => {
                    self.slots[pslot0].mod_idx = pslot0 * 2 + 1;
                    self.slots[pslot1].mod_idx = 72;
                    self.slots[slot0].mod_idx = pslot1 * 2;
                    self.slots[slot1].mod_idx = slot0 * 2;
                    self.channels[channel].out = [pslot0 * 2, slot1 * 2, 72, 72];
                }
                0x03 => {
                    self.slots[pslot0].mod_idx = pslot0 * 2 + 1;
                    self.slots[pslot1].mod_idx = 72;
                    self.slots[slot0].mod_idx = pslot1 * 2;
                    self.slots[slot1].mod_idx = 72;
                    self.channels[channel].out = [pslot0 * 2, slot0 * 2, slot1 * 2, 72];
                }
                _ => {}
            }
        } else {
            match self.channels[channel].alg & 0x01 {
                0x00 => {
                    self.slots[slot0].mod_idx = slot0 * 2 + 1;
                    self.slots[slot1].mod_idx = slot0 * 2;
                    self.channels[channel].out = [slot1 * 2, 72, 72, 72];
                }
                0x01 => {
                    self.slots[slot0].mod_idx = slot0 * 2 + 1;
                    self.slots[slot1].mod_idx = 72;
                    self.channels[channel].out = [slot0 * 2, slot1 * 2, 72, 72];
                }
                _ => {}
            }
        }
    }
}

fn decoded_slot(high: usize, regm: u8) -> Option<usize> {
    let decoded = AD_SLOT[(regm & 0x1f) as usize];
    (decoded >= 0).then_some(18 * high + decoded as usize)
}

fn clip_sample(sample: i32) -> i16 {
    sample.clamp(-32768, 32767) as i16
}

fn masked_accm(accm: i16, mask: u16) -> i16 {
    ((accm as i32) & mask as i32) as i16
}

fn pan_from_channel_mask(mask: u16) -> i32 {
    ((mask as u32) << 16) as i32
}

fn envelope_calc_exp(mut level: u32) -> i16 {
    if level > 0x1fff {
        level = 0x1fff;
    }
    ((EXPROM[(level & 0xff) as usize] as u32) >> (level >> 8)) as i16
}

// Unified waveform LUT: for each (waveform, phase & 0x3ff) it stores a 16-bit
// entry whose top bit is the sign-flip mask and whose low 15 bits are the
// logsin-domain addend that gets fed into envelope_calc_exp. This collapses the
// eight per-waveform branches plus their internal sub-branches into a single
// indexed load.
const WF_ROM: [u16; 8 * 1024] = build_wf_rom();

const fn build_wf_rom() -> [u16; 8 * 1024] {
    let mut table = [0u16; 8 * 1024];
    let mut wf: usize = 0;
    while wf < 8 {
        let mut phase: u16 = 0;
        while phase < 1024 {
            let (out, neg) = wf_entry(wf as u8, phase);
            table[wf * 1024 + phase as usize] = ((neg as u16) << 15) | (out & 0x7fff);
            phase += 1;
        }
        wf += 1;
    }
    table
}

const fn wf_entry(wf: u8, phase: u16) -> (u16, bool) {
    let phase = phase & 0x3ff;
    match wf {
        0 => {
            let neg = (phase & 0x200) != 0;
            let out = if (phase & 0x100) != 0 {
                LOGSINROM[((phase & 0xff) ^ 0xff) as usize]
            } else {
                LOGSINROM[(phase & 0xff) as usize]
            };
            (out, neg)
        }
        1 => {
            let out = if (phase & 0x200) != 0 {
                0x1000
            } else if (phase & 0x100) != 0 {
                LOGSINROM[((phase & 0xff) ^ 0xff) as usize]
            } else {
                LOGSINROM[(phase & 0xff) as usize]
            };
            (out, false)
        }
        2 => {
            let out = if (phase & 0x100) != 0 {
                LOGSINROM[((phase & 0xff) ^ 0xff) as usize]
            } else {
                LOGSINROM[(phase & 0xff) as usize]
            };
            (out, false)
        }
        3 => {
            let out = if (phase & 0x100) != 0 {
                0x1000
            } else {
                LOGSINROM[(phase & 0xff) as usize]
            };
            (out, false)
        }
        4 => {
            let neg = (phase & 0x300) == 0x100;
            let out = if (phase & 0x200) != 0 {
                0x1000
            } else if (phase & 0x80) != 0 {
                LOGSINROM[(((phase ^ 0xff) << 1) & 0xff) as usize]
            } else {
                LOGSINROM[((phase << 1) & 0xff) as usize]
            };
            (out, neg)
        }
        5 => {
            let out = if (phase & 0x200) != 0 {
                0x1000
            } else if (phase & 0x80) != 0 {
                LOGSINROM[(((phase ^ 0xff) << 1) & 0xff) as usize]
            } else {
                LOGSINROM[((phase << 1) & 0xff) as usize]
            };
            (out, false)
        }
        6 => {
            let neg = (phase & 0x200) != 0;
            (0, neg)
        }
        _ => {
            // wf = 7: triangle. Sign flips on the upper half, and the phase is
            // mirrored into 0..=0x1ff before the << 3.
            let neg = (phase & 0x200) != 0;
            let inner = if neg { (phase & 0x1ff) ^ 0x1ff } else { phase };
            (inner << 3, neg)
        }
    }
}

fn envelope_calc_sin(waveform: u8, phase: u16, envelope: u16) -> i16 {
    let idx = (((waveform & 0x07) as usize) << 10) | ((phase & 0x3ff) as usize);
    let entry = WF_ROM[idx];
    let out = (entry & 0x7fff) as u32;
    let neg = if (entry & 0x8000) != 0 { 0xffffu16 } else { 0 };
    (envelope_calc_exp(out + ((envelope as u32) << 3)) as u16 ^ neg) as i16
}
