use lemmings_game::hd::{
    assets::AssetError,
    catalog::{Catalog, valid_scope},
};
fn catalog(rows: &[(u16, u32, u32)]) -> Vec<u8> {
    let mut b = b"LHDIDX01".to_vec();
    b.extend((rows.len() as u16).to_le_bytes());
    b.extend([0, 0]);
    for &(mask, size, crc) in rows {
        b.extend(mask.to_le_bytes());
        b.extend([0, 0]);
        b.extend(size.to_le_bytes());
        b.extend(crc.to_le_bytes());
    }
    b
}
#[test]
fn selection_includes_common_theme_and_optional_special_only() {
    let b = catalog(&[(1, 64, 0), (2, 64, 0), (4, 64, 0), (64, 64, 0)]);
    let c = Catalog::parse(&b).unwrap();
    assert_eq!(
        c.select(3)
            .unwrap()
            .iter()
            .map(|e| e.index)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        c.select(69)
            .unwrap()
            .iter()
            .map(|e| e.index)
            .collect::<Vec<_>>(),
        vec![0, 2, 3]
    );
    for scope in [0, 1, 7, 1024, 195] {
        assert!(!valid_scope(scope));
        assert!(c.select(scope).is_err());
    }
}
#[test]
fn catalog_rejects_truncation_reserved_bits_and_unbounded_totals() {
    let b = catalog(&[(1, 64, 0)]);
    for n in 0..b.len() {
        assert!(Catalog::parse(&b[..n]).is_err());
    }
    for (offset, byte) in [(0, b'X'), (8, 0), (10, 1), (12, 0), (14, 1), (16, 1)] {
        let mut bad = b.clone();
        bad[offset] = byte;
        assert!(Catalog::parse(&bad).is_err());
    }
    assert!(Catalog::parse(&catalog(&[(1, 128 * 1024 * 1024, 0); 5])).is_err());
    let c = Catalog::parse(&catalog(&[(1, 128 * 1024 * 1024, 0)])).unwrap();
    assert!(c.select(3).is_err());
    let c = Catalog::parse(&catalog(&[(1, 64, 0); 33])).unwrap();
    assert!(c.select(3).is_err());
}
#[test]
fn selected_file_size_and_crc_are_checked_before_use() {
    let c = Catalog::parse(&catalog(&[(1, 64, 0)])).unwrap();
    let e = c.select(3).unwrap()[0];
    assert_eq!(e.verify(&[]), Err(AssetError::Size));
    assert_eq!(e.verify(&[0; 64]), Err(AssetError::Checksum));
}
