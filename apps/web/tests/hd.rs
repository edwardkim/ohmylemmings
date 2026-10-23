use lemmings_web::WebGame;
fn pack() -> Vec<u8> {
    let mut b = b"LHDPAK01".to_vec();
    b.extend(1u16.to_le_bytes());
    b.extend(1u16.to_le_bytes());
    // One opaque black RGBA pixel. CRC32(0,0,0,255)=0x0c463091.
    let rgba = [0, 0, 0, 255];
    let mut crc = !0u32;
    for byte in rgba {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb88320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    for n in [1u32, 1, 4, !crc, 0] {
        b.extend(n.to_le_bytes());
    }
    b.extend(rgba);
    b.extend([1, 0]);
    for n in [0u16, 0, 0] {
        b.extend(n.to_le_bytes());
    }
    for n in [0u32, 0, 1, 1] {
        b.extend(n.to_le_bytes());
    }
    for n in [256u16, 256] {
        b.extend(n.to_le_bytes());
    }
    b.extend([0; 4]);
    b
}
#[test]
fn hd_loading_rendering_failure_and_restart_preserve_the_game_contract() {
    let mut hd = WebGame::new("builder", "reference").unwrap();
    let mut plain = WebGame::new("builder", "reference").unwrap();
    let before = hd.state();
    hd.load_hd(&pack()).unwrap();
    assert_eq!(hd.state(), before);
    for _ in 0..90 {
        hd.hd_commands().unwrap();
        assert_eq!(hd.state(), plain.state());
        assert_eq!(hd.sound_state(), plain.sound_state());
        hd.step(1.).unwrap();
        plain.step(1.).unwrap();
    }
    let info = hd.hd_info();
    let state = hd.state();
    assert!(hd.load_hd(b"bad").is_err());
    assert_eq!(hd.hd_info(), info);
    assert_eq!(hd.state(), state);
    hd.restart_level().unwrap();
    hd.hd_commands().unwrap();
    assert_eq!(hd.state(), before);
    hd.reset("win", "reference").unwrap();
    assert!(!hd.hd_commands().unwrap().is_empty());
}
#[test]
fn hd_pause_nuke_minimap_and_invalid_pointer_reuse_existing_input_rules() {
    let mut hd = WebGame::new("builder", "reference").unwrap();
    let mut plain = WebGame::new("builder", "reference").unwrap();
    hd.hd_pointer("down", 1430., 1000., 0., 1., 1.).unwrap();
    plain.key("Space", false).unwrap();
    assert_eq!(hd.state(), plain.state());
    hd.hd_pointer("down", 1430., 1000., 0., 1., 1.).unwrap();
    plain.key("Space", false).unwrap();
    let single = hd.state();
    hd.hd_pointer("down", 1500., 1000., 0., 1., 1.).unwrap();
    assert_eq!(hd.state(), single);
    hd.hd_pointer("down", 1500., 1000., 0., 1., 2.).unwrap();
    plain.key("KeyN", false).unwrap();
    assert_eq!(hd.state(), plain.state());
    let before = (hd.state(), hd.view_state());
    for (x, y, button, buttons, clicks) in [
        (f64::NAN, 20., 0., 1., 1.),
        (20., 20., 3., 1., 1.),
        (20., 20., 0., f64::NAN, 1.),
    ] {
        assert!(
            hd.hd_pointer("down", x, y, button, buttons, clicks)
                .is_err()
        );
        assert_eq!((hd.state(), hd.view_state()), before);
    }
    assert_eq!(
        hd.hd_pointer("down", 1600., 1000., 0., 1., 1.).unwrap(),
        "restart-level"
    );
    hd.hd_pointer("down", 1750., 1000., 0., 1., 1.).unwrap();
    let moved = hd.view_state();
    assert!(moved.contains("\"dragging\":true"));
    hd.hd_pointer("up", -1., -1., 0., 0., 0.).unwrap();
    assert!(hd.view_state().contains("\"dragging\":false"));
}
