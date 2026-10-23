//! Bounded atlas catalog; paths and I/O are owned by the platform adapter.
use super::assets::{AssetError, crc32};
use crate::Assets;
const MAX_BYTES: usize = 128 * 1024 * 1024;
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    pub index: usize,
    pub bytes: usize,
    pub checksum: u32,
    pub scope: u16,
}
#[derive(Debug)]
pub struct Catalog {
    entries: Vec<Entry>,
}
pub fn level_scope(assets: &Assets) -> Result<u16, Box<dyn std::error::Error>> {
    let mut scope = 3;
    if let Some((set, level, _, _)) = assets.hd_art()? {
        if set > 4 {
            return Err("unsupported HD theme".into());
        }
        scope = 1 | (1u16 << (set + 1));
        if let Some(special) = level.header().special_graphic_set() {
            if special >= 4 {
                return Err("unsupported HD special scene".into());
            }
            scope |= 1u16 << (special + 6);
        }
    }
    Ok(scope)
}
pub fn valid_scope(scope: u16) -> bool {
    scope & 1 != 0
        && scope & !1023 == 0
        && (scope & 62).count_ones() == 1
        && (scope & 960).count_ones() <= 1
}
impl Catalog {
    pub fn parse(data: &[u8]) -> Result<Self, AssetError> {
        if data.len() < 12 {
            return Err(AssetError::Truncated);
        }
        if &data[..8] != b"LHDIDX01" {
            return Err(AssetError::Magic);
        }
        let count = u16::from_le_bytes([data[8], data[9]]) as usize;
        if !(1..=128).contains(&count) || data[10..12] != [0, 0] || data.len() != 12 + count * 12 {
            return Err(AssetError::Limit);
        }
        let mut entries = Vec::new();
        let mut total = 0usize;
        for (index, row) in data[12..].chunks_exact(12).enumerate() {
            let scope = u16::from_le_bytes([row[0], row[1]]);
            let bytes = u32::from_le_bytes(row[4..8].try_into().expect("four bytes")) as usize;
            let checksum = u32::from_le_bytes(row[8..12].try_into().expect("four bytes"));
            if scope == 0
                || scope & !1023 != 0
                || row[2..4] != [0, 0]
                || !(64..=MAX_BYTES).contains(&bytes)
            {
                return Err(AssetError::Limit);
            }
            total += bytes;
            if total > 512 * 1024 * 1024 {
                return Err(AssetError::Limit);
            }
            entries.push(Entry {
                index,
                bytes,
                checksum,
                scope,
            });
        }
        Ok(Self { entries })
    }
    pub fn select(&self, scope: u16) -> Result<Vec<Entry>, AssetError> {
        if !valid_scope(scope) {
            return Err(AssetError::Key);
        }
        let selected: Vec<_> = self
            .entries
            .iter()
            .filter(|e| e.scope & scope != 0)
            .copied()
            .collect();
        if !(1..=32).contains(&selected.len())
            || 12 + selected.iter().map(|e| 4 + e.bytes).sum::<usize>() > MAX_BYTES
        {
            return Err(AssetError::Limit);
        }
        Ok(selected)
    }
}
impl Entry {
    pub fn verify(&self, data: &[u8]) -> Result<(), AssetError> {
        if data.len() != self.bytes {
            return Err(AssetError::Size);
        }
        if crc32(data) != self.checksum {
            return Err(AssetError::Checksum);
        }
        Ok(())
    }
}
