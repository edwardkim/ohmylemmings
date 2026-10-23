use lemmings_game::hd::assets::{AssetError, AssetKey, AssetPack};

fn crc(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & (0u32.wrapping_sub(crc & 1)));
        }
    }
    !crc
}

fn fixture() -> Vec<u8> {
    let mut b = b"LHDPAK01".to_vec();
    b.extend(1u16.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    let rgba = [10u8, 20, 30, 0, 40, 50, 60, 255];
    for n in [2u32, 1, 8, crc(&rgba), 0] {
        b.extend(n.to_le_bytes());
    }
    b.extend(rgba);
    b.extend([0, 0]); // actor, slot0
    for n in [0u16, 0, 0] {
        b.extend(n.to_le_bytes());
    } // frame, atlas, reserved
    for n in [0u32, 0, 2, 1] {
        b.extend(n.to_le_bytes());
    }
    for n in [96u16, 60] {
        b.extend(n.to_le_bytes());
    }
    for n in [48i16, 60] {
        b.extend(n.to_le_bytes());
    }
    b
}

#[test]
fn rgba_is_borrowed_and_frame_metadata_is_explicit() {
    let b = fixture();
    let pack = AssetPack::parse(&b).unwrap();
    assert_eq!(pack.atlases().len(), 1);
    let atlas = &pack.atlases()[0];
    assert_eq!(atlas.rgba().as_ptr(), b[32..].as_ptr());
    assert_eq!((atlas.width(), atlas.height()), (2, 1));
    let frame = pack.frame(AssetKey::Actor { slot: 0, frame: 0 }).unwrap();
    assert_eq!(frame.rect, [0, 0, 2, 1]);
    assert_eq!(frame.display, [96, 60]);
    assert_eq!(frame.anchor, [48, 60]);
    assert_eq!(
        atlas.row(frame, 0).unwrap(),
        &[10, 20, 30, 0, 40, 50, 60, 255]
    );
    assert!(atlas.row(frame, 1).is_none());
    assert!(pack.frame(AssetKey::Actor { slot: 2, frame: 0 }).is_none());
}

#[test]
fn every_truncation_and_trailing_data_is_rejected() {
    let b = fixture();
    for end in 0..b.len() {
        assert!(AssetPack::parse(&b[..end]).is_err(), "{end}");
    }
    let mut b = b;
    b.push(0);
    assert_eq!(AssetPack::parse(&b).unwrap_err(), AssetError::TrailingBytes);
}

#[test]
fn versions_counts_sizes_and_pixel_checksum_are_validated() {
    for (index, bytes) in [
        (7, vec![b'2']),
        (8, vec![0, 0]),
        (8, vec![17, 0]),
        (10, vec![0, 0]),
        (12, vec![0, 0, 0, 0]),
        (12, 4097u32.to_le_bytes().to_vec()),
        (20, 7u32.to_le_bytes().to_vec()),
        (32, vec![99]),
    ] {
        let mut b = fixture();
        b[index..index + bytes.len()].copy_from_slice(&bytes);
        assert!(AssetPack::parse(&b).is_err(), "{index}");
    }
}

#[test]
fn keys_indices_rectangles_display_and_reserved_bits_are_checked() {
    // Frame starts after the12-byte header and20-byte atlas header plus8 pixels.
    for (offset, bytes) in [
        (0, vec![4]),
        (1, vec![28]),
        (2, 8u16.to_le_bytes().to_vec()),
        (4, 1u16.to_le_bytes().to_vec()),
        (6, 1u16.to_le_bytes().to_vec()),
        (8, u32::MAX.to_le_bytes().to_vec()),
        (16, 0u32.to_le_bytes().to_vec()),
        (20, 2u32.to_le_bytes().to_vec()),
        (24, 0u16.to_le_bytes().to_vec()),
        (28, 5000i16.to_le_bytes().to_vec()),
    ] {
        let mut b = fixture();
        b[40 + offset..40 + offset + bytes.len()].copy_from_slice(&bytes);
        assert!(AssetPack::parse(&b).is_err(), "{offset}");
    }
}

#[test]
fn duplicate_keys_are_rejected_and_trimmed_anchors_are_allowed() {
    let mut b = fixture();
    b[10..12].copy_from_slice(&2u16.to_le_bytes());
    b.extend_from_within(40..72);
    assert_eq!(AssetPack::parse(&b).unwrap_err(), AssetError::DuplicateKey);
    let mut b = fixture();
    b[68..70].copy_from_slice(&(-12i16).to_le_bytes());
    assert_eq!(AssetPack::parse(&b).unwrap().frames()[0].anchor, [-12, 60]);
}

#[test]
fn material_keys_do_not_alias_actor_keys_and_unknown_ids_are_rejected() {
    let mut b = fixture();
    b[40] = 1;
    assert!(
        AssetPack::parse(&b)
            .unwrap()
            .frame(AssetKey::Rock)
            .is_some()
    );
    b[41] = 1;
    assert!(AssetPack::parse(&b).is_err());
}

#[test]
fn row_checks_publicly_constructed_foreign_rectangles_without_panicking() {
    let b = fixture();
    let pack = AssetPack::parse(&b).unwrap();
    let mut frame = pack.frames()[0];
    frame.rect = [u32::MAX, 0, u32::MAX, 1];
    assert!(pack.atlases()[0].row(&frame, 0).is_none());
}

#[test]
fn alpha_cutoff_is_preserved_and_reserved_bits_are_rejected() {
    let mut b = fixture();
    b[28..32].copy_from_slice(&1u32.to_le_bytes());
    let pack = AssetPack::parse(&b).unwrap();
    assert_eq!(pack.atlases()[0].alpha_cutoff(), 1);
    assert_eq!(pack.atlases()[0].rgba(), &b[32..40]);
    b[28..32].copy_from_slice(&255u32.to_le_bytes());
    assert!(AssetPack::parse(&b).is_err());
    b[28..32].copy_from_slice(&256u32.to_le_bytes());
    assert!(AssetPack::parse(&b).is_err());
}

#[test]
fn facility_frames_use_theme_and_id_without_aliasing_actors_or_materials() {
    for (slot, count) in [(0u8, 1u16), (1, 10)] {
        for frame in 0..count {
            let mut b = fixture();
            b[40] = 2;
            b[41] = slot;
            b[42..44].copy_from_slice(&frame.to_le_bytes());
            let pack = AssetPack::parse(&b).unwrap();
            assert!(
                pack.frame(AssetKey::Object {
                    set: 0,
                    id: slot,
                    frame
                })
                .is_some()
            );
            assert!(pack.frame(AssetKey::Actor { slot, frame }).is_none());
        }
        let mut b = fixture();
        b[40] = 2;
        b[41] = slot;
        b[42..44].copy_from_slice(&count.to_le_bytes());
        assert_eq!(AssetPack::parse(&b).unwrap_err(), AssetError::Key);
    }
    for slot in [2, 15, 16, 255] {
        let mut b = fixture();
        b[40] = 2;
        b[41] = slot;
        assert_eq!(AssetPack::parse(&b).unwrap_err(), AssetError::Key);
    }
}

#[test]
fn every_toolbar_icon_resolves_to_the_existing_control_and_minimap_has_no_icon() {
    use lemmings_game::hd::assets::ICON_CONTROLS;
    use lemmings_game::hd::{Control, control_regions};
    let regions = control_regions();
    for (id, control) in ICON_CONTROLS.into_iter().enumerate() {
        assert!(regions.iter().any(|r| r.control == control));
        let mut b = fixture();
        b[40] = 3;
        b[41] = id as u8;
        let pack = AssetPack::parse(&b).unwrap();
        assert!(pack.frame(AssetKey::Icon(control)).is_some());
        assert!(pack.frame(AssetKey::Icon(Control::Minimap)).is_none());
        b[42] = 1;
        assert_eq!(AssetPack::parse(&b).unwrap_err(), AssetError::Key);
    }
    assert_eq!(ICON_CONTROLS.len(), regions.len() - 1);
    for id in [13, 14, 255] {
        let mut b = fixture();
        b[40] = 3;
        b[41] = id;
        assert_eq!(AssetPack::parse(&b).unwrap_err(), AssetError::Key);
    }
}

#[test]
fn every_original_actor_frame_is_required_for_complete_character_coverage() {
    use lemmings_resources::LemmingAnimationInfo;
    let partial = fixture();
    let missing = AssetPack::parse(&partial).unwrap().missing_actor_frames();
    assert_eq!(missing.len(), 336);
    assert!(!missing.contains(&(0, 0)));
    let mut full = fixture();
    full.truncate(40);
    full[10..12].copy_from_slice(&337u16.to_le_bytes());
    for slot in 0..28 {
        for frame in 0..u16::from(LemmingAnimationInfo::for_slot(slot).unwrap().frame_count) {
            let mut record = fixture()[40..].to_vec();
            record[1] = slot as u8;
            record[2..4].copy_from_slice(&frame.to_le_bytes());
            full.extend(record);
        }
    }
    assert!(
        AssetPack::parse(&full)
            .unwrap()
            .missing_actor_frames()
            .is_empty()
    );
}

#[test]
fn expanded_character_pack_accepts_32_small_atlases_and_rejects_33() {
    let original = fixture();
    let mut bytes = original[..12].to_vec();
    bytes[8..10].copy_from_slice(&32u16.to_le_bytes());
    for _ in 0..32 {
        bytes.extend(&original[12..40]);
    }
    bytes.extend(&original[40..]);
    assert_eq!(AssetPack::parse(&bytes).unwrap().atlases().len(), 32);
    bytes[8..10].copy_from_slice(&33u16.to_le_bytes());
    assert!(AssetPack::parse(&bytes).is_err());
}
