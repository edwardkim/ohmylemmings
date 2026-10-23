use lemmings_game::hd::{
    Control,
    input::{Target, map_point},
    scene::{Draw, Renderer},
};
use lemmings_game::{Assets, LevelRun, pacing::Pacing};

fn run() -> LevelRun {
    LevelRun::new(Assets::synthetic_builder(), 1, Some(2), Pacing::Reference).unwrap()
}

#[test]
fn hd_controls_map_to_existing_panel_semantics_and_map_keeps_camera_precision() {
    assert_eq!(map_point(20., 20., 1584, false), Target::Legacy(3, 3));
    assert_eq!(map_point(410., 1000., 1584, false), Target::Legacy(8, 180));
    assert_eq!(map_point(570., 1000., 1584, false), Target::Legacy(40, 180));
    assert_eq!(map_point(1580., 1000., 1584, false), Target::Restart);
    assert!(matches!(
        map_point(1800., 1000., 1584, false),
        Target::Camera(_)
    ));
    assert_eq!(map_point(1920., 20., 1584, false), Target::Outside);
    assert_eq!(map_point(f64::NAN, 20., 1584, false), Target::Outside);
    assert_eq!(map_point(200., 200., 1584, true), Target::Legacy(160, 100));
    assert_eq!(Control::Minimap.original_button(), None);
}

#[test]
fn renderer_without_hd_art_draws_original_sprites_and_only_rebuilds_changed_terrain() {
    let mut run = run();
    let before = lemmings_game::diagnostics::state(&run.session);
    let sound = run.sound.diagnostic_json();
    let mut renderer = Renderer::fallback();
    renderer.reset(&run.assets).unwrap();
    let first = renderer.frame(&run).unwrap();
    assert!(first.iter().any(|d| matches!(d, Draw::Blit { .. })));
    let rebuilt = renderer.terrain_rebuilds();
    assert!(rebuilt > 0);
    renderer.frame(&run).unwrap();
    assert_eq!(renderer.terrain_rebuilds(), rebuilt);
    assert_eq!(lemmings_game::diagnostics::state(&run.session), before);
    assert_eq!(run.sound.diagnostic_json(), sound);
    for _ in 0..80 {
        run.tick().unwrap();
        renderer.frame(&run).unwrap();
    }
    assert!(
        renderer
            .textures()
            .iter()
            .all(|t| t.rgba.len() == t.width as usize * t.height as usize * 4)
    );
}
