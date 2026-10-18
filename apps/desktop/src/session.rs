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
        let mut events = Vec::new();
        let mut spawn = None;
        let schedule = InputSchedule::new(&self.inputs)?;
        let steel = TerrainMask::new(self.width, self.height, &self.steel)?;
        let actors = &mut self.actors;
        let remaining = &mut self.remaining_diggers;
        let bashers = &mut self.remaining_bashers;
        let context = BashingContext {
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
        let outcome = self.scheduler.step_with_skills(
            &schedule,
            |command| {
                let id = match command {
                    StartupCommand::AssignDigger(id) | StartupCommand::AssignBasher(id) => id,
                    _ => return false,
                };
                actors
                    .iter_mut()
                    .find(|a| a.state().id == id)
                    .is_some_and(|a| match command {
                        StartupCommand::AssignDigger(_) => a.assign_digger(remaining, steel),
                        StartupCommand::AssignBasher(_) => a.assign_basher(bashers, context),
                        _ => false,
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
            step_actors_with_skills(
                &mut self.actors,
                &mut TerrainMaskMut::new(self.width, self.height, &mut self.terrain)?,
                context,
                |_, _| {},
            )?;
        }
        self.events = events;
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
