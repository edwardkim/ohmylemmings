use lemmings_game::hd::download::requirements;
use lemmings_resources::LevelHeader;

fn header() -> LevelHeader {
    LevelHeader {
        release_rate: 50,
        total: 10,
        required: 5,
        minutes: 5,
        skills: [0; 8],
        screen_x: 0,
        graphics: 2,
        special: 0,
        superlemming: false,
    }
}

#[test]
fn digger_level_needs_its_actions_and_objects_without_other_skills_or_themes() {
    let mut h = header();
    h.skills[7] = 10;
    let keys = requirements(h, &[(0, 1), (1, 0), (6, 0)]).unwrap();
    for slot in [0, 1, 2, 3, 4, 16, 17, 20, 21, 26, 27] {
        assert!(keys.contains(&[0, slot]), "missing reachable action {slot}");
    }
    for slot in [
        5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 18, 19, 22, 23, 24, 25,
    ] {
        assert!(!keys.contains(&[0, slot]), "unused action {slot}");
    }
    assert!(keys.contains(&[1, 2]));
    assert!(!keys.contains(&[1, 0]));
    for id in [0, 1, 6] {
        assert!(keys.contains(&[2, 32 + id]));
    }
    assert!(!keys.contains(&[2, 37]));
    assert_eq!(keys.iter().filter(|k| k[0] == 3).count(), 13);
    assert!(!keys.iter().any(|k| k[0] == 4));
}

#[test]
fn every_skill_and_hazard_adds_all_possible_actions_including_nuke_without_bombers() {
    let mut h = header();
    h.skills = [1; 8];
    h.special = 4;
    let keys = requirements(h, &[(5, 5), (7, 6)]).unwrap();
    for slot in 0..28 {
        assert!(keys.contains(&[0, slot]));
    }
    assert!(keys.contains(&[4, 3]));
    let keys = requirements(header(), &[]).unwrap();
    assert!(keys.contains(&[0, 26]) && keys.contains(&[0, 27]));
}

#[test]
fn invalid_selectors_are_rejected_and_repeated_objects_do_not_duplicate_keys() {
    for (graphics, special) in [(5, 0), (0, 5)] {
        let mut h = header();
        h.graphics = graphics;
        h.special = special;
        assert!(requirements(h, &[]).is_err());
    }
    assert!(requirements(header(), &[(16, 0)]).is_err());
    let keys = requirements(header(), &[(0, 1), (0, 1)]).unwrap();
    assert_eq!(keys.iter().filter(|&&k| k == [2, 32]).count(), 1);
}
