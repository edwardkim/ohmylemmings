use lemmings_resources::{AdlibImage, AdlibPlayer, AdlibStreamError, AdlibWrite};
fn fixture() -> Vec<u8> {
    let mut b = vec![0; 22125];
    fn word(b: &mut [u8], p: usize, v: usize) {
        b[p..p + 2].copy_from_slice(&(v as u16).to_le_bytes());
    }
    for i in 0..21 {
        word(&mut b, 2926 + i * 2, 2968);
    }
    word(&mut b, 2968, 0x4300);
    word(&mut b, 2970, 3200 - 2926);
    b[2972] = 2;
    b[2973] = 1;
    word(&mut b, 2974, 3000 - 2926);
    word(&mut b, 3000, 3100 - 2926);
    word(&mut b, 3002, 0);
    word(&mut b, 3004, 3000 - 2926);
    b[3100..3105].copy_from_slice(&[0xc1, 0xe2, 10, 0x80, 0x85]);
    for i in 0..18 {
        word(&mut b, 21989 + i * 2, 22025);
    }
    b[22025..22029].copy_from_slice(&[0xc1, 0xe0, 10, 0x85]);
    b[3200..3216].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 0, 9, 10, 11, 0xc3, 0, 1, 0]);
    let instrument = b[3200..3216].to_vec();
    b[21731..21747].copy_from_slice(&instrument);
    for i in 0..128 {
        b[2215 + i] = (i & 63) as u8;
    }
    b[2727..2823].fill(2);
    b[2823..2919].fill(0);
    word(&mut b, 2343, 0x8123);
    b
}
fn writes() -> [AdlibWrite; 256] {
    [AdlibWrite::default(); 256]
}
#[test]
fn note_stream_has_original_register_order_and_music_divider() {
    let b = fixture();
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    p.request_music(0).unwrap();
    let mut out = writes();
    let n = p.step(&mut out, 128).unwrap();
    let pairs: Vec<_> = out[..n].iter().map(|w| (w.register, w.value)).collect();
    assert_eq!(
        &pairs[..13],
        &[
            (0xb0, 0),
            (0xb1, 0),
            (0xb2, 0),
            (0xb3, 0),
            (0xb4, 0),
            (0xb5, 0),
            (0xb6, 0),
            (0xb7, 0),
            (0xb8, 0),
            (1, 0x20),
            (0xbd, 0xc0),
            (8, 0),
            (4, 0x21)
        ]
    );
    assert_eq!(
        &pairs[13..],
        &[
            (0x60, 1),
            (0x63, 2),
            (0x80, 3),
            (0x83, 4),
            (0xe0, 7),
            (0xe3, 8),
            (0xc0, 9),
            (0x20, 5),
            (0x23, 6),
            (0x40, 0x0a),
            (0x43, 0xd5),
            (0xb0, 0),
            (0xa0, 0x23),
            (0xb0, 0x29)
        ]
    );
    assert!(out[..n].iter().all(|w| w.audio_step == 0));
    assert_eq!(p.state().channels[0][1], 3);
    assert_eq!(p.state().cycle, 2);
    assert_eq!(p.step(&mut out, 128).unwrap(), 0);
    assert_eq!(p.state().channels[0][1], 3);
    assert_eq!(p.step(&mut out, 128).unwrap(), 0);
    assert_eq!(p.state().channels[0][1], 2);
}
#[test]
fn failure_preserves_state_and_output_for_budget_and_capacity() {
    let b = fixture();
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    p.request_music(0).unwrap();
    let state = p.state();
    let sentinel = AdlibWrite {
        audio_step: 99,
        register: 99,
        value: 99,
    };
    let mut out = [sentinel; 256];
    assert!(matches!(p.step(&mut out, 0), Err(AdlibStreamError::Budget)));
    assert_eq!(p.state(), state);
    assert_eq!(out, [sentinel; 256]);
    assert!(matches!(
        p.step(&mut out[..1], 128),
        Err(AdlibStreamError::Capacity { .. })
    ));
    assert_eq!(p.state(), state);
    assert_eq!(out, [sentinel; 256]);
}
#[test]
fn effects_use_four_slots_bypass_music_wait_and_queue_when_full() {
    let mut b = fixture();
    b[22026] = 0xff;
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    let mut out = writes();
    for slot in (5..9).rev() {
        p.request_effect(0).unwrap();
        p.step(&mut out, 128).unwrap();
        assert_eq!(p.state().channels[slot][16], 2);
    }
    // All four slots are occupied, so the next request remains queued.
    p.request_effect(1).unwrap();
    p.step(&mut out, 128).unwrap();
    assert_eq!(p.state().pending_effect, 2);
}
#[test]
fn invalid_indices_do_not_mutate_requests() {
    let b = fixture();
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    let before = p.state();
    assert!(p.request_music(21).is_err());
    assert!(p.request_effect(18).is_err());
    assert_eq!(p.state(), before);
}
#[test]
fn initial_rest_without_envelope_can_wait_for_instrument() {
    let mut b = fixture();
    b[3100..3107].copy_from_slice(&[0xe2, 0x81, 0xc1, 10, 0x85, 0x85, 0x85]);
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    p.request_music(0).unwrap();
    let mut out = writes();
    for _ in 0..8 {
        p.step(&mut out, 128).unwrap();
    }
    assert_ne!(p.state().channels[0][2], 0);
}
#[test]
fn effect_frequency_uses_wrapping_driver_table_addresses() {
    let mut b = fixture();
    b[21739] = 246;
    b[22027] = 0; // Original effect 14 also crosses nominal note tables.
    b[2727 + 250] = 0xff;
    b[2823 + 250] = 0x1f;
    b[2343 + 15 * 32..2345 + 15 * 32].copy_from_slice(&0x0123_u16.to_le_bytes());
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    p.request_effect(0).unwrap();
    let mut out = writes();
    let n = p.step(&mut out, 128).unwrap();
    assert!(
        out[..n]
            .iter()
            .any(|w| w.register == 0xa8 && w.value == 0x91)
    );
}
#[test]
fn every_control_opcode_and_program_loop_is_repeatable() {
    let mut b = fixture();
    b[3100..3115].copy_from_slice(&[
        0xc1, 0xe1, 0x84, 0xfe, 0x86, 10, 0x82, 0x87, 12, 0x81, 0xb0, 127, 0x80, 0x85, 0x85,
    ]);
    let mut a = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    a.request_music(0).unwrap();
    let mut c = a;
    let mut x = writes();
    let mut y = writes();
    let mut trace = Vec::new();
    for _ in 0..80 {
        let n = a.step(&mut x, 128).unwrap();
        trace.extend_from_slice(&x[..n]);
    }
    let mut partitioned = Vec::new();
    for length in [1, 17, 3, 59] {
        for _ in 0..length {
            let n = c.step(&mut y, 128).unwrap();
            partitioned.extend_from_slice(&y[..n]);
        }
    }
    assert_eq!(trace, partitioned);
    assert_eq!(a.state(), c.state());
    assert!(a.state().channels[0][16] != 0);
}
#[test]
fn stop_restart_clears_active_channels_and_queues_without_resetting_clock() {
    let b = fixture();
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    let mut out = writes();
    p.request_music(0).unwrap();
    p.request_effect(0).unwrap();
    p.step(&mut out, 128).unwrap();
    p.request_effect(1).unwrap();
    p.request_stop();
    let n = p.step(&mut out, 128).unwrap();
    assert_eq!(n, 28);
    assert!(p.state().channels.iter().all(|c| c[16] == 0));
    assert_eq!(p.state().pending_effect, 0);
    assert!(out[..n].iter().all(|w| w.audio_step == 1));
    p.request_music(0).unwrap();
    p.step(&mut out, 128).unwrap();
    assert_eq!(p.state().audio_step, 3);
    assert_eq!(p.state().channels[0][16], 1);
}
#[test]
fn song_end_stops_music_while_effect_end_stops_only_its_slot() {
    let mut b = fixture();
    b[3100] = 0x83;
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    p.request_music(0).unwrap();
    p.request_effect(0).unwrap();
    let mut out = writes();
    p.step(&mut out, 128).unwrap();
    assert_eq!(p.state().channels[0][16], 0);
    assert_eq!(p.state().playing_music, 0);
    for _ in 0..3 {
        p.step(&mut out, 128).unwrap();
    }
    assert_eq!(p.state().channels[8][16], 0);
}
#[test]
fn malformed_opcode_instrument_and_program_are_transactional() {
    for bytes in [&[0x88][..], &[0xc0], &[0xc1, 0xb0], &[0x80]] {
        let mut b = fixture();
        b[3100..3100 + bytes.len()].copy_from_slice(bytes);
        if bytes == [0xc1, 0xb0] {
            b[3102..21731].fill(0xe0);
        }
        if bytes == [0x80] {
            b[3002..3004].copy_from_slice(&65535_u16.to_le_bytes());
        }
        let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
        p.request_music(0).unwrap();
        let before = p.state();
        let mut out = writes();
        let unchanged = out;
        assert!(p.step(&mut out, 128).is_err());
        assert_eq!(p.state(), before);
        assert_eq!(out, unchanged);
    }
}
#[test]
fn command_only_program_loops_are_bounded_even_with_unlimited_caller_budget() {
    let mut b = fixture();
    b[3100] = 0x80;
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    p.request_music(0).unwrap();
    let before = p.state();
    let mut out = writes();
    assert_eq!(p.step(&mut out, usize::MAX), Err(AdlibStreamError::Budget));
    assert_eq!(p.state(), before);
}
#[test]
fn zero_music_divider_wraps_to_256_driver_calls() {
    let mut b = fixture();
    b[2972] = 0;
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    p.request_music(0).unwrap();
    let mut out = writes();
    p.step(&mut out, 128).unwrap();
    assert_eq!(p.state().channels[0][1], 3);
    for _ in 0..255 {
        p.step(&mut out, 128).unwrap();
    }
    assert_eq!(p.state().channels[0][1], 3);
    p.step(&mut out, 128).unwrap();
    assert_eq!(p.state().channels[0][1], 2);
}
#[test]
fn malformed_stream_bytes_never_panic_and_errors_preserve_every_field() {
    for at in [
        3002, 3003, 3100, 3101, 3102, 3103, 3104, 3214, 22025, 22026, 22027,
    ] {
        for value in 0..=255 {
            let mut b = fixture();
            b[at] = value;
            let Ok(image) = AdlibImage::parse(&b) else {
                continue;
            };
            let mut p = AdlibPlayer::new(image);
            p.request_music(0).unwrap();
            p.request_effect(0).unwrap();
            let mut out = writes();
            for _ in 0..10 {
                let before = p.state();
                let unchanged = out;
                if p.step(&mut out, 128).is_err() {
                    assert_eq!(p.state(), before);
                    assert_eq!(out, unchanged);
                    break;
                }
            }
        }
    }
}
#[test]
fn operands_and_instruments_cannot_cross_music_or_effect_boundaries() {
    for command in [0x84, 0xb0, 0xc1] {
        let mut b = fixture();
        b[3000..3002].copy_from_slice(&(21730_u16 - 2926).to_le_bytes());
        b[21730] = command;
        let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
        p.request_music(0).unwrap();
        let before = p.state();
        let mut out = writes();
        assert!(p.step(&mut out, 128).is_err());
        assert_eq!(p.state(), before);
        assert_eq!(out, writes());
    }
    let mut b = fixture();
    b[2970..2972].copy_from_slice(&(21715_u16 - 2926).to_le_bytes());
    b[3100] = 0xdf;
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    p.request_music(0).unwrap();
    let before = p.state();
    assert!(matches!(
        p.step(&mut writes(), 128),
        Err(AdlibStreamError::Address { .. })
    ));
    assert_eq!(p.state(), before);
    b = fixture();
    b[21989..21991].copy_from_slice(&22124_u16.to_le_bytes());
    b[22124] = 0x84;
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    p.request_effect(0).unwrap();
    let before = p.state();
    assert!(matches!(
        p.step(&mut writes(), 128),
        Err(AdlibStreamError::Address { .. })
    ));
    assert_eq!(p.state(), before);
}
#[test]
fn effect_request_is_released_on_the_call_after_a_slot_ends() {
    let mut b = fixture();
    b[22026] = 0xff;
    let mut p = AdlibPlayer::new(AdlibImage::parse(&b).unwrap());
    let mut out = writes();
    for i in 0..4 {
        p.request_effect(i).unwrap();
        p.step(&mut out, 128).unwrap();
    }
    p.request_effect(17).unwrap();
    for _ in 0..28 {
        p.step(&mut out, 128).unwrap();
    }
    assert_eq!(p.state().pending_effect, 18);
    assert_eq!(p.state().channels[8][16], 0);
    p.step(&mut out, 128).unwrap();
    assert_eq!(p.state().pending_effect, 0);
    assert_eq!(p.state().channels[8][16], 2);
}
