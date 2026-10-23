//! Check every restored original actor slot/frame through the shared renderer.
//! Synthetic restored states are not natural gameplay or performance evidence.
#[cfg(test)]
mod tests {
    #[test]
    fn all_restored_actor_states_render_without_ticks_or_original_fallback() {
        let mut b = b"LHDPAK01".to_vec();
        b.extend(1u16.to_le_bytes());
        b.extend(337u16.to_le_bytes());
        for n in [1u32, 1, 4, 0x0c463091, 0] {
            b.extend(n.to_le_bytes());
        }
        b.extend([0, 0, 0, 255]);
        for slot in 0..28 {
            let info = lemmings_resources::LemmingAnimationInfo::for_slot(slot).unwrap();
            for frame in 0..u16::from(info.frame_count) {
                b.extend([0, slot as u8]);
                for n in [frame, 0, 0] {
                    b.extend(n.to_le_bytes());
                }
                for n in [0u32, 0, 1, 1] {
                    b.extend(n.to_le_bytes());
                }
                for n in [96u16, 96, 48, 90] {
                    b.extend(n.to_le_bytes());
                }
            }
        }
        assert_eq!(super::check(&b).unwrap(), 337);
    }
}
use lemmings_core::{Actor, MotionAction, MotionDirection, MotionRules};
use lemmings_game::{Assets, LevelRun, hd::scene::Renderer, pacing::Pacing};
use lemmings_resources::{LemmingAnimationInfo, LemmingSpriteKind, SpriteDirection};
use std::error::Error;

fn check(bytes: &[u8]) -> Result<usize, Box<dyn Error>> {
    let mut renderer = Renderer::new(bytes)?;
    let mut run = LevelRun::new(Assets::synthetic_builder(), 1, None, Pacing::Reference)?;
    for _ in 0..60 {
        run.tick()?;
    }
    let initial = run
        .session
        .actors
        .first()
        .ok_or("no synthetic actor")?
        .state();
    renderer.reset(&run.assets)?;
    let mut checks = 0;
    for slot in 0..28 {
        let info = LemmingAnimationInfo::for_slot(slot).ok_or("source slot")?;
        let action = match info.kind {
            LemmingSpriteKind::Walking => MotionAction::Walking,
            LemmingSpriteKind::Jumping => MotionAction::Jumping,
            LemmingSpriteKind::Falling => MotionAction::Falling,
            LemmingSpriteKind::Splatting => MotionAction::Splatting,
            LemmingSpriteKind::Digging => MotionAction::Digging,
            LemmingSpriteKind::Bashing => MotionAction::Bashing,
            LemmingSpriteKind::Mining => MotionAction::Mining,
            LemmingSpriteKind::Building => MotionAction::Building,
            LemmingSpriteKind::Shrugging => MotionAction::Shrugging,
            LemmingSpriteKind::Blocking => MotionAction::Blocking,
            LemmingSpriteKind::Climbing => MotionAction::Climbing,
            LemmingSpriteKind::Umbrella => MotionAction::Floating,
            LemmingSpriteKind::Drowning => MotionAction::Drowning,
            LemmingSpriteKind::Vaporizing => MotionAction::Vaporizing,
            LemmingSpriteKind::Exiting => MotionAction::Exiting,
            LemmingSpriteKind::Hoisting => MotionAction::Hoisting,
            LemmingSpriteKind::OhNo => MotionAction::OhNo,
            LemmingSpriteKind::Explosion => MotionAction::Exploding,
        };
        for frame in 0..info.frame_count {
            let mut state = initial;
            state.x = 100;
            state.y = 100;
            state.action = action;
            state.frame = frame;
            state.fallen = 0;
            state.direction = if action == MotionAction::Splatting {
                MotionDirection::Stopped
            } else if info.direction == Some(SpriteDirection::Left) {
                MotionDirection::Left
            } else {
                MotionDirection::Right
            };
            state.is_climber = matches!(action, MotionAction::Climbing | MotionAction::Hoisting);
            state.is_floater = action == MotionAction::Floating;
            state.blocker_saved = (action == MotionAction::Blocking).then_some([0; 9]);
            state.bricks_left = if action == MotionAction::Building {
                12
            } else {
                0
            };
            run.session.actors.clear();
            run.session.actors.push(
                Actor::from_state(state, MotionRules::default())
                    .map_err(|e| format!("{slot}/{frame}: {e:?}"))?,
            );
            let before = lemmings_game::diagnostics::state(&run.session);
            let sound = run.sound.diagnostic_json();
            renderer.frame(&run)?;
            if !renderer.missing().is_empty()
                || !renderer
                    .info()
                    .contains(&format!("\"hd_actor_frames\":[[{slot},{frame}]]"))
            {
                return Err(format!("wrong HD actor route {slot}/{frame}").into());
            }
            if before != lemmings_game::diagnostics::state(&run.session)
                || sound != run.sound.diagnostic_json()
            {
                return Err("rendering changed restored state or sound".into());
            }
            checks += 1;
        }
    }
    Ok(checks)
}
fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .ok_or("usage: hd_character_probe assets.hdp")?;
    let bytes = std::fs::read(path)?;
    let checks = check(&bytes)?;
    println!(
        "{{\"status\":\"PASS\",\"restored_animations\":28,\"restored_frames\":{checks},\"no_actor_fallback\":true,\"rendering_read_only\":true,\"scope\":\"restored synthetic actor states, not natural gameplay or performance\"}}"
    );
    Ok(())
}
