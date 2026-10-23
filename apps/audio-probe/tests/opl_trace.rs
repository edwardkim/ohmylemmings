use lemmings_audio_probe::opl_pcm;
fn wire(rate: u32, frames: u32, events: &[(u32, u8, u8)]) -> Vec<u8> {
    let mut data = Vec::new();
    for value in [rate, frames, events.len() as u32] {
        data.extend(value.to_le_bytes());
    }
    for &(frame, reg, value) in events {
        data.extend(frame.to_le_bytes());
        data.extend([reg, value]);
    }
    data
}
#[test]
fn malformed_trace_cannot_allocate_unbounded_pcm_or_reorder_registers() {
    let good = wire(44100, 2, &[(0, 0x20, 1)]);
    for end in 0..good.len() {
        assert!(opl_pcm(&good[..end]).is_err());
    }
    assert_eq!(opl_pcm(&wire(44100, 1, &[])).unwrap(), [0]);
    for bad in [
        wire(0, 1, &[]),
        wire(44100, 16_000_001, &[]),
        wire(44100, 2, &[(2, 0, 0)]),
        wire(44100, 2, &[(1, 0, 0), (0, 0, 0)]),
    ] {
        assert!(opl_pcm(&bad).is_err());
    }
    let mut bad = good.clone();
    bad[8..12].copy_from_slice(&1_000_001u32.to_le_bytes());
    assert!(opl_pcm(&bad).is_err());
    let mut bad = good;
    bad.push(0);
    assert!(opl_pcm(&bad).is_err());
}
