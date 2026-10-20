//! Entirely invented bytes; no original game files needed by these tests.
use lemmings_resources::{
    GROUND_RECORD_SIZE, LEMMING_SPRITE_DATA_SIZE, LEVEL_RECORD_SIZE, MAIN_MASK_RECORD_SIZE,
    MainUiSection,
};
use std::collections::BTreeMap;

// Literal-only DAT writer, independently exercising the production decoder.
fn packed(bytes: &[u8]) -> Vec<u8> {
    let bits: String = bytes
        .rchunks(264)
        .map(|chunk| {
            let mut bits = if chunk.len() <= 8 {
                format!("00{:03b}", chunk.len() - 1)
            } else {
                format!("111{:08b}", chunk.len() - 9)
            };
            for byte in chunk.iter().rev() {
                bits.push_str(&format!("{byte:08b}"));
            }
            bits
        })
        .collect();
    let payload: Vec<u8> = bits
        .as_bytes()
        .rchunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .enumerate()
                .fold(0, |v, (i, b)| v | ((b - b'0') << i))
        })
        .collect();
    let mut out = vec![
        ((bits.len() - 1) % 8 + 1) as u8,
        payload.iter().fold(0, |a, b| a ^ b),
        0,
        0,
    ];
    out.extend(u16::try_from(bytes.len()).unwrap().to_be_bytes());
    out.extend([0, 0]);
    out.extend(u16::try_from(payload.len() + 10).unwrap().to_be_bytes());
    out.extend(payload);
    out
}

pub fn bank(special: bool) -> BTreeMap<String, Vec<u8>> {
    let mut level = vec![0; LEVEL_RECORD_SIZE];
    for (i, word) in [50u16, 1, 1, 1, 1, 2, 3, 4, 5, 6, 7, 8]
        .into_iter()
        .enumerate()
    {
        level[2 * i..2 * i + 2].copy_from_slice(&word.to_be_bytes());
    }
    level[29] = u8::from(special);
    level[32..40].copy_from_slice(&[0, 64, 0, 40, 0, 1, 0, 0]);
    level[288..1888].fill(255);
    level[288..292].copy_from_slice(&[0, 64, 52, 0]); // own 8-pixel ground at (48,100)
    level[2016..].fill(b' ');
    level[2016..2025].copy_from_slice(b"BYTE BANK");
    let dat = packed(&level).repeat(8);
    let mut ground = vec![0; GROUND_RECORD_SIZE];
    ground[28 + 3..28 + 10].copy_from_slice(&[1, 8, 1, 5, 0, 4, 0]);
    ground[448..454].copy_from_slice(&[8, 1, 0, 0, 3, 0]);
    ground[984..987].copy_from_slice(&[63, 10, 2]);
    let mut graphics = packed(&[255, 0, 0, 255]);
    graphics.extend(packed(&[255, 0, 0, 0, 255]));
    let mut main = Vec::new();
    for size in [
        LEMMING_SPRITE_DATA_SIZE,
        MAIN_MASK_RECORD_SIZE,
        MainUiSection::PanelDigits.decoded_size(),
        1,
        1,
        1,
        1,
    ] {
        main.extend(packed(&vec![0; size]));
    }
    let mut odd = vec![0; 80 * 56];
    for record in odd.chunks_exact_mut(56) {
        record[..24].copy_from_slice(&level[..24]);
        record[24..].copy_from_slice(&level[2016..]);
    }
    BTreeMap::from([
        ("LEVEL009.DAT".into(), dat.clone()),
        ("LEVEL000.DAT".into(), dat),
        ("ODDTABLE.DAT".into(), odd),
        ("GROUND0O.DAT".into(), ground),
        ("VGAGR0.DAT".into(), graphics),
        ("MAIN.DAT".into(), main),
    ])
}
