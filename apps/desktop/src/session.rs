use crate::pacing::{Pacing, PlaybackClock};
use lemmings_core::*;
use std::{error::Error, fmt::Write as _, time::Duration};

pub struct Session {
    pub scheduler: StartupScheduler,
    pub actors: Vec<Actor>,
    pub inputs: Vec<TimedInput<StartupCommand>>,
    pub events: Vec<String>,
    pub remaining_diggers: u16,
    pub remaining_bashers: u16,
    pub remaining_miners: u16,
    pub remaining_builders: u16,
    pub remaining_blockers: u16,
    pub remaining_bombers: u16,
    pub remaining_climbers: u16,
    pub remaining_floaters: u16,
    explosion_mask: [u8; 44],
    visual_blast: Option<(Vec<u8>, Vec<u8>)>,
    object_map: Option<Vec<u8>>,
    has_exit: bool,
    rescued_this_tick: bool,
    traps: TrapStates,
    hazard_changed: bool,
    completion: Option<CompletionRules>,
    result: Option<LevelResult>,
    brick_mask: Vec<u8>,
    mine_masks: [u8; 104],
    bash_masks: [u8; 160],
    one_way_left: Vec<u8>,
    one_way_right: Vec<u8>,
    steel: Vec<u8>,
    clock: PlaybackClock,
    entrances: Vec<EntrancePoint>,
    terrain: Vec<u8>,
    width: usize,
    height: usize,
}
impl Session {
    pub fn new(
        config: StartupConfig,
        entrances: Vec<EntrancePoint>,
        width: usize,
        height: usize,
        terrain: Vec<u8>,
    ) -> Result<Self, Box<dyn Error>> {
        TerrainMask::new(width, height, &terrain)?;
        if entrances.len() != usize::from(config.entrance_count) {
            return Err("entrance count mismatch".into());
        }
        for point in &entrances {
            point.x.checked_add(24).ok_or("entrance x overflow")?;
            point.y.checked_add(14).ok_or("entrance y overflow")?;
        }
        Ok(Self {
            scheduler: StartupScheduler::new(config)?,
            actors: Vec::new(),
            inputs: Vec::new(),
            events: Vec::new(),
            remaining_diggers: 0,
            remaining_bashers: 0,
            remaining_miners: 0,
            remaining_builders: 0,
            remaining_blockers: 0,
            remaining_bombers: 0,
            remaining_climbers: 0,
            remaining_floaters: 0,
            explosion_mask: [0; 44],
            visual_blast: None,
            object_map: None,
            has_exit: false,
            rescued_this_tick: false,
            traps: [None; 16],
            hazard_changed: false,
            completion: None,
            result: None,
            brick_mask: vec![0; width * height],
            mine_masks: [0; 104],
            bash_masks: [0; 160],
            one_way_left: vec![0; width * height],
            one_way_right: vec![0; width * height],
            steel: vec![0; width * height],
            clock: PlaybackClock::new(Pacing::Reference),
            entrances,
            terrain,
            width,
            height,
        })
    }
    pub fn configure_completion(&mut self, required: u32) -> Result<(), Box<dyn Error>> {
        if self.completion.is_some() {
            return Err("completion already configured".into());
        }
        self.completion = Some(CompletionRules::new(
            self.scheduler.config().total_lemmings,
            required,
        )?);
        Ok(())
    }
    pub fn result(&self) -> Option<LevelResult> {
        self.result
    }
    pub fn completion_rules(&self) -> Option<CompletionRules> {
        self.completion
    }
    pub fn completion_snapshot(&self) -> String {
        let Some(rules) = self.completion else {
            return "null".into();
        };
        let denominator = self.scheduler.nuke_state().max_lemmings;
        let done = percentage(self.saved() as u32, denominator);
        let phase = self
            .result
            .map_or("Playing", |r| if r.success { "Won" } else { "Lost" });
        let reason = self
            .result
            .map_or_else(|| "null".into(), |r| format!("\"{:?}\"", r.reason));
        format!(
            "{{\"phase\":\"{phase}\",\"reason\":{reason},\"original_total\":{},\"lemming_count\":{denominator},\"required\":{},\"rescued\":{},\"target_percent\":{},\"rescued_percent\":{done}}}",
            rules.total(),
            rules.required(),
            self.saved(),
            rules.target_percent()
        )
    }
    pub fn configure_digger(
        &mut self,
        remaining: u16,
        steel: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        TerrainMask::new(self.width, self.height, &steel)?;
        self.remaining_diggers = remaining;
        self.steel = steel;
        Ok(())
    }
    pub fn render_terrain(&self) -> &[u8] {
        self.visual_blast.as_ref().map_or(self.terrain(), |v| &v.0)
    }
    pub fn render_bricks(&self) -> &[u8] {
        self.visual_blast
            .as_ref()
            .map_or(self.brick_mask(), |v| &v.1)
    }
    pub fn terrain(&self) -> &[u8] {
        &self.terrain
    }
    pub fn configure_basher(
        &mut self,
        remaining: u16,
        masks: [u8; 160],
        left: Vec<u8>,
        right: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        TerrainMask::new(self.width, self.height, &left)?;
        TerrainMask::new(self.width, self.height, &right)?;
        self.remaining_bashers = remaining;
        self.bash_masks = masks;
        self.one_way_left = left;
        self.one_way_right = right;
        Ok(())
    }
    pub fn digger_snapshot(&self) -> String {
        let hash = self.terrain.iter().fold(0xcbf29ce484222325u64, |h, &v| {
            (h ^ u64::from(v)).wrapping_mul(0x100000001b3)
        });
        let flags: Vec<_> = self
            .actors
            .iter()
            .filter(|a| a.state().new_digger)
            .map(|a| a.state().id.to_string())
            .collect();
        format!(
            "{{\"remaining\":{},\"new_diggers\":[{}],\"terrain_fnv1a64\":\"{hash:016x}\"}}",
            self.remaining_diggers,
            flags.join(",")
        )
    }
    pub fn configure_miner(&mut self, remaining: u16, masks: [u8; 104]) {
        self.remaining_miners = remaining;
        self.mine_masks = masks;
    }
    pub fn configure_blocker(
        &mut self,
        remaining: u16,
        cells: Vec<u8>,
    ) -> Result<(), Box<dyn Error>> {
        ObjectMap::new(
            DOS_OBJECT_MAP_WIDTH,
            DOS_OBJECT_MAP_HEIGHT,
            DOS_OBJECT_MAP_ORIGIN,
            DOS_OBJECT_MAP_ORIGIN,
            &cells,
        )?;
        self.remaining_blockers = remaining;
        self.has_exit = cells.contains(&OBJECT_EXIT);
        self.object_map = Some(cells);
        Ok(())
    }
    pub fn configure_traps(&mut self, traps: TrapStates) {
        self.traps = traps;
    }
    pub fn traps(&self) -> &TrapStates {
        &self.traps
    }
    pub fn hazard_just_happened(&self) -> bool {
        self.hazard_changed
    }
    pub fn hazards_snapshot(&self) -> String {
        let traps = self
            .traps
            .iter()
            .enumerate()
            .filter_map(|(i, t)| t.map(|t| format!("[{i},{},{}]", t.frame(), t.triggered())))
            .collect::<Vec<_>>()
            .join(",");
        let count = |reason| {
            self.actors
                .iter()
                .filter(|a| a.state().removed == Some(reason))
                .count()
        };
        format!(
            "{{\"traps\":[{traps}],\"drowned\":{},\"vaporized\":{},\"trapped\":{}}}",
            count(RemovalReason::Drowned),
            count(RemovalReason::Vaporized),
            count(RemovalReason::Trapped)
        )
    }
    pub fn object_map_data(&self) -> &[u8] {
        self.object_map.as_deref().unwrap_or(&[])
    }
    pub fn has_exit(&self) -> bool {
        self.has_exit
    }
    pub fn rescue_just_happened(&self) -> bool {
        self.rescued_this_tick
    }
    pub fn saved(&self) -> usize {
        self.actors
            .iter()
            .filter(|a| a.state().removed == Some(RemovalReason::Rescued))
            .count()
    }
    pub fn exit_snapshot(&self) -> String {
        let exiting = self
            .actors
            .iter()
            .filter_map(|a| {
                let s = a.state();
                (s.action == MotionAction::Exiting && s.removed.is_none()).then(|| s.id.to_string())
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("{{\"saved\":{},\"exiting\":[{exiting}]}}", self.saved())
    }
    pub fn blocker_snapshot(&self) -> String {
        let fields = self
            .actors
            .iter()
            .filter_map(|a| {
                a.state()
                    .blocker_saved
                    .map(|saved| format!("[{}, {:?}]", a.state().id, saved))
            })
            .collect::<Vec<_>>()
            .join(",");
        let cached = self
            .actors
            .iter()
            .map(|a| {
                format!(
                    "[{},{},{}]",
                    a.state().id,
                    a.state().object_below,
                    a.state().object_in_front
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let hash = self
            .object_map_data()
            .iter()
            .fold(0xcbf29ce484222325u64, |h, &v| {
                (h ^ u64::from(v)).wrapping_mul(0x100000001b3)
            });
        format!(
            "{{\"remaining\":{},\"fields\":[{fields}],\"objects\":[{cached}],\"object_map_fnv1a64\":\"{hash:016x}\"}}",
            self.remaining_blockers
        )
    }
    pub fn configure_floater(&mut self, remaining: u16) {
        self.remaining_floaters = remaining;
    }
    pub fn floater_snapshot(&self) -> String {
        let states = self
            .actors
            .iter()
            .map(|a| {
                let s = a.state();
                format!("[{},{},{}]", s.id, s.is_floater, s.float_index)
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"remaining\":{},\"actors\":[{states}]}}",
            self.remaining_floaters
        )
    }
    pub fn configure_climber(&mut self, remaining: u16) {
        self.remaining_climbers = remaining;
    }
    pub fn climber_snapshot(&self) -> String {
        let states = self
            .actors
            .iter()
            .map(|a| {
                let s = a.state();
                format!("[{},{},{}]", s.id, s.is_climber, s.walking_once)
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"remaining\":{},\"actors\":[{states}]}}",
            self.remaining_climbers
        )
    }
    pub fn configure_bomber(
        &mut self,
        remaining: u16,
        mask: [u8; 44],
    ) -> Result<(), Box<dyn Error>> {
        if self.object_map.is_none() {
            return Err("Bomber requires a configured object map".into());
        }
        self.remaining_bombers = remaining;
        self.explosion_mask = mask;
        Ok(())
    }
    pub fn bomber_snapshot(&self) -> String {
        let states = self
            .actors
            .iter()
            .map(|a| {
                let s = a.state();
                format!(
                    "[{},{},{},{}]",
                    s.id, s.explosion_timer, s.particle_timer, s.particle_frame
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"remaining\":{},\"actors\":[{states}]}}",
            self.remaining_bombers
        )
    }
    pub fn configure_builder(&mut self, remaining: u16) {
        self.remaining_builders = remaining;
    }
    pub fn brick_mask(&self) -> &[u8] {
        &self.brick_mask
    }
    pub fn builder_snapshot(&self) -> String {
        let bricks = self
            .actors
            .iter()
            .filter(|a| a.state().action == MotionAction::Building)
            .map(|a| format!("[{},{}]", a.state().id, a.state().bricks_left))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"remaining\":{},\"bricks\":[{bricks}]}}",
            self.remaining_builders
        )
    }
    pub fn desired_paused(&self) -> bool {
        self.inputs
            .iter()
            .rev()
            .take_while(|i| i.control_step == self.scheduler.step_state().control_step)
            .find_map(|i| match i.command {
                InputCommand::Pause => Some(true),
                InputCommand::Resume => Some(false),
                _ => None,
            })
            .unwrap_or(self.scheduler.step_state().paused)
    }

    pub fn desired_rate(&self) -> u8 {
        self.inputs
            .iter()
            .rev()
            .take_while(|i| i.control_step == self.scheduler.step_state().control_step)
            .find_map(|i| match i.command {
                InputCommand::Custom(StartupCommand::SetReleaseRate(rate))
                    if (self.scheduler.config().initial_release_rate..=99).contains(&rate) =>
                {
                    Some(rate)
                }
                _ => None,
            })
            .unwrap_or(self.scheduler.state().release_rate)
    }
    pub fn queue(&mut self, command: InputCommand<StartupCommand>) -> Result<(), Box<dyn Error>> {
        if self.result.is_some() {
            return Err("level already finished".into());
        }
        let control_step = self.scheduler.step_state().control_step;
        let sequence = match self
            .inputs
            .last()
            .filter(|i| i.control_step == control_step)
        {
            Some(previous) => previous
                .sequence
                .checked_add(1)
                .ok_or("input sequence overflow")?,
            None => 0,
        };
        self.inputs.push(TimedInput {
            control_step,
            sequence,
            command,
        });
        Ok(())
    }
    pub fn tick(&mut self) -> Result<(), Box<dyn Error>> {
        if self.result.is_some() {
            return Ok(());
        }
        let seconds_before_tick = self.scheduler.state().seconds_remaining;
        let mut events = Vec::new();
        let mut rescued_this_tick = false;
        let mut hazard_changed = false;
        let mut spawn = None;
        let schedule = InputSchedule::new(&self.inputs)?;
        let steel = TerrainMask::new(self.width, self.height, &self.steel)?;
        let actors = &mut self.actors;
        let remaining = &mut self.remaining_diggers;
        let bashers = &mut self.remaining_bashers;
        let miners = &mut self.remaining_miners;
        let builders = &mut self.remaining_builders;
        let blockers = &mut self.remaining_blockers;
        let bombers = &mut self.remaining_bombers;
        let climbers = &mut self.remaining_climbers;
        let floaters = &mut self.remaining_floaters;
        let raw_map = &mut self.object_map;
        let context = BashingContext {
            object_map: None,
            masks: &self.bash_masks,
            steel,
            one_way_left: Some(TerrainMask::new(
                self.width,
                self.height,
                &self.one_way_left,
            )?),
            one_way_right: Some(TerrainMask::new(
                self.width,
                self.height,
                &self.one_way_right,
            )?),
        };
        let mining = MiningContext::new(&self.mine_masks, context);
        let outcome = self.scheduler.step_with_skills(
            &schedule,
            |command| {
                let id = match command {
                    StartupCommand::AssignDigger(id)
                    | StartupCommand::AssignBasher(id)
                    | StartupCommand::AssignMiner(id)
                    | StartupCommand::AssignBuilder(id)
                    | StartupCommand::AssignBlocker(id)
                    | StartupCommand::AssignBomber(id)
                    | StartupCommand::AssignClimber(id)
                    | StartupCommand::AssignFloater(id) => id,
                    _ => return false,
                };
                actors
                    .iter_mut()
                    .find(|a| a.state().id == id)
                    .is_some_and(|a| {
                        if matches!(command, StartupCommand::AssignFloater(_)) {
                            return a.assign_floater(floaters);
                        }
                        if matches!(command, StartupCommand::AssignClimber(_)) {
                            return a.assign_climber(climbers);
                        }
                        if matches!(command, StartupCommand::AssignBomber(_)) {
                            return a.assign_bomber(bombers);
                        }
                        if matches!(command, StartupCommand::AssignBlocker(_)) {
                            return raw_map.as_mut().is_some_and(|cells| {
                                a.assign_blocker(
                                    blockers,
                                    &mut ObjectMapMut::new(
                                        DOS_OBJECT_MAP_WIDTH,
                                        DOS_OBJECT_MAP_HEIGHT,
                                        DOS_OBJECT_MAP_ORIGIN,
                                        DOS_OBJECT_MAP_ORIGIN,
                                        cells,
                                    )
                                    .expect("configured map"),
                                )
                            });
                        }
                        let objects = BashingContext {
                            object_map: raw_map.as_ref().map(|cells| {
                                ObjectMap::new(
                                    DOS_OBJECT_MAP_WIDTH,
                                    DOS_OBJECT_MAP_HEIGHT,
                                    DOS_OBJECT_MAP_ORIGIN,
                                    DOS_OBJECT_MAP_ORIGIN,
                                    cells,
                                )
                                .expect("configured map")
                            }),
                            ..context
                        };
                        match command {
                            StartupCommand::AssignDigger(_) => {
                                a.assign_digger_with_objects(remaining, objects)
                            }
                            StartupCommand::AssignBasher(_) => a.assign_basher(bashers, objects),
                            StartupCommand::AssignMiner(_) => {
                                a.assign_miner(miners, MiningContext { objects, ..mining })
                            }
                            StartupCommand::AssignBuilder(_) => a.assign_builder(builders),
                            _ => false,
                        }
                    })
            },
            |event| match event {
                StartupEvent::Input {
                    control_step,
                    sequence,
                    accepted,
                } => events.push(format!("Input:{control_step}:{sequence}:{accepted}")),
                StartupEvent::Spawn(request) => {
                    events.push(format!(
                        "Spawn:{}:{}:{}",
                        request.actor_id, request.entrance_index, request.iteration
                    ));
                    spawn = Some(request);
                }
                StartupEvent::LetsGo => events.push("LetsGo".into()),
                StartupEvent::EntranceSound => events.push("EntranceSound".into()),
                StartupEvent::EntrancesOpened => events.push("EntrancesOpened".into()),
                StartupEvent::Tick(_) => {}
            },
        )?;
        if let Some(request) = spawn {
            self.actors.push(Actor::from_spawn(
                request,
                &self.entrances,
                MotionRules::default(),
            )?);
        }
        if outcome.iteration.is_some() {
            self.visual_blast = None;
            let mut blast_pixels = Vec::new();
            let brick_mask = &mut self.brick_mask;
            let width = self.width;
            let update_bricks = |state: ActorState, out: MotionOutcome| {
                rescued_this_tick |= out.removal == Some(RemovalReason::Rescued);
                hazard_changed |= matches!(
                    out.removal,
                    Some(
                        RemovalReason::Drowned | RemovalReason::Vaporized | RemovalReason::Trapped
                    )
                );
                for (dy, row) in out.explosion_erased.into_iter().enumerate() {
                    for dx in 0..16 {
                        if row & (1 << dx) != 0 {
                            // Core reports only real, clipped changes. Preserve the
                            // pre-erasure material for the DOS one-frame display lag.
                            let x = (i64::from(state.x) - 8 + dx as i64) as usize;
                            let y = (i64::from(state.y) - 14 + dy as i64) as usize;
                            let index = y * width + x;
                            blast_pixels.push((index, brick_mask[index]));
                        }
                    }
                }
                for (x, y) in out.added_pixels.into_iter().flatten() {
                    brick_mask[y as usize * width + x as usize] = 1;
                }
            };
            let mut terrain = TerrainMaskMut::new(self.width, self.height, &mut self.terrain)?;
            if let Some(cells) = &mut self.object_map {
                let mut map = ObjectMapMut::new(
                    DOS_OBJECT_MAP_WIDTH,
                    DOS_OBJECT_MAP_HEIGHT,
                    DOS_OBJECT_MAP_ORIGIN,
                    DOS_OBJECT_MAP_ORIGIN,
                    cells,
                )?;
                step_actors_with_hazards(
                    &mut self.actors,
                    &mut terrain,
                    BomberContext::new(mining, &self.explosion_mask),
                    &mut map,
                    &mut self.traps,
                    update_bricks,
                )?;
            } else {
                step_actors_with_jobs(&mut self.actors, &mut terrain, mining, update_bricks)?;
            }
            for (brick, &solid) in self.brick_mask.iter_mut().zip(&self.terrain) {
                if solid == 0 {
                    *brick = 0;
                }
            }
            if !blast_pixels.is_empty() {
                let mut visual = self.terrain.clone();
                let mut bricks = self.brick_mask.clone();
                for (index, material) in blast_pixels {
                    if visual[index] == 0 {
                        visual[index] = 1;
                        bricks[index] = material;
                    }
                }
                self.visual_blast = Some((visual, bricks));
            }
            self.scheduler.update_nuking(&mut self.actors)?;
            advance_traps(&mut self.traps);
        }
        if let Some(rules) = self.completion {
            self.result = rules.evaluate(CompletionProgress {
                seconds_before_tick,
                released: self.scheduler.state().released,
                removed: self
                    .actors
                    .iter()
                    .filter(|a| a.state().removed.is_some())
                    .count()
                    .try_into()?,
                rescued: self.saved().try_into()?,
                max_lemmings: self.scheduler.nuke_state().max_lemmings,
                nuked: self.scheduler.nuke_state().is_nuked,
                particle_timer: self
                    .actors
                    .iter()
                    .map(|a| a.state().particle_timer)
                    .max()
                    .unwrap_or(0),
            })?;
        }
        self.events = events;
        self.rescued_this_tick = rescued_this_tick;
        self.hazard_changed = hazard_changed;
        Ok(())
    }
    pub fn due(&mut self, elapsed: Duration) -> Result<u64, Box<dyn Error>> {
        Ok(self.clock.advance(elapsed)?)
    }
    pub fn set_pacing(&mut self, pacing: Pacing) {
        self.clock = PlaybackClock::new(pacing);
    }
    #[cfg(test)]
    pub fn advance(&mut self, elapsed: Duration) -> Result<u64, Box<dyn Error>> {
        let due = self.due(elapsed)?;
        for _ in 0..due {
            self.tick()?;
        }
        Ok(due)
    }
    pub fn snapshot(&self) -> String {
        snapshot(&self.scheduler, &self.actors, &self.events)
    }
}
fn snapshot(scheduler: &StartupScheduler, actors: &[Actor], events: &[String]) -> String {
    let s = scheduler.state();
    let step = scheduler.step_state();
    let removed = actors
        .iter()
        .filter(|a| a.state().removed.is_some())
        .count();
    let mut out = format!(
        "{{\"control_step\":{},\"iteration\":{},\"paused\":{},\"seconds_remaining\":{},\"clock_frame\":{},\"glitch_pause_iterations\":{},\"release_rate\":{},\"spawn_countdown\":{},\"entrances_open\":{},\"released\":{},\"alive\":{},\"removed\":{},\"actors\":[",
        step.control_step,
        step.iteration,
        step.paused,
        s.seconds_remaining,
        s.clock_frame,
        s.glitch_pause_iterations,
        s.release_rate,
        s.spawn_countdown,
        s.entrances_open,
        s.released,
        actors.len() - removed,
        removed
    );
    for (i, actor) in actors.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let a = actor.state();
        let removal = a
            .removed
            .map_or_else(|| "null".to_owned(), |r| format!("\"{r:?}\""));
        write!(out, "{{\"id\":{},\"born_iteration\":{},\"x\":{},\"y\":{},\"direction\":{},\"action\":\"{:?}\",\"frame\":{},\"end_of_animation\":{},\"fallen\":{},\"removed\":{removal}}}",
            a.id, a.born_iteration, a.x, a.y, a.direction.delta(), a.action, a.frame, a.end_of_animation, a.fallen).unwrap();
    }
    out.push_str("],\"events\":[");
    for (i, event) in events.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write!(out, "\"{event}\"").unwrap();
    }
    out.push_str("]}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    fn completion_session(minutes: u16, total: u32, required: u32) -> Session {
        let a = crate::assets::Assets::synthetic_builder();
        let mut config = a.config;
        config.time_limit_minutes = minutes;
        config.total_lemmings = total;
        config.pause_glitch = false;
        let mut s = Session::new(config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_blocker(10, a.object_map).unwrap();
        s.configure_bomber(10, a.explosion_mask).unwrap();
        s.configure_completion(required).unwrap();
        s
    }

    #[test]
    fn completion_clock_pause_deadline_and_terminal_state_are_stable() {
        let mut s = completion_session(1, 100_000, 1);
        for _ in 0..16 {
            s.tick().unwrap();
        }
        s.queue(InputCommand::Pause).unwrap();
        for _ in 0..8 {
            s.tick().unwrap();
        }
        assert_eq!(
            (
                s.scheduler.state().seconds_remaining,
                s.scheduler.state().clock_frame
            ),
            (60, 16)
        );
        s.queue(InputCommand::Resume).unwrap();
        s.tick().unwrap();
        assert_eq!(
            (
                s.scheduler.state().seconds_remaining,
                s.scheduler.state().clock_frame
            ),
            (59, 0)
        );
        while s.scheduler.step_state().iteration < 1020 {
            s.tick().unwrap();
        }
        assert_eq!(s.scheduler.state().seconds_remaining, 0);
        assert!(s.result().is_none());
        s.tick().unwrap();
        assert_eq!(s.scheduler.step_state().iteration, 1021);
        assert_eq!(s.result().unwrap().reason, EndReason::TimeExpired);
        let before = (
            s.snapshot(),
            s.completion_snapshot(),
            s.bomber_snapshot(),
            s.inputs.len(),
        );
        for _ in 0..5 {
            s.tick().unwrap();
        }
        assert!(s.queue(InputCommand::Pause).is_err());
        assert_eq!(
            (
                s.snapshot(),
                s.completion_snapshot(),
                s.bomber_snapshot(),
                s.inputs.len()
            ),
            before
        );
    }

    #[test]
    fn deadline_actor_update_can_rescue_before_result_is_frozen() {
        let a = crate::assets::Assets::synthetic_exit(false);
        let mut config = a.config;
        config.time_limit_minutes = 0;
        config.total_lemmings = 2;
        let mut s = Session::new(config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_blocker(0, a.object_map).unwrap();
        for _ in 0..54 {
            s.tick().unwrap();
        }
        assert_eq!(s.actors.len(), 1);
        let mut state = s.actors[0].state();
        state.action = MotionAction::Exiting;
        state.frame = 7;
        state.fallen = 0;
        s.actors[0] = Actor::from_state(state, MotionRules::default()).unwrap();
        s.configure_completion(1).unwrap();
        s.tick().unwrap();
        let result = s.result().unwrap();
        assert_eq!(
            (result.reason, result.rescued, result.success),
            (EndReason::TimeExpired, 1, true)
        );
        assert_eq!(s.saved(), 1);
    }

    #[test]
    fn early_nuke_finishes_without_births_or_division_by_zero() {
        let mut s = completion_session(5, 80, 40);
        s.queue(InputCommand::Custom(StartupCommand::Nuke)).unwrap();
        s.tick().unwrap();
        let result = s.result().unwrap();
        assert_eq!(
            (
                result.reason,
                result.lemming_count,
                result.rescued_percent,
                result.success
            ),
            (EndReason::PopulationExhausted, 0, 0, false)
        );
        assert!(s.actors.is_empty());
    }

    #[test]
    fn completion_configuration_is_validated_and_cannot_replace_a_frozen_result() {
        let mut s = completion_session(0, 10, 5);
        assert!(s.configure_completion(11).is_err());
        s.tick().unwrap();
        let before = s.completion_snapshot();
        assert!(s.configure_completion(0).is_err());
        assert_eq!(s.completion_snapshot(), before);
    }

    #[test]
    fn trap_animation_freezes_on_pause_and_reuses_after_wrap() {
        let a = crate::assets::Assets::synthetic_hazards(0, false);
        let mut s = Session::new(a.config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_blocker(0, a.object_map).unwrap();
        s.configure_traps(a.traps);
        s.configure_bomber(0, [0; 44]).unwrap();
        while !s.traps()[0].unwrap().triggered() {
            s.tick().unwrap();
        }
        let before = s.traps()[0].unwrap();
        assert_eq!(before.frame(), 1);
        s.queue(InputCommand::Pause).unwrap();
        for _ in 0..5 {
            s.tick().unwrap();
            assert_eq!(s.traps()[0], Some(before));
        }
        s.queue(InputCommand::Resume).unwrap();
        for _ in 0..11 {
            s.tick().unwrap();
        }
        assert!(!s.traps()[0].unwrap().triggered());
        while s
            .actors
            .iter()
            .filter(|a| a.state().removed == Some(RemovalReason::Trapped))
            .count()
            < 2
        {
            s.tick().unwrap();
        }
        assert_eq!(s.saved(), 0);
    }
    #[test]
    fn nuke_arms_drowning_without_turning_death_into_a_rescue() {
        let a = crate::assets::Assets::synthetic_hazards(OBJECT_WATER, false);
        let mut config = a.config;
        config.total_lemmings = 1;
        let mut s = Session::new(config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_blocker(0, a.object_map).unwrap();
        s.configure_traps(a.traps);
        s.configure_bomber(0, [0; 44]).unwrap();
        while !s
            .actors
            .iter()
            .any(|a| a.state().action == MotionAction::Drowning)
        {
            s.tick().unwrap();
        }
        s.queue(InputCommand::Custom(StartupCommand::Nuke)).unwrap();
        s.tick().unwrap();
        assert_eq!(s.actors[0].state().explosion_timer, 79);
        for _ in 0..100 {
            s.tick().unwrap();
        }
        assert_eq!(s.actors[0].state().removed, Some(RemovalReason::Drowned));
        assert_eq!(s.saved(), 0);
    }

    #[test]
    fn paused_exit_keeps_animation_and_rescue_count_then_rescues_exactly_once() {
        let a = crate::assets::Assets::synthetic_exit(false);
        let mut config = a.config;
        config.total_lemmings = 1;
        let mut s = Session::new(config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_blocker(0, a.object_map).unwrap();
        s.configure_bomber(0, [0; 44]).unwrap();
        while !s
            .actors
            .iter()
            .any(|a| a.state().action == MotionAction::Exiting)
        {
            s.tick().unwrap();
        }
        let before = s.actors[0];
        s.queue(InputCommand::Pause).unwrap();
        for _ in 0..5 {
            s.tick().unwrap();
            assert_eq!(s.actors[0], before);
            assert_eq!(s.saved(), 0);
        }
        s.queue(InputCommand::Resume).unwrap();
        for _ in 0..8 {
            s.tick().unwrap();
        }
        assert_eq!(s.saved(), 1);
        for _ in 0..20 {
            s.tick().unwrap();
        }
        assert_eq!(s.saved(), 1);
        assert!(s.snapshot().contains("\"alive\":0,\"removed\":1"));
    }

    #[test]
    fn nuke_can_arm_exit_without_recounting_the_removed_survivor() {
        let a = crate::assets::Assets::synthetic_exit(false);
        let mut config = a.config;
        config.total_lemmings = 1;
        let mut s = Session::new(config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_blocker(0, a.object_map).unwrap();
        s.configure_bomber(0, [0; 44]).unwrap();
        while !s
            .actors
            .iter()
            .any(|a| a.state().action == MotionAction::Exiting)
        {
            s.tick().unwrap();
        }
        s.queue(InputCommand::Custom(StartupCommand::Nuke)).unwrap();
        s.tick().unwrap();
        assert_eq!(s.actors[0].state().explosion_timer, 79);
        for _ in 0..100 {
            s.tick().unwrap();
        }
        assert_eq!(s.saved(), 1);
        assert_eq!(s.remaining_bombers, 0);
        assert!(!s.scheduler.nuke_state().assigning);
    }
    fn session() -> Session {
        Session::new(
            StartupConfig {
                time_limit_minutes: 5,
                total_lemmings: 10,
                initial_release_rate: 50,
                entrance_count: 1,
                pause_glitch: true,
            },
            vec![EntrancePoint { x: 0, y: 0 }],
            96,
            160,
            vec![0; 96 * 160],
        )
        .unwrap()
    }
    #[test]
    fn nuke_arms_one_after_motion_then_pauses_and_preserves_all_inventories() {
        let a = crate::assets::Assets::synthetic_builder();
        let mut s = Session::new(a.config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_blocker(10, vec![128; DOS_OBJECT_MAP_WIDTH * DOS_OBJECT_MAP_HEIGHT])
            .unwrap();
        s.configure_bomber(0, [255; 44]).unwrap();
        for _ in 0..120 {
            s.tick().unwrap();
        }
        let released = s.scheduler.state().released;
        s.queue(InputCommand::Custom(StartupCommand::Nuke)).unwrap();
        s.tick().unwrap();
        assert_eq!(s.actors[0].state().explosion_timer, 79);
        assert_eq!(s.actors[1].state().explosion_timer, 0);
        s.queue(InputCommand::Pause).unwrap();
        s.tick().unwrap();
        assert_eq!(s.actors[0].state().explosion_timer, 79);
        assert_eq!(s.scheduler.nuke_state().next_index, 1);
        s.queue(InputCommand::Resume).unwrap();
        s.tick().unwrap();
        assert_eq!(s.actors[0].state().explosion_timer, 78);
        assert_eq!(s.actors[1].state().explosion_timer, 79);
        for _ in 0..150 {
            s.tick().unwrap();
        }
        assert_eq!(s.scheduler.state().released, released);
        assert_eq!(s.remaining_bombers, 0);
        assert_eq!(s.remaining_blockers, 10);
        assert!(s.actors.iter().all(|a| a.state().removed.is_some()));
    }
    #[test]
    fn paused_floater_assignment_consumes_once_and_survives_resume() {
        let a = crate::assets::Assets::synthetic_builder();
        let mut s = Session::new(a.config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_floater(2);
        for _ in 0..80 {
            s.tick().unwrap();
        }
        s.queue(InputCommand::Pause).unwrap();
        for _ in 0..2 {
            s.queue(InputCommand::Custom(StartupCommand::AssignFloater(0)))
                .unwrap();
        }
        s.tick().unwrap();
        let before = s.actors[0].state();
        assert!(before.is_floater);
        assert_eq!(s.remaining_floaters, 1);
        s.tick().unwrap();
        assert_eq!(s.actors[0].state(), before);
        s.queue(InputCommand::Resume).unwrap();
        s.tick().unwrap();
        assert!(s.actors[0].state().is_floater);
    }
    #[test]
    fn paused_climber_assignment_consumes_once_and_survives_resume() {
        let a = crate::assets::Assets::synthetic_builder();
        let mut s = Session::new(a.config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_climber(2);
        for _ in 0..80 {
            s.tick().unwrap();
        }
        s.queue(InputCommand::Pause).unwrap();
        for _ in 0..2 {
            s.queue(InputCommand::Custom(StartupCommand::AssignClimber(0)))
                .unwrap();
        }
        s.tick().unwrap();
        let before = s.actors[0].state();
        assert!(before.is_climber);
        assert_eq!(s.remaining_climbers, 1);
        s.tick().unwrap();
        assert_eq!(s.actors[0].state(), before);
        s.queue(InputCommand::Resume).unwrap();
        s.tick().unwrap();
        assert!(s.actors[0].state().is_climber);
    }
    #[test]
    fn paused_bomber_on_blocker_preserves_timer_then_restores_map_and_erases_terrain() {
        let a = crate::assets::Assets::synthetic_builder();
        let mut s = Session::new(a.config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_blocker(2, a.object_map).unwrap();
        s.configure_bomber(2, [255; 44]).unwrap();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        let map = s.object_map_data().to_vec();
        s.queue(InputCommand::Pause).unwrap();
        for command in [
            StartupCommand::AssignBlocker(0),
            StartupCommand::AssignBomber(0),
            StartupCommand::AssignBomber(0),
        ] {
            s.queue(InputCommand::Custom(command)).unwrap();
        }
        s.tick().unwrap();
        assert_eq!(s.remaining_bombers, 1);
        assert_eq!(s.actors[0].state().explosion_timer, 79);
        for _ in 0..5 {
            s.tick().unwrap();
        }
        assert_eq!(s.actors[0].state().explosion_timer, 79);
        let original = s.terrain().to_vec();
        s.queue(InputCommand::Resume).unwrap();
        for _ in 0..96 {
            s.tick().unwrap();
        }
        assert_eq!(s.actors[0].state().removed, Some(RemovalReason::Exploded));
        assert_eq!(s.object_map_data(), map);
        assert_ne!(s.terrain(), original);
        assert_ne!(s.render_terrain(), s.terrain());
        s.queue(InputCommand::Pause).unwrap();
        s.tick().unwrap();
        assert_ne!(s.render_terrain(), s.terrain());
        s.queue(InputCommand::Resume).unwrap();
        s.tick().unwrap();
        assert_eq!(s.render_terrain(), s.terrain());
        for _ in 0..52 {
            s.tick().unwrap();
        }
        assert_eq!(s.actors[0].state().particle_timer, 0);
    }
    #[test]
    fn paused_blocker_assignment_consumes_once_then_releases_after_support_is_removed() {
        let a = crate::assets::Assets::synthetic_builder();
        let mut s = Session::new(a.config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_blocker(2, a.object_map).unwrap();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        let before = s.object_map_data().to_vec();
        s.queue(InputCommand::Pause).unwrap();
        s.queue(InputCommand::Custom(StartupCommand::AssignBlocker(0)))
            .unwrap();
        s.queue(InputCommand::Custom(StartupCommand::AssignBlocker(0)))
            .unwrap();
        s.tick().unwrap();
        assert_eq!(s.remaining_blockers, 1);
        assert_eq!(s.actors[0].state().action, MotionAction::Blocking);
        assert_ne!(s.object_map_data(), before);
        let position = s.actors[0].state();
        s.terrain[position.y as usize * s.width + position.x as usize] = 0;
        s.queue(InputCommand::Resume).unwrap();
        s.tick().unwrap();
        assert_eq!(s.actors[0].state().action, MotionAction::Walking);
        assert_eq!(s.object_map_data(), before);
    }
    #[test]
    fn paused_builder_assignment_consumes_once_and_adds_material_only_after_resume() {
        let a = crate::assets::Assets::synthetic();
        let mut s = Session::new(a.config, a.entrances, a.width, a.height, a.terrain).unwrap();
        s.configure_builder(2);
        for _ in 0..80 {
            s.tick().unwrap();
        }
        s.queue(InputCommand::Pause).unwrap();
        s.queue(InputCommand::Custom(StartupCommand::AssignBuilder(0)))
            .unwrap();
        s.queue(InputCommand::Custom(StartupCommand::AssignBuilder(0)))
            .unwrap();
        let terrain = s.terrain().to_vec();
        s.tick().unwrap();
        assert_eq!(s.remaining_builders, 1);
        assert_eq!(
            (s.actors[0].state().action, s.actors[0].state().bricks_left),
            (MotionAction::Building, 12)
        );
        assert_eq!(s.terrain(), terrain);
        assert!(s.events.last().unwrap().ends_with("false"));
        s.queue(InputCommand::Resume).unwrap();
        for _ in 0..9 {
            s.tick().unwrap();
        }
        assert_eq!(s.brick_mask().iter().filter(|&&v| v == 1).count(), 6);
        assert!(s.terrain().iter().zip(terrain).any(|(&a, b)| a > b));
    }
    #[test]
    fn same_tick_dig_and_rebuild_changes_material_even_when_final_collision_is_unchanged() {
        let mut s = session();
        let digger = Actor::from_state(
            ActorState {
                id: 0,
                born_iteration: 0,
                x: 34,
                y: 40,
                direction: MotionDirection::Right,
                action: MotionAction::Digging,
                frame: 7,
                end_of_animation: false,
                fallen: 0,
                new_digger: false,
                bricks_left: 0,
                blocker_saved: None,
                is_climber: false,
                is_floater: false,
                float_index: 0,
                walking_once: false,
                explosion_timer: 0,
                particle_timer: 0,
                particle_frame: 0,
                object_below: 0,
                object_in_front: 0,
                removed: None,
            },
            MotionRules::default(),
        )
        .unwrap();
        let builder = Actor::from_state(
            ActorState {
                id: 1,
                x: 32,
                y: 41,
                action: MotionAction::Building,
                frame: 8,
                bricks_left: 12,
                ..digger.state()
            },
            digger.rules(),
        )
        .unwrap();
        s.actors = vec![digger, builder];
        let index = 40 * 96 + 34;
        s.terrain[index] = 1;
        assert_eq!(s.brick_mask[index], 0);
        s.tick().unwrap();
        assert_eq!((s.terrain[index], s.brick_mask[index]), (1, 1));
        s.actors[0] = Actor::from_state(
            ActorState {
                y: 41,
                frame: 0,
                new_digger: true,
                ..digger.state()
            },
            digger.rules(),
        )
        .unwrap();
        s.tick().unwrap();
        assert_eq!((s.terrain[index], s.brick_mask[index]), (0, 0));
    }
    #[test]
    fn paused_miner_assignment_keeps_motion_still_and_replays_inventory_and_excavation() {
        let assets = crate::assets::Assets::synthetic();
        let mut s = Session::new(
            assets.config,
            assets.entrances,
            assets.width,
            assets.height,
            assets.terrain,
        )
        .unwrap();
        s.configure_miner(2, assets.mine_masks);
        for _ in 0..80 {
            s.tick().unwrap();
        }
        s.queue(InputCommand::Pause).unwrap();
        s.queue(InputCommand::Custom(StartupCommand::AssignMiner(0)))
            .unwrap();
        s.queue(InputCommand::Custom(StartupCommand::AssignMiner(0)))
            .unwrap();
        let before = s.terrain().to_vec();
        s.tick().unwrap();
        assert_eq!(s.remaining_miners, 1);
        assert_eq!(
            (s.actors[0].state().action, s.actors[0].state().frame),
            (MotionAction::Mining, 0)
        );
        assert_eq!(s.terrain(), before);
        assert!(s.events.last().unwrap().ends_with("false"));
        s.queue(InputCommand::Resume).unwrap();
        s.tick().unwrap();
        s.tick().unwrap();
        assert!(s.terrain().iter().zip(before).any(|(&a, b)| a != b));
    }
    #[test]
    fn paused_basher_assignment_replays_inventory_and_mutable_terrain() {
        let assets = crate::assets::Assets::synthetic_basher();
        let mut s = Session::new(
            assets.config,
            assets.entrances,
            assets.width,
            assets.height,
            assets.terrain,
        )
        .unwrap();
        s.configure_basher(2, assets.bash_masks, vec![0; 640 * 160], vec![0; 640 * 160])
            .unwrap();
        for _ in 0..80 {
            s.tick().unwrap();
        }
        s.queue(InputCommand::Pause).unwrap();
        s.queue(InputCommand::Custom(StartupCommand::AssignBasher(0)))
            .unwrap();
        s.queue(InputCommand::Custom(StartupCommand::AssignBasher(0)))
            .unwrap();
        let before = s.terrain().to_vec();
        s.tick().unwrap();
        assert_eq!(s.remaining_bashers, 1);
        assert_eq!(s.actors[0].state().frame, 0);
        assert_eq!(s.terrain(), before);
        assert!(s.events.last().unwrap().ends_with("false"));
        s.queue(InputCommand::Resume).unwrap();
        for _ in 0..5 {
            s.tick().unwrap();
        }
        assert!(s.terrain().iter().zip(before).any(|(&a, b)| a != b));
    }
    #[test]
    fn birth_updates_in_same_tick_and_removed_records_survive() {
        let mut s = session();
        for _ in 0..54 {
            s.tick().unwrap();
        }
        assert_eq!((s.actors[0].state().y, s.actors[0].state().fallen), (17, 6));
        for _ in 54..340 {
            s.tick().unwrap();
        }
        assert_eq!(s.actors.len(), 10);
        assert_eq!(
            s.actors
                .iter()
                .filter(|a| a.state().removed.is_some())
                .count(),
            9
        );
        let removed = s.actors[0];
        for _ in 340..400 {
            s.tick().unwrap();
        }
        assert_eq!(s.actors[0], removed);
        assert!(s.actors.iter().all(|a| a.state().removed.is_some()));
    }
    #[test]
    fn same_boundary_pause_resume_order_and_invalid_rate_are_recorded() {
        let mut s = session();
        s.queue(InputCommand::Pause).unwrap();
        s.queue(InputCommand::Resume).unwrap();
        s.queue(InputCommand::Custom(StartupCommand::SetReleaseRate(49)))
            .unwrap();
        s.tick().unwrap();
        assert_eq!(s.scheduler.step_state().iteration, 1);
        assert_eq!(s.scheduler.state().release_rate, 50);
        assert_eq!(
            s.events,
            vec!["Input:0:0:true", "Input:0:1:true", "Input:0:2:false"]
        );
        assert_eq!(s.inputs.len(), 3);
    }
    #[test]
    fn paused_control_calls_keep_clock_and_actors_still() {
        let mut s = session();
        for _ in 0..60 {
            s.tick().unwrap();
        }
        let before = s.actors.clone();
        s.queue(InputCommand::Pause).unwrap();
        for _ in 0..20 {
            s.tick().unwrap();
        }
        assert_eq!(s.actors, before);
        assert_eq!(s.scheduler.step_state().iteration, 60);
        s.queue(InputCommand::Resume).unwrap();
        s.tick().unwrap();
        assert_eq!(s.scheduler.step_state().iteration, 61);
    }
    #[test]
    fn elapsed_partitions_produce_identical_complete_state() {
        let mut a = session();
        let mut b = session();
        a.advance(Duration::from_secs(20)).unwrap();
        for _ in 0..20000 {
            b.advance(Duration::from_millis(1)).unwrap();
        }
        assert_eq!(a.snapshot(), b.snapshot());
        assert_eq!(a.scheduler.step_state().iteration, 340);
    }
    #[test]
    fn pacing_changes_delivery_without_changing_control_step_replay() {
        let mut runs = Vec::new();
        for pacing in [Pacing::Dos, Pacing::Reference] {
            let mut s = session();
            s.set_pacing(pacing);
            let mut snapshots = vec![s.snapshot()];
            let mut milliseconds = 0;
            while s.scheduler.step_state().control_step < 340 {
                milliseconds += 1;
                for _ in 0..s.due(Duration::from_millis(1)).unwrap() {
                    match s.scheduler.step_state().control_step {
                        16 => s.queue(InputCommand::Pause).unwrap(),
                        34 => s.queue(InputCommand::Resume).unwrap(),
                        54 => s
                            .queue(InputCommand::Custom(StartupCommand::SetReleaseRate(99)))
                            .unwrap(),
                        _ => (),
                    }
                    s.tick().unwrap();
                    snapshots.push(s.snapshot());
                }
            }
            runs.push((milliseconds, snapshots));
        }
        assert!(runs[0].0 > runs[1].0);
        assert_eq!(runs[0].1, runs[1].1);
        assert_eq!(runs[0].1.len(), 341);
    }
    #[test]
    fn digging_replays_keep_every_actor_and_terrain_byte_across_render_patterns() {
        let mut runs = Vec::new();
        for pacing in [Pacing::Dos, Pacing::Reference] {
            let mut s = session();
            s.terrain[30 * 96..].fill(1);
            s.configure_digger(2, vec![0; 96 * 160]).unwrap();
            s.set_pacing(pacing);
            let mut states = Vec::new();
            while s.scheduler.step_state().control_step < 180 {
                for _ in 0..s.due(Duration::from_millis(1)).unwrap() {
                    if s.scheduler.step_state().control_step >= 180 {
                        break;
                    }
                    if s.scheduler.step_state().control_step == 60 {
                        s.queue(InputCommand::Custom(StartupCommand::AssignDigger(0)))
                            .unwrap();
                        s.queue(InputCommand::Custom(StartupCommand::AssignDigger(0)))
                            .unwrap();
                    }
                    s.tick().unwrap();
                    states.push((s.snapshot(), s.digger_snapshot(), s.terrain.clone()));
                }
            }
            assert_eq!(s.remaining_diggers, 1);
            assert!(s.terrain.iter().filter(|&&v| v == 1).count() < 96 * 130);
            runs.push(states);
        }
        assert_eq!(runs[0], runs[1]);
    }
    #[test]
    fn missing_nonbinary_terrain_and_entrance_mismatch_fail_at_load() {
        let s = session();
        let c = s.scheduler.config();
        assert!(Session::new(c, vec![], 96, 160, vec![0; 96 * 160]).is_err());
        assert!(Session::new(c, vec![EntrancePoint { x: 0, y: 0 }], 96, 160, vec![]).is_err());
        assert!(
            Session::new(
                c,
                vec![EntrancePoint { x: 0, y: 0 }],
                96,
                160,
                vec![2; 96 * 160]
            )
            .is_err()
        );
    }
}
