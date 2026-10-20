use lemmings_core::{Difficulty, LevelId};
use lemmings_resources::{
    ADLIB_EFFECT_COUNT, ADLIB_IMAGE_SIZE, ADLIB_MUSIC_COUNT, AdlibFile, AdlibImage,
};
fn word(b: &mut [u8], at: usize, v: usize) {
    b[at..at + 2].copy_from_slice(&u16::try_from(v).unwrap().to_le_bytes());
}
fn fixture() -> Vec<u8> {
    let mut b = vec![0; 22125];
    for i in 0..21 {
        let h = 2968 + i * 64;
        word(&mut b, 2926 + i * 2, h);
        word(&mut b, h, 0x1200 + i);
        word(&mut b, h + 2, h + 48 - 2926);
        b[h + 4] = i as u8 % 4;
        let count = if i == 20 { 8 } else { 2 };
        b[h + 5] = count;
        for c in 0..usize::from(count) {
            word(&mut b, h + 6 + c * 2, h + 24 + c * 2 - 2926);
            word(&mut b, h + 24 + c * 2, h + 40 + c - 2926);
            b[h + 40 + c] = c as u8 + 5;
        }
    }
    for i in 0..18 {
        word(&mut b, 21989 + i * 2, 22025 + i * 4);
    }
    word(&mut b, 21989 + 17 * 2, 22124);
    b
}
#[test]
fn all_twenty_one_tracks_and_eighteen_effects_preserve_little_endian_fields() {
    let bytes = fixture();
    let image = AdlibImage::parse(&bytes).unwrap();
    assert_eq!(
        (ADLIB_MUSIC_COUNT, ADLIB_EFFECT_COUNT, ADLIB_IMAGE_SIZE),
        (21, 18, 22125)
    );
    assert_eq!(image.as_bytes().as_ptr(), bytes.as_ptr());
    for i in 0..21 {
        let m = image.music(i).unwrap();
        let h = 2968 + i * 64;
        assert_eq!(
            (
                m.header_offset,
                m.sample_rate_factor,
                m.instrument_offset,
                m.wait_cycles
            ),
            (h, 0x1200 + i as u16, h + 48, i as u8 % 4)
        );
        assert_eq!(m.channels().len(), if i == 20 { 8 } else { 2 });
        for (c, ch) in m.channels().iter().enumerate() {
            assert_eq!(
                (ch.program_offset, ch.first_bar_offset),
                (h + 24 + c * 2, h + 40 + c)
            );
        }
        assert_eq!(image.music(i).unwrap(), m);
    }
    for i in 0..18 {
        assert_eq!(
            image.effect_offset(i).unwrap(),
            if i == 17 { 22124 } else { 22025 + i * 4 }
        );
    }
}
#[test]
fn displayed_level_order_cycles_all_twenty_one_tracks_across_groups() {
    let bytes = fixture();
    let image = AdlibImage::parse(&bytes).unwrap();
    for d in [
        Difficulty::Fun,
        Difficulty::Tricky,
        Difficulty::Taxing,
        Difficulty::Mayhem,
    ] {
        for n in 1..=30 {
            let level = LevelId::new(d, n).unwrap();
            assert_eq!(
                image.music_for_level(level).unwrap(),
                image.music(level.global_index() % 21).unwrap()
            );
        }
    }
    assert_ne!(
        image
            .music_for_level(LevelId::new(Difficulty::Fun, 21).unwrap())
            .unwrap(),
        image.music(0).unwrap()
    );
}
#[test]
fn truncated_extra_images_and_invalid_public_indices_return_errors() {
    let bytes = fixture();
    for n in [0, 1, 2968, 21989, 22124] {
        assert!(AdlibImage::parse(&bytes[..n]).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(AdlibImage::parse(&extra).is_err());
    let image = AdlibImage::parse(&bytes).unwrap();
    for i in [21, usize::MAX] {
        assert!(image.music(i).is_err());
    }
    for i in [18, usize::MAX] {
        assert!(image.effect_offset(i).is_err());
    }
}
#[test]
fn every_track_including_the_last_is_validated_before_exposing_the_image() {
    for i in 0..21 {
        for offset in [0, 2967, 21730, 65535] {
            let mut b = fixture();
            word(&mut b, 2926 + i * 2, offset);
            assert!(AdlibImage::parse(&b).is_err(), "track {i}: {offset}");
        }
    }
}
#[test]
fn zero_sample_factor_and_empty_or_excess_music_channels_are_rejected() {
    for c in [0, 9, 255] {
        let mut b = fixture();
        b[2968 + 5] = c;
        assert!(AdlibImage::parse(&b).is_err());
    }
    let mut b = fixture();
    word(&mut b, 2968, 0);
    assert!(AdlibImage::parse(&b).is_err());
}
#[test]
fn instrument_program_and_initial_bar_pointers_cover_their_first_read() {
    for (at, values) in [
        (2970, vec![0, 21720 - 2926, 65535]),
        (2974, vec![0, 21730 - 2926, 65535]),
        (2992, vec![0, 21731 - 2926, 65535]),
    ] {
        for v in values {
            let mut b = fixture();
            word(&mut b, at, v);
            assert!(AdlibImage::parse(&b).is_err(), "pointer {at}: {v}");
        }
    }
}
#[test]
fn all_effect_addresses_exclude_the_table_and_the_byte_after_the_image() {
    for i in 0..18 {
        for offset in [0, 21989, 22024, 22125, 65535] {
            let mut b = fixture();
            word(&mut b, 21989 + i * 2, offset);
            assert!(AdlibImage::parse(&b).is_err());
        }
    }
}
#[test]
fn dat_wrapper_rejects_missing_extra_damaged_and_wrong_size_sections() {
    assert!(AdlibFile::parse(&[]).is_err());
    let mut b = vec![8, 255, 0, 0, 0, 1, 0, 0, 0, 11, 255];
    assert!(AdlibFile::parse(&b).is_err());
    b[4..6].copy_from_slice(&22125_u16.to_be_bytes());
    let f = AdlibFile::parse(&b).unwrap();
    assert!(f.decode_into(&mut [0; 1]).is_err());
    assert!(f.decode_into(&mut vec![0; 22125]).is_err());
    let mut duplicate = b.clone();
    duplicate.extend_from_slice(&b);
    assert!(AdlibFile::parse(&duplicate).is_err());
    b[1] = 0;
    assert!(AdlibFile::parse(&b).is_err());
}

#[test]
fn channel_order_and_shared_bars_are_preserved_without_per_track_partitioning() {
    let mut bytes = fixture();
    word(&mut bytes, 2974, 2994 - 2926);
    word(&mut bytes, 2976, 2992 - 2926);
    word(&mut bytes, 2994, 21727 - 2926);
    let image = AdlibImage::parse(&bytes).unwrap();
    let music = image.music(0).unwrap();
    assert_eq!(music.channels()[0].program_offset, 2994);
    assert_eq!(music.channels()[1].program_offset, 2992);
    assert_eq!(music.channels()[0].first_bar_offset, 21727);
}
