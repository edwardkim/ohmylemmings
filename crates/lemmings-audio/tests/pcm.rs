use lemmings_audio::{AdlibPcm, AudioRate, MAX_PCM_FRAMES, PcmError};
use lemmings_resources::{AdlibImage, AdlibWrite};
fn fixture() -> Vec<u8> {
    let mut b = vec![0; 22125];
    fn word(b: &mut [u8], p: usize, v: usize) {
        b[p..p + 2].copy_from_slice(&(v as u16).to_le_bytes());
    }
    for i in 0..21 {
        word(&mut b, 2926 + i * 2, 2968);
    }
    word(&mut b, 2968, 0x4000);
    word(&mut b, 2970, 3200 - 2926);
    b[2972] = 7;
    b[2973] = 1;
    word(&mut b, 2974, 3000 - 2926);
    word(&mut b, 3000, 3100 - 2926);
    word(&mut b, 3002, 0);
    word(&mut b, 3004, 3000 - 2926);
    b[3100..3105].copy_from_slice(&[0xc1, 0xe2, 35, 0x80, 0x85]);
    b[3200..3216].copy_from_slice(&[
        0xf0, 0xf0, 0x0f, 0x0f, 0x21, 0x21, 0, 0, 0, 0, 127, 0, 0, 0, 1, 0,
    ]);
    let inst = b[3200..3216].to_vec();
    b[21731..21747].copy_from_slice(&inst);
    for i in 0..18 {
        word(&mut b, 21989 + i * 2, 22025);
    }
    b[22025..22029].copy_from_slice(&[0xc1, 0xe2, 35, 0x85]);
    b[2215..2343].fill(0);
    b[2727..2823].fill(4);
    b[2823..2919].fill(0);
    word(&mut b, 2343, 0x8298);
    b
}
#[test]
fn rational_clock_carries_every_fraction_and_does_not_use_header_factor() {
    let b = fixture();
    let mut p = AdlibPcm::new(AdlibImage::parse(&b).unwrap(), 44100, AudioRate::DOS_VGA).unwrap();
    p.request_music(0).unwrap();
    let mut scratch = [AdlibWrite::default(); 128];
    let mut out = [0; 4410];
    p.render(&mut out, &mut scratch, 4096).unwrap();
    assert_eq!(p.position().frames, 4410);
    assert_eq!(p.driver_state().audio_step, 8);
    assert!(out.iter().any(|&v| v != 0));
}
#[test]
fn pcm_is_identical_under_arbitrary_output_partitioning() {
    let b = fixture();
    let img = AdlibImage::parse(&b).unwrap();
    let mut a = AdlibPcm::new(img, 48000, AudioRate::DOS_VGA).unwrap();
    let mut c = AdlibPcm::new(img, 48000, AudioRate::DOS_VGA).unwrap();
    a.request_music(0).unwrap();
    c.request_music(0).unwrap();
    let mut scratch = [AdlibWrite::default(); 256];
    let mut x = [0; 4096];
    a.render(&mut x, &mut scratch, 4096).unwrap();
    let mut y = [0; 4096];
    let mut at = 0;
    for size in [1, 127, 512, 3, 2048, 1405] {
        c.render(&mut y[at..at + size], &mut scratch, 4096).unwrap();
        at += size;
    }
    assert_eq!(x, y);
    assert_eq!(a.position(), c.position());
    assert_eq!(a.driver_state(), c.driver_state());
}
#[test]
fn later_stream_error_preserves_pcm_driver_phase_and_synthesizer() {
    let mut b = fixture();
    b[3103] = 0x88;
    let img = AdlibImage::parse(&b).unwrap();
    let mut p = AdlibPcm::new(img, 8000, AudioRate::new(1000, 1).unwrap()).unwrap();
    p.request_music(0).unwrap();
    let before = p.position();
    let driver = p.driver_state();
    let mut out = [123; 512];
    let mut scratch = [AdlibWrite::default(); 128];
    assert!(p.render(&mut out, &mut scratch, 4096).is_err());
    assert_eq!(out, [123; 512]);
    assert_eq!(p.position(), before);
    assert_eq!(p.driver_state(), driver);
    // A successful prefix after the failed request must match a fresh synthesizer.
    let mut reference = AdlibPcm::new(img, 8000, AudioRate::new(1000, 1).unwrap()).unwrap();
    reference.request_music(0).unwrap();
    let mut x = [0; 32];
    let mut y = [0; 32];
    p.render(&mut x, &mut scratch, 4096).unwrap();
    reference.render(&mut y, &mut scratch, 4096).unwrap();
    assert_eq!(x, y);
}
#[test]
fn invalid_rates_indices_capacity_and_output_length_are_atomic() {
    let b = fixture();
    let img = AdlibImage::parse(&b).unwrap();
    for rate in [0, 7999, 192001, u32::MAX] {
        assert!(AdlibPcm::new(img, rate, AudioRate::DOS_VGA).is_err());
    }
    for (n, d) in [(0, 1), (1, 0), (1, 2), (1001, 1), (u32::MAX, 1)] {
        assert!(AudioRate::new(n, d).is_err());
    }
    let mut p = AdlibPcm::new(img, 44100, AudioRate::DOS_VGA).unwrap();
    let before = p.position();
    assert!(p.request_music(21).is_err());
    assert!(p.request_effect(18).is_err());
    assert!(p.reset_music(usize::MAX).is_err());
    assert_eq!(p.position(), before);
    p.request_music(0).unwrap();
    let driver = p.driver_state();
    let mut out = [321; 64];
    assert!(matches!(
        p.render(&mut out, &mut [], 4096),
        Err(PcmError::Scratch { .. })
    ));
    assert_eq!(out, [321; 64]);
    assert_eq!(p.driver_state(), driver);
    assert!(
        p.render(&mut vec![0; MAX_PCM_FRAMES + 1], &mut [], 4096)
            .is_err()
    );
}
#[test]
fn reset_is_a_fresh_generation_and_stop_keeps_the_clock_monotonic() {
    let b = fixture();
    let img = AdlibImage::parse(&b).unwrap();
    let mut a = AdlibPcm::new(img, 44100, AudioRate::DOS_VGA).unwrap();
    let mut scratch = [AdlibWrite::default(); 256];
    a.request_music(0).unwrap();
    let mut initial = [0; 2048];
    a.render(&mut initial, &mut scratch, 4096).unwrap();
    a.request_stop();
    a.render(&mut [0; 1024], &mut scratch, 4096).unwrap();
    assert_eq!(a.position().frames, 3072);
    a.reset_music(0).unwrap();
    assert_eq!(a.position().frames, 0);
    let mut again = [0; 2048];
    a.render(&mut again, &mut scratch, 4096).unwrap();
    assert_eq!(initial, again);
}
#[test]
fn header_factor_does_not_replace_original_irq_clock() {
    let a = fixture();
    let mut b = a.clone();
    b[2968..2970].copy_from_slice(&1_u16.to_le_bytes());
    let mut x = AdlibPcm::new(AdlibImage::parse(&a).unwrap(), 49716, AudioRate::DOS_VGA).unwrap();
    let mut y = AdlibPcm::new(AdlibImage::parse(&b).unwrap(), 49716, AudioRate::DOS_VGA).unwrap();
    x.request_music(0).unwrap();
    y.request_music(0).unwrap();
    let mut left = [0; 8192];
    let mut right = [0; 8192];
    let mut scratch = [AdlibWrite::default(); 128];
    x.render(&mut left, &mut scratch, 4096).unwrap();
    y.render(&mut right, &mut scratch, 4096).unwrap();
    assert_eq!(left, right);
    assert_eq!(x.position(), y.position());
    assert_ne!(
        x.driver_state().sample_rate_factor,
        y.driver_state().sample_rate_factor
    );
}
#[test]
fn queued_control_waits_for_the_next_driver_boundary_and_empty_render_is_inert() {
    let b = fixture();
    let mut p = AdlibPcm::new(AdlibImage::parse(&b).unwrap(), 44100, AudioRate::DOS_VGA).unwrap();
    let mut scratch = [AdlibWrite::default(); 128];
    p.request_music(0).unwrap();
    let before = p.position();
    let driver = p.driver_state();
    p.render(&mut [], &mut [], 0).unwrap();
    assert_eq!(p.position(), before);
    assert_eq!(p.driver_state(), driver);
    p.render(&mut [0; 1], &mut scratch, 4096).unwrap();
    p.request_stop();
    let call = p.driver_state().audio_step;
    let remaining = p.position().remaining_frames as usize;
    p.render(&mut vec![0; remaining], &mut scratch, 0).unwrap();
    assert_eq!(p.driver_state().audio_step, call);
    assert!(p.driver_state().channels.iter().any(|c| c[16] != 0));
    p.render(&mut [0; 1], &mut scratch, 4096).unwrap();
    assert_eq!(p.driver_state().audio_step, call + 1);
    assert!(p.driver_state().channels.iter().all(|c| c[16] == 0));
}
#[test]
fn equivalent_rates_reduce_and_large_valid_terms_do_not_overflow() {
    assert_eq!(
        AudioRate::new(100, 100).unwrap(),
        AudioRate::new(1, 1).unwrap()
    );
    assert_eq!(
        AudioRate::new(u32::MAX, u32::MAX).unwrap(),
        AudioRate::new(1, 1).unwrap()
    );
    let b = fixture();
    let mut p = AdlibPcm::new(
        AdlibImage::parse(&b).unwrap(),
        192000,
        AudioRate::new(u32::MAX, u32::MAX).unwrap(),
    )
    .unwrap();
    p.request_music(0).unwrap();
    p.render(&mut [0; 8192], &mut [AdlibWrite::default(); 128], 4096)
        .unwrap();
    assert_eq!(p.driver_state().audio_step, 1);
}
