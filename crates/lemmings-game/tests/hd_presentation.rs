//! HD-02: presentation data and coordinates must not change the logical game.
use lemmings_core::{InputCommand, StartupCommand};
use lemmings_game::{
    Assets, LevelRun,
    hd::{self, Control, Hit, SnapshotError, Viewport},
    lifecycle::ResultAction,
    pacing::Pacing,
    panel::SKILLS,
};

fn run(assets: Assets) -> LevelRun {
    LevelRun::new(assets, 1, Some(2), Pacing::Reference).unwrap()
}

#[test]
fn full_hd_edges_separate_game_panel_and_output_exterior() {
    let viewport = Viewport::fit(1920.0, 1080.0).unwrap();
    assert_eq!(viewport.hit(0.0, 0.0, 640), Some(Hit::Game { x: 0, y: 0 }));
    assert_eq!(
        viewport.hit(1919.999, 959.999, 640),
        Some(Hit::Game { x: 319, y: 159 })
    );
    for point in [(-0.01, 0.0), (1920.0, 0.0), (0.0, 1080.0)] {
        assert_eq!(viewport.hit(point.0, point.1, 640), None);
    }
    assert_eq!(viewport.hit(10.0, 960.0, 640), None); // Status is not a game click.
    assert_eq!(
        viewport.hit(400.0, 960.0, 640),
        Some(Hit::Control(Control::Slower))
    );
}

#[test]
fn resize_letterboxing_and_common_dpi_scales_keep_the_same_logical_point() {
    for (width, height) in [
        (1280.0, 720.0),
        (1366.0, 768.0),
        (1920.0, 1200.0),
        (2560.0, 1080.0),
        (2400.0, 1350.0),
        (2880.0, 1620.0),
        (3840.0, 2160.0),
    ] {
        let viewport = Viewport::fit(width, height).unwrap();
        let rect = viewport.rect();
        let scale = rect.width / 1920.0;
        let x = rect.x + 123.5 * 6.0 * scale;
        let y = rect.y + 87.5 * 6.0 * scale;
        assert_eq!(viewport.hit(x, y, 1600), Some(Hit::Game { x: 123, y: 87 }));
        if rect.x > 0.0 {
            assert_eq!(viewport.hit(rect.x / 2.0, height / 2.0, 1600), None);
        }
        if rect.y > 0.0 {
            assert_eq!(viewport.hit(width / 2.0, rect.y / 2.0, 1600), None);
        }
        assert_eq!(viewport.hit(rect.x + rect.width, rect.y, 1600), None);
        assert_eq!(viewport.hit(rect.x, rect.y + rect.height, 1600), None);
    }
}

#[test]
fn nonfinite_and_empty_sizes_or_input_cannot_emit_a_target() {
    for bad in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(Viewport::fit(bad, 1080.0).is_none());
        assert!(Viewport::fit(1920.0, bad).is_none());
    }
    let viewport = Viewport::fit(1920.0, 1080.0).unwrap();
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(viewport.hit(bad, 20.0, 640), None);
        assert_eq!(viewport.hit(20.0, bad, 640), None);
    }
    assert_eq!(viewport.hit(20.0, 20.0, 0), None);
}

#[test]
fn every_hd_panel_button_has_its_own_action_and_skill_order() {
    let viewport = Viewport::fit(1920.0, 1080.0).unwrap();
    let expected = [
        (400.0, Control::Slower),
        (480.0, Control::Faster),
        (1392.0, Control::Pause),
        (1480.0, Control::Nuke),
        (1568.0, Control::Restart),
    ];
    for (left, action) in expected {
        assert_eq!(viewport.hit(left, 960.0, 1600), Some(Hit::Control(action)));
    }
    for (index, skill) in SKILLS.into_iter().enumerate() {
        let left = 560.0 + index as f64 * 104.0;
        assert_eq!(
            viewport.hit(left, 960.0, 1600),
            Some(Hit::Control(Control::Skill(skill)))
        );
        assert_eq!(
            viewport.hit(left + 103.999, 1079.999, 1600),
            Some(Hit::Control(Control::Skill(skill)))
        );
    }
}

#[test]
fn minimap_content_excludes_its_frame_and_clamps_camera_at_world_ends() {
    let viewport = Viewport::fit(1920.0, 1080.0).unwrap();
    assert_eq!(viewport.hit(1656.0, 1000.0, 1600), None);
    assert_eq!(viewport.hit(1664.0, 975.999, 1600), None);
    assert_eq!(viewport.hit(1912.0, 1000.0, 1600), None);
    assert_eq!(viewport.hit(1800.0, 1064.0, 1600), None);
    assert_eq!(
        viewport.hit(1664.0, 976.0, 1600),
        Some(Hit::Minimap { camera: 0 })
    );
    assert_eq!(
        viewport.hit(1788.0, 1000.0, 1600),
        Some(Hit::Minimap { camera: 640 })
    );
    assert_eq!(
        viewport.hit(1911.999, 1000.0, 1600),
        Some(Hit::Minimap { camera: 1280 })
    );
    assert_eq!(
        viewport.hit(1800.0, 1000.0, 100),
        Some(Hit::Minimap { camera: 0 })
    );
    assert_eq!(
        viewport.hit(1912.0 - 1e-9, 1000.0, usize::MAX),
        Some(Hit::Minimap { camera: i32::MAX })
    );
}

#[test]
fn shared_control_regions_and_physical_boundaries_agree_at_fractional_sizes() {
    let regions = hd::control_regions();
    assert_eq!(regions.len(), 14);
    assert_eq!(
        Control::Nuke.original_button(),
        Some(lemmings_game::panel::Button::Nuke)
    );
    assert_eq!(Control::Restart.original_button(), None);
    for (width, height) in [(1366.0, 768.0), (1280.0, 800.0), (1537.0, 863.0)] {
        let viewport = Viewport::fit(width, height).unwrap();
        let rect = viewport.rect();
        let scale = rect.width / 1920.0;
        for region in regions {
            let x = rect.x + region.rect.x * scale;
            let y = rect.y + region.rect.y * scale;
            let hit = viewport.hit(x, y, 1600);
            if region.control == Control::Minimap {
                assert_eq!(hit, Some(Hit::Minimap { camera: 0 }));
            } else {
                assert_eq!(hit, Some(Hit::Control(region.control)));
            }
        }
        let right = rect.x + rect.width;
        let just_inside = f64::from_bits(right.to_bits() - 1);
        assert_eq!(
            viewport.hit(just_inside, rect.y, 1600),
            Some(Hit::Game { x: 319, y: 0 })
        );
    }
}

#[test]
fn negative_extreme_actor_feet_do_not_wrap_or_change_the_source_record() {
    let mut game = run(Assets::synthetic());
    for _ in 0..80 {
        game.tick().unwrap();
    }
    let mut state = game.session.actors[0].state();
    state.x = i32::MIN;
    state.y = i32::MIN;
    game.session.actors[0] = lemmings_core::Actor::from_state(state, Default::default()).unwrap();
    game.view.camera = i32::MAX;
    let snapshot = hd::snapshot(&game).unwrap();
    assert_eq!(snapshot.actors[0].state, state);
    assert_eq!(
        snapshot.actors[0].foot,
        ((i64::from(i32::MIN) - 320) * 6, i64::from(i32::MIN) * 6)
    );
}

#[test]
fn explosion_snapshot_keeps_delayed_visible_mask_and_removed_particle_records() {
    let mut game = run(Assets::synthetic());
    for _ in 0..80 {
        game.tick().unwrap();
    }
    let mut state = game.session.actors[0].state();
    state.x = 120;
    state.y = 100;
    state.explosion_timer = 1;
    game.session.actors[0] = lemmings_core::Actor::from_state(state, Default::default()).unwrap();
    let mut found = false;
    for _ in 0..80 {
        game.tick().unwrap();
        if game.session.render_terrain() != game.session.terrain() {
            let snapshot = hd::snapshot(&game).unwrap();
            assert_ne!(snapshot.terrain.visible, snapshot.terrain.collision);
            assert_eq!(
                snapshot.terrain.visible.as_ptr(),
                game.session.render_terrain().as_ptr()
            );
            assert_eq!(snapshot.actors.len(), game.session.actors.len());
            assert!(snapshot.actors.iter().any(|a| a.state.removed
                == Some(lemmings_core::RemovalReason::Exploded)
                && a.state.particle_timer > 0));
            found = true;
            break;
        }
    }
    assert!(
        found,
        "fixture must exercise the actual explosion display delay"
    );
}

#[test]
fn completion_rules_come_from_the_session_even_if_asset_metadata_changes() {
    let mut game = run(Assets::synthetic_completion("win"));
    let required = game.session.completion_rules().unwrap().required();
    game.assets.required = Some(0);
    assert_eq!(hd::snapshot(&game).unwrap().hud.required, Some(required));
    game.assets.one_way_right.pop();
    assert_eq!(
        hd::snapshot(&game).unwrap_err(),
        SnapshotError::MaskLength("one_way_right")
    );
}

#[test]
fn sprite_anchors_keep_feet_fixed_when_trimmed_mirrored_or_offscreen() {
    let normal = hd::sprite_rect((-12, 30), (96, 60), (20, 55), false).unwrap();
    let flipped = hd::sprite_rect((-12, 30), (96, 60), (20, 55), true).unwrap();
    assert_eq!((normal.x, normal.y), (-32, -25));
    assert_eq!((flipped.x, flipped.y), (-88, -25));
    assert_eq!(hd::sprite_rect((0, 0), (0, 60), (0, 0), false), None);
    assert_eq!(
        hd::sprite_rect((i64::MIN, 0), (96, 60), (1, 0), false),
        None
    );
}

#[test]
fn snapshot_borrows_authoritative_masks_and_preserves_full_actor_state() {
    let mut game = run(Assets::synthetic());
    for _ in 0..80 {
        game.tick().unwrap();
    }
    game.view.camera = 1;
    let snapshot = hd::snapshot(&game).unwrap();
    assert_eq!(snapshot.version, 1);
    assert_eq!(
        snapshot.terrain.collision.as_ptr(),
        game.session.terrain().as_ptr()
    );
    assert_eq!(
        snapshot.terrain.visible.as_ptr(),
        game.session.render_terrain().as_ptr()
    );
    assert_eq!(
        snapshot.terrain.bricks.as_ptr(),
        game.session.render_bricks().as_ptr()
    );
    assert_eq!(snapshot.terrain.steel.as_ptr(), game.assets.steel.as_ptr());
    assert_eq!(
        snapshot.terrain.one_way_left.as_ptr(),
        game.assets.one_way_left.as_ptr()
    );
    assert_eq!(snapshot.traps, game.session.traps());
    assert_eq!(snapshot.actors.len(), game.session.actors.len());
    for (shown, original) in snapshot.actors.iter().zip(&game.session.actors) {
        assert_eq!(shown.state, original.state());
        assert_eq!(
            shown.foot,
            (
                (i64::from(shown.state.x) - 1) * 6,
                i64::from(shown.state.y) * 6
            )
        );
    }
}

#[test]
fn pending_pause_and_release_rate_are_shown_without_consuming_inputs() {
    let mut game = run(Assets::synthetic());
    game.session.queue(InputCommand::Pause).unwrap();
    game.session
        .queue(InputCommand::Custom(StartupCommand::SetReleaseRate(99)))
        .unwrap();
    let before = lemmings_game::diagnostics::state(&game.session);
    let snapshot = hd::snapshot(&game).unwrap();
    assert!(snapshot.hud.paused);
    assert_eq!(snapshot.hud.release_rate, 99);
    assert_eq!(snapshot.iteration, 0);
    assert_eq!(snapshot.control_step, 0);
    assert_eq!(lemmings_game::diagnostics::state(&game.session), before);
}

#[test]
fn camera_clamps_in_the_snapshot_without_mutating_the_original_view() {
    let mut game = run(Assets::synthetic());
    game.view.camera = i32::MAX;
    assert_eq!(hd::snapshot(&game).unwrap().camera, 320);
    assert_eq!(game.view.camera, i32::MAX);
    game.view.camera = -10;
    assert_eq!(hd::snapshot(&game).unwrap().camera, 0);
    assert_eq!(game.view.camera, -10);
}

#[test]
fn inconsistent_asset_dimensions_or_material_buffers_fail_before_rendering() {
    let mut game = run(Assets::synthetic());
    game.assets.width += 1;
    assert_eq!(
        hd::snapshot(&game).unwrap_err(),
        SnapshotError::AssetDimensions
    );
    game.assets.width -= 1;
    game.assets.steel.pop();
    assert_eq!(
        hd::snapshot(&game).unwrap_err(),
        SnapshotError::MaskLength("steel")
    );
}

#[test]
fn repeated_hd_snapshots_leave_ticks_terrain_original_rgb_and_sound_identical() {
    let mut observed = run(Assets::synthetic_builder());
    let mut control = run(Assets::synthetic_builder());
    for step in 0..180 {
        if step == 80 {
            for game in [&mut observed, &mut control] {
                game.session
                    .queue(InputCommand::Custom(StartupCommand::AssignBuilder(0)))
                    .unwrap();
            }
        }
        if step == 140 {
            for game in [&mut observed, &mut control] {
                game.session
                    .queue(InputCommand::Custom(StartupCommand::Nuke))
                    .unwrap();
            }
        }
        for _ in 0..3 {
            let snapshot = hd::snapshot(&observed).unwrap();
            assert_eq!(snapshot.hud.skills[4], observed.session.remaining_builders);
            assert_eq!(
                snapshot.iteration,
                observed.session.scheduler.step_state().iteration
            );
        }
        assert_eq!(
            lemmings_game::diagnostics::state(&observed.session),
            lemmings_game::diagnostics::state(&control.session)
        );
        assert_eq!(observed.session.terrain(), control.session.terrain());
        assert_eq!(observed.session.brick_mask(), control.session.brick_mask());
        assert_eq!(
            observed.sound.diagnostic_json(),
            control.sound.diagnostic_json()
        );
        if step % 17 == 0 {
            assert_eq!(
                observed
                    .view
                    .frame(&observed.assets, &observed.session)
                    .unwrap(),
                control
                    .view
                    .frame(&control.assets, &control.session)
                    .unwrap()
            );
        }
        observed.tick().unwrap();
        control.tick().unwrap();
    }
}

#[test]
fn frozen_results_and_a_fresh_restart_have_separate_display_state() {
    let mut game = run(Assets::synthetic_completion("win"));
    for _ in 0..1200 {
        game.tick().unwrap();
        if game.session.result().is_some() {
            break;
        }
    }
    let result = game.session.result().unwrap();
    assert!(result.success);
    let before = lemmings_game::diagnostics::state(&game.session);
    let shown = hd::snapshot(&game).unwrap();
    assert_eq!(shown.hud.result, Some(result));
    assert!(shown.hud.can_continue);
    assert_eq!(shown.hud.rescued_percent, result.rescued_percent);
    assert_eq!(lemmings_game::diagnostics::state(&game.session), before);
    let restart = game
        .prepare(ResultAction::Restart, Pacing::Reference, |_| {
            Ok((Assets::synthetic_completion("win"), game.next))
        })
        .unwrap()
        .unwrap();
    let fresh = hd::snapshot(&restart).unwrap();
    assert_eq!(fresh.hud.result, None);
    assert_eq!(
        (fresh.control_step, fresh.iteration, fresh.hud.rescued),
        (0, 0, 0)
    );
    assert_eq!(fresh.hud.skills[7], restart.session.remaining_diggers);
}
