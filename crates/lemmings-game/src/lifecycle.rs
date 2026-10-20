//! Shared scene transitions. Simulation results remain immutable until replacement.
use crate::{assets::Assets, pacing::Pacing, render::View, session::Session};
use std::error::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultAction {
    Restart,
    Continue,
}

pub struct LevelRun {
    pub level: u8,
    pub next: Option<u8>,
    pub assets: Assets,
    pub session: Session,
    pub view: View,
}

impl LevelRun {
    pub fn new(
        assets: Assets,
        level: u8,
        next: Option<u8>,
        pacing: Pacing,
    ) -> Result<Self, Box<dyn Error>> {
        let mut session = Session::new(
            assets.config,
            assets.entrances.clone(),
            assets.width,
            assets.height,
            assets.terrain.clone(),
        )?;
        session.set_pacing(pacing);
        if let Some(required) = assets.required {
            session.configure_completion(required)?;
        }
        session.configure_digger(assets.diggers, assets.steel.clone())?;
        session.configure_basher(
            assets.bashers,
            assets.bash_masks,
            assets.one_way_left.clone(),
            assets.one_way_right.clone(),
        )?;
        session.configure_miner(assets.miners, assets.mine_masks);
        session.configure_builder(assets.builders);
        session.configure_blocker(assets.blockers, assets.object_map.clone())?;
        session.configure_traps(assets.traps);
        session.configure_bomber(assets.bombers, assets.explosion_mask)?;
        session.configure_climber(assets.climbers);
        session.configure_floater(assets.floaters);
        let mut view = View::new(assets.screen_x);
        view.next_level = next;
        Ok(Self {
            level,
            next,
            assets,
            session,
            view,
        })
    }

    pub fn target(&self, action: ResultAction) -> Option<u8> {
        let result = self.session.result()?;
        match action {
            ResultAction::Restart => Some(self.level),
            ResultAction::Continue if !result.success => Some(self.level),
            ResultAction::Continue => self.next,
        }
    }

    /// Build before replacing: a missing/damaged next level leaves the result intact.
    pub fn prepare(
        &self,
        action: ResultAction,
        pacing: Pacing,
        load: impl FnOnce(u8) -> Result<(Assets, Option<u8>), Box<dyn Error>>,
    ) -> Result<Option<Self>, Box<dyn Error>> {
        let Some(level) = self.target(action) else {
            return Ok(None);
        };
        let (assets, next) = load(level)?;
        Self::new(assets, level, next, pacing).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lemmings_core::{InputCommand, StartupCommand};
    use std::time::Duration;

    fn finished(kind: &str, level: u8, next: Option<u8>) -> LevelRun {
        let mut run =
            LevelRun::new(Assets::synthetic_completion(kind), level, next, Pacing::Dos).unwrap();
        for _ in 0..1200 {
            run.session.tick().unwrap();
            if run.session.result().is_some() {
                return run;
            }
        }
        panic!("fixture did not finish");
    }

    #[test]
    fn continue_retries_failure_advances_success_and_stops_at_last_level() {
        let win = finished("win", 1, Some(2));
        assert_eq!(win.target(ResultAction::Continue), Some(2));
        assert_eq!(win.target(ResultAction::Restart), Some(1));
        let loss = finished("loss", 2, Some(3));
        assert_eq!(loss.target(ResultAction::Continue), Some(2));
        let last = finished("win", 30, None);
        assert_eq!(last.target(ResultAction::Continue), None);
        assert_eq!(last.target(ResultAction::Restart), Some(30));
        let playing =
            LevelRun::new(Assets::synthetic_completion("win"), 1, Some(2), Pacing::Dos).unwrap();
        assert_eq!(playing.target(ResultAction::Restart), None);
    }

    #[test]
    fn restart_recreates_every_session_state_and_view_without_mutating_assets() {
        let mut old = finished("win", 1, Some(2));
        old.view.camera = 80;
        old.view.selected_skill = crate::render::SelectedSkill::Builder;
        old.view.replaying = true;
        old.view.panel.held_rate = Some(1);
        old.view.panel.dragging_map = true;
        old.view.panel.pointer = Some((250, 180));
        old.view.right_mouse = true;
        let mut remembered = old.session.actors[0].state();
        remembered.removed = None;
        remembered.action = lemmings_core::MotionAction::Walking;
        old.view.selection.update(
            [remembered],
            Some((i64::from(remembered.x), i64::from(remembered.y) - 5)),
            false,
        );
        assert!(old.view.selection.hits.primary.is_some());
        old.session.remaining_diggers = 0;
        let fresh =
            LevelRun::new(Assets::synthetic_completion("win"), 1, Some(2), Pacing::Dos).unwrap();
        let restarted = old
            .prepare(ResultAction::Restart, Pacing::Dos, |_| {
                Ok((Assets::synthetic_completion("win"), Some(2)))
            })
            .unwrap()
            .unwrap();
        assert_eq!(restarted.session.snapshot(), fresh.session.snapshot());
        assert_eq!(
            restarted.session.digger_snapshot(),
            fresh.session.digger_snapshot()
        );
        assert_eq!(
            restarted.session.completion_snapshot(),
            fresh.session.completion_snapshot()
        );
        assert_eq!(restarted.session.terrain(), fresh.session.terrain());
        assert_eq!(
            restarted.session.object_map_data(),
            fresh.session.object_map_data()
        );
        assert_eq!(restarted.session.brick_mask(), fresh.session.brick_mask());
        assert!(restarted.session.inputs.is_empty());
        assert!(restarted.session.actors.is_empty());
        assert_eq!(restarted.view.camera, fresh.view.camera);
        assert!(!restarted.view.replaying);
        assert_eq!(restarted.view.panel.held_rate, None);
        assert!(!restarted.view.panel.dragging_map);
        assert_eq!(restarted.view.panel.pointer, None);
        assert!(!restarted.view.right_mouse);
        assert_eq!(
            restarted.view.selection,
            lemmings_core::PointerSelection::default()
        );
        assert_eq!(restarted.view.selected_skill, fresh.view.selected_skill);
        assert!(restarted.session.result().is_none());
        assert!(old.session.result().is_some());
    }

    #[test]
    fn nuke_inventory_terrain_traps_and_wall_clock_restart_from_pristine_assets() {
        let assets = Assets::synthetic_completion("particles");
        let mut dirty = LevelRun::new(assets, 1, None, Pacing::Dos).unwrap();
        dirty
            .session
            .queue(InputCommand::Custom(StartupCommand::AssignBomber(0)))
            .unwrap();
        for _ in 0..60 {
            dirty.session.tick().unwrap();
        }
        dirty
            .session
            .queue(InputCommand::Custom(StartupCommand::Nuke))
            .unwrap();
        for _ in 0..500 {
            dirty.session.tick().unwrap();
        }
        assert!(dirty.session.result().is_some());
        dirty.session.due(Duration::from_millis(69)).unwrap();
        let mut restarted = dirty
            .prepare(ResultAction::Restart, Pacing::Dos, |_| {
                Ok((Assets::synthetic_completion("particles"), None))
            })
            .unwrap()
            .unwrap();
        let mut fresh = LevelRun::new(
            Assets::synthetic_completion("particles"),
            1,
            None,
            Pacing::Dos,
        )
        .unwrap();
        assert_eq!(restarted.session.due(Duration::from_millis(2)).unwrap(), 0);
        for _ in 0..200 {
            restarted.session.tick().unwrap();
            fresh.session.tick().unwrap();
            assert_eq!(restarted.session.snapshot(), fresh.session.snapshot());
            assert_eq!(
                restarted.session.bomber_snapshot(),
                fresh.session.bomber_snapshot()
            );
            assert_eq!(
                restarted.session.hazards_snapshot(),
                fresh.session.hazards_snapshot()
            );
            assert_eq!(restarted.session.terrain(), fresh.session.terrain());
        }
    }

    #[test]
    fn restart_clears_triggered_traps_pause_and_changed_release_rate() {
        fn assets() -> Assets {
            let mut a = Assets::synthetic_hazards(0, false);
            a.config.total_lemmings = 1;
            a.required = Some(1);
            a
        }
        let mut old = LevelRun::new(assets(), 1, None, Pacing::Dos).unwrap();
        old.session
            .queue(InputCommand::Custom(StartupCommand::SetReleaseRate(99)))
            .unwrap();
        old.session.queue(InputCommand::Pause).unwrap();
        old.session.tick().unwrap();
        old.session.queue(InputCommand::Resume).unwrap();
        for _ in 0..1200 {
            old.session.tick().unwrap();
            if old.session.result().is_some() {
                break;
            }
        }
        assert!(old.session.result().is_some());
        assert!(old.session.traps().iter().flatten().any(|t| t.triggered()));
        let restarted = old
            .prepare(ResultAction::Restart, Pacing::Dos, |_| Ok((assets(), None)))
            .unwrap()
            .unwrap();
        let fresh = LevelRun::new(assets(), 1, None, Pacing::Dos).unwrap();
        assert_eq!(restarted.session.snapshot(), fresh.session.snapshot());
        assert_eq!(
            restarted.session.hazards_snapshot(),
            fresh.session.hazards_snapshot()
        );
        assert_eq!(restarted.session.traps(), fresh.session.traps());
        assert!(restarted.session.inputs.is_empty());
    }

    #[test]
    fn failed_next_load_preserves_result_session_and_view() {
        let old = finished("win", 1, Some(2));
        let before = old.session.snapshot();
        assert!(
            old.prepare(ResultAction::Continue, Pacing::Dos, |_| Err(
                "missing next assets".into()
            ))
            .is_err()
        );
        assert_eq!(old.session.snapshot(), before);
        assert!(old.session.result().unwrap().success);
        assert!(
            old.prepare(ResultAction::Continue, Pacing::Dos, |_| {
                let mut invalid = Assets::synthetic_completion("loss");
                invalid.entrances.clear();
                Ok((invalid, None))
            })
            .is_err()
        );
        assert_eq!(old.session.snapshot(), before);
        let next = old
            .prepare(ResultAction::Continue, Pacing::Dos, |_| {
                Ok((Assets::synthetic_completion("loss"), None))
            })
            .unwrap()
            .unwrap();
        assert_eq!(next.level, 2);
        assert!(next.session.result().is_none());
        assert_eq!(
            next.session.terrain(),
            Assets::synthetic_completion("loss").terrain
        );
    }
}
