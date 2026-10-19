//! DOSORIG pointer arbitration, independent of SDL, rendering and resources.
//! Pinned Game.pas PrioritizedHitTest; Styles.Base.pas supplies foot offsets.
use super::{ActorState, MotionAction};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssignmentSkill {
    Climber,
    Floater,
    Bomber,
    Blocker,
    Builder,
    Basher,
    Miner,
    Digger,
}

/// Stable IDs captured at the input boundary. The secondary target is used only
/// by Builder/Basher/Miner/Digger when the primary action cannot take that job.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PointerAssignment {
    pub skill: AssignmentSkill,
    pub primary: u32,
    pub secondary: Option<u32>,
    pub right_click: bool,
}
impl PointerAssignment {
    pub fn target(self, primary: ActorState) -> Option<u32> {
        use AssignmentSkill::*;
        use MotionAction::*;
        let fallback = match self.skill {
            Builder => {
                if i64::from(primary.y) - foot_y(primary.action) < -5 {
                    return None;
                }
                !matches!(
                    primary.action,
                    Walking | Shrugging | Bashing | Mining | Digging
                )
            }
            Basher => !matches!(
                primary.action,
                Walking | Shrugging | Building | Mining | Digging
            ),
            Miner => !matches!(
                primary.action,
                Walking | Shrugging | Building | Bashing | Digging
            ),
            Digger => {
                if primary.object_below == 137 {
                    return None;
                }
                !matches!(
                    primary.action,
                    Walking | Shrugging | Building | Bashing | Mining
                )
            }
            _ => false,
        };
        if fallback {
            self.secondary
        } else {
            Some(self.primary)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PointerHits {
    pub count: usize,
    pub primary: Option<u32>,
    pub secondary: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PointerSelection {
    pub hits: PointerHits,
    last_non_prioritized: Option<u32>,
}

fn foot_y(action: MotionAction) -> i64 {
    use MotionAction::*;
    match action {
        Building | Mining | Exiting => 13,
        Digging | Climbing | Hoisting => 12,
        Floating => 16,
        Vaporizing => 14,
        Exploding => 25,
        _ => 10,
    }
}
fn prioritized(action: MotionAction) -> bool {
    use MotionAction::*;
    matches!(
        action,
        Blocking | Building | Shrugging | Bashing | Mining | Digging | OhNo
    )
}
impl PointerSelection {
    /// `point` is the canonical world cursor hotspot (before Lemmix's -3,+2
    /// adjustment). Actors arrive in stable ID order. All comparisons use i64.
    pub fn update(
        &mut self,
        actors: impl IntoIterator<Item = ActorState>,
        point: Option<(i64, i64)>,
        right: bool,
    ) {
        self.hits = PointerHits::default();
        let Some((cx, cy)) = point.and_then(|(x, y)| Some((x.checked_sub(3)?, y.checked_add(2)?)))
        else {
            return;
        };
        let mut priority = None;
        for a in actors {
            if a.removed.is_some() {
                continue;
            }
            let left = i64::from(a.x)
                - if a.action == MotionAction::Exploding {
                    16
                } else {
                    8
                };
            let top = i64::from(a.y) - foot_y(a.action);
            if (left..=left + 12).contains(&cx) && (top..=top + 12).contains(&cy) {
                self.hits.count += 1;
                if prioritized(a.action) {
                    priority = Some(a.id);
                } else {
                    self.hits.secondary = Some(a.id);
                }
            }
        }
        if self.hits.secondary.is_some() {
            self.last_non_prioritized = self.hits.secondary;
        }
        self.hits.primary = if right {
            self.hits.secondary
        } else {
            priority.or(self.hits.secondary)
        };
    }
    pub fn assignment(&self, skill: AssignmentSkill) -> Option<PointerAssignment> {
        if self.hits.count == 0 {
            return None;
        }
        Some(PointerAssignment {
            skill,
            primary: self.hits.primary.or(self.last_non_prioritized)?,
            secondary: self.hits.secondary,
            right_click: self.hits.primary.is_none(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Actor, EntrancePoint, MotionDirection, MotionRules, RemovalReason, SpawnRequest};
    fn actor(id: u32, action: MotionAction) -> ActorState {
        let mut s = Actor::from_spawn(
            SpawnRequest {
                actor_id: id,
                entrance_index: 0,
                iteration: 54,
            },
            &[EntrancePoint { x: 76, y: 86 }],
            MotionRules::default(),
        )
        .unwrap()
        .state();
        s.action = action;
        s.fallen = 0;
        s.direction = MotionDirection::Right;
        s
    }
    #[test]
    fn inclusive_hotspot_edges_use_each_actions_foot() {
        for action in [
            MotionAction::Walking,
            MotionAction::Mining,
            MotionAction::Floating,
            MotionAction::Exploding,
        ] {
            let a = actor(0, action);
            let left = 100
                - if action == MotionAction::Exploding {
                    13
                } else {
                    5
                };
            let top = 100 - foot_y(action) - 2;
            for (x, y, hit) in [
                (left, top, true),
                (left + 12, top + 12, true),
                (left - 1, top, false),
                (left + 13, top, false),
                (left, top - 1, false),
                (left, top + 13, false),
            ] {
                let mut p = PointerSelection::default();
                p.update([a], Some((x, y)), false);
                assert_eq!(p.hits.count, usize::from(hit), "{action:?} {x},{y}");
            }
        }
    }
    #[test]
    fn priority_and_last_actor_override_distance_and_right_filters_workers() {
        let mut last = actor(2, MotionAction::Walking);
        last.x = 105;
        let mut p = PointerSelection::default();
        p.update(
            [
                actor(0, MotionAction::Building),
                actor(1, MotionAction::Walking),
                last,
            ],
            Some((100, 95)),
            false,
        );
        assert_eq!(
            p.hits,
            PointerHits {
                count: 3,
                primary: Some(0),
                secondary: Some(2)
            }
        );
        p.update(
            [
                actor(0, MotionAction::Building),
                actor(1, MotionAction::Walking),
                last,
            ],
            Some((100, 95)),
            true,
        );
        assert_eq!(p.hits.primary, Some(2));
    }
    #[test]
    fn remote_assignment_requires_worker_hit_but_keeps_last_target_across_empty_space_and_trap() {
        let a = actor(7, MotionAction::Walking);
        let mut p = PointerSelection::default();
        p.update([a], Some((100, 95)), false);
        let mut dead = a;
        dead.removed = Some(RemovalReason::Trapped);
        p.update([dead], None, true);
        assert!(p.assignment(AssignmentSkill::Blocker).is_none());
        let mut worker = actor(8, MotionAction::Building);
        worker.x = 200;
        p.update([dead, worker], Some((200, 95)), true);
        assert_eq!(p.hits.primary, None);
        assert_eq!(
            p.assignment(AssignmentSkill::Blocker),
            Some(PointerAssignment {
                skill: AssignmentSkill::Blocker,
                primary: 7,
                secondary: None,
                right_click: true
            })
        );
        p = PointerSelection::default();
        p.update([worker], Some((200, 95)), true);
        assert!(p.assignment(AssignmentSkill::Blocker).is_none());
    }
    #[test]
    fn fallback_checks_action_and_primary_guards_before_secondary() {
        let mut a = actor(3, MotionAction::Blocking);
        let mut request = PointerAssignment {
            skill: AssignmentSkill::Digger,
            primary: 3,
            secondary: Some(4),
            right_click: false,
        };
        assert_eq!(request.target(a), Some(4));
        a.object_below = 137;
        assert_eq!(request.target(a), None);
        request.skill = AssignmentSkill::Builder;
        a.y = 4;
        assert_eq!(request.target(a), None);
        request.skill = AssignmentSkill::Climber;
        assert_eq!(request.target(a), Some(3));
    }
    #[test]
    fn extreme_coordinates_do_not_overflow_or_select_removed_particles() {
        let mut a = actor(1, MotionAction::Exploding);
        a.x = i32::MIN;
        a.y = i32::MAX;
        let mut p = PointerSelection::default();
        p.update([a], Some((i64::from(a.x), i64::from(a.y))), false);
        a.removed = Some(RemovalReason::Exploded);
        a.particle_timer = 52;
        p.update([a], Some((i64::from(a.x), i64::from(a.y))), false);
        assert_eq!(p.hits.count, 0);
        for point in [(i64::MIN, i64::MAX), (i64::MAX, i64::MIN)] {
            p.update([a], Some(point), true);
            assert_eq!(p.hits.count, 0);
        }
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;
    use crate::{Actor, EntrancePoint, MotionRules, SpawnRequest};
    #[test]
    fn secondary_builder_head_and_digger_steel_are_guarded_on_primary_only() {
        let mut primary = Actor::from_spawn(
            SpawnRequest {
                actor_id: 1,
                entrance_index: 0,
                iteration: 54,
            },
            &[EntrancePoint { x: 76, y: 86 }],
            MotionRules::default(),
        )
        .unwrap()
        .state();
        primary.action = MotionAction::Blocking;
        primary.fallen = 0;
        primary.blocker_saved = Some([128; 9]);
        primary.object_below = 128;
        let mut secondary = primary;
        secondary.id = 0;
        secondary.action = MotionAction::Walking;
        secondary.blocker_saved = None;
        secondary.y = 3;
        let mut actor = Actor::from_state(secondary, MotionRules::default()).unwrap();
        let mut n = 2;
        assert!(!actor.assign_builder(&mut n));
        assert!(actor.assign_builder_secondary(&mut n, primary));
        assert_eq!(n, 1);
        secondary.y = 100;
        secondary.object_below = 137;
        let mut actor = Actor::from_state(secondary, MotionRules::default()).unwrap();
        assert!(actor.assign_digger_secondary(&mut n, primary));
        assert_eq!(n, 0);
        assert_eq!(actor.state().action, MotionAction::Digging);
    }
}
