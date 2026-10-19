extern crate std;
use super::*;
use crate::{FixedTickClock, TimedInput};
use core::time::Duration;
use std::{vec, vec::Vec};

#[test]
fn nuke_input_at_control_overflow_changes_neither_nuke_nor_startup_state() {
    let mut s = StartupScheduler::new(config()).unwrap();
    s.stepper = TickStepper::from_state(StepState {
        control_step: u64::MAX,
        iteration: 80,
        paused: false,
    })
    .unwrap();
    let before = s;
    let inputs = [input(
        u64::MAX,
        0,
        InputCommand::Custom(StartupCommand::Nuke),
    )];
    assert!(
        s.step(&InputSchedule::new(&inputs).unwrap(), |_| panic!(
            "no overflow callback"
        ))
        .is_err()
    );
    assert_eq!(s, before);
}

fn config() -> StartupConfig {
    StartupConfig {
        time_limit_minutes: 5,
        total_lemmings: 10,
        initial_release_rate: 50,
        entrance_count: 1,
        pause_glitch: true,
    }
}
fn input(
    step: u64,
    sequence: u32,
    command: InputCommand<StartupCommand>,
) -> TimedInput<StartupCommand> {
    TimedInput {
        control_step: step,
        sequence,
        command,
    }
}
fn rate(step: u64, sequence: u32, value: u8) -> TimedInput<StartupCommand> {
    input(
        step,
        sequence,
        InputCommand::Custom(StartupCommand::SetReleaseRate(value)),
    )
}
fn run(
    startup: &mut StartupScheduler,
    inputs: &[TimedInput<StartupCommand>],
    count: usize,
) -> Vec<StartupEvent> {
    let schedule = InputSchedule::new(inputs).unwrap();
    let mut events = Vec::new();
    for _ in 0..count {
        startup.step(&schedule, |e| events.push(e)).unwrap();
    }
    events
}
fn births(events: &[StartupEvent]) -> Vec<SpawnRequest> {
    events
        .iter()
        .filter_map(|e| {
            if let StartupEvent::Spawn(s) = e {
                Some(*s)
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn initial_state_preserves_config_and_accepts_extreme_counts_and_time() {
    let cfg = StartupConfig {
        time_limit_minutes: u16::MAX,
        total_lemmings: u32::MAX,
        ..config()
    };
    let startup = StartupScheduler::new(cfg).unwrap();
    assert_eq!(startup.config(), cfg);
    assert_eq!(startup.step_state(), StepState::default());
    assert_eq!(
        startup.state(),
        StartupState {
            seconds_remaining: 3_932_100,
            clock_frame: 0,
            glitch_pause_iterations: 0,
            release_rate: 50,
            spawn_countdown: 20,
            entrances_open: false,
            released: 0
        }
    );
}

#[test]
fn invalid_config_reports_rate_before_entrances_and_rejects_all_outside_values() {
    for value in 100..=255 {
        assert_eq!(
            StartupScheduler::new(StartupConfig {
                initial_release_rate: value,
                entrance_count: 255,
                ..config()
            }),
            Err(StartupConfigError::ReleaseRate(value))
        );
    }
    for value in 5..=255 {
        assert_eq!(
            StartupScheduler::new(StartupConfig {
                entrance_count: value,
                ..config()
            }),
            Err(StartupConfigError::EntranceCount(value))
        );
    }
}

#[test]
fn clock_decrements_on_seventeen_and_borrows_minutes_without_rounding() {
    let mut s = StartupScheduler::new(config()).unwrap();
    run(&mut s, &[], 16);
    assert_eq!(
        (s.state().seconds_remaining, s.state().clock_frame),
        (300, 16)
    );
    run(&mut s, &[], 1);
    assert_eq!(
        (s.state().seconds_remaining, s.state().clock_frame),
        (299, 0)
    );
    run(&mut s, &[], 17 * 59);
    assert_eq!(
        (s.state().seconds_remaining, s.state().clock_frame),
        (240, 0)
    );
}

#[test]
fn clock_saturates_at_zero_while_game_termination_remains_a_caller_duty() {
    for minutes in [0, 1] {
        let mut s = StartupScheduler::new(StartupConfig {
            time_limit_minutes: minutes,
            total_lemmings: 0,
            ..config()
        })
        .unwrap();
        run(&mut s, &[], 17 * 61);
        assert_eq!(s.state().seconds_remaining, 0);
        assert_eq!(s.state().clock_frame, 0);
        assert_eq!(s.step_state().iteration, 17 * 61);
    }
}

#[test]
fn f00_startup_markers_precede_ticks_and_first_birth_is_iteration_fifty_four() {
    let mut s = StartupScheduler::new(config()).unwrap();
    let events = run(&mut s, &[], 54);
    let markers: Vec<_> = events
        .iter()
        .filter(|e| !matches!(e, StartupEvent::Tick(_)))
        .copied()
        .collect();
    assert_eq!(
        markers,
        [
            StartupEvent::LetsGo,
            StartupEvent::EntranceSound,
            StartupEvent::EntrancesOpened,
            StartupEvent::Spawn(SpawnRequest {
                actor_id: 0,
                entrance_index: 0,
                iteration: 54
            })
        ]
    );
    for (marker, iteration) in [
        (StartupEvent::LetsGo, 15),
        (StartupEvent::EntranceSound, 34),
        (StartupEvent::EntrancesOpened, 35),
    ] {
        let i = events.iter().position(|e| *e == marker).unwrap();
        assert_eq!(events[i + 1], StartupEvent::Tick(iteration));
    }
    assert_eq!(events[events.len() - 1], StartupEvent::Tick(54));
    assert_eq!(s.state().spawn_countdown, 28);
}

#[test]
fn entrance_opening_starts_the_twenty_tick_countdown_on_the_same_tick() {
    let mut s = StartupScheduler::new(config()).unwrap();
    run(&mut s, &[], 34);
    assert!(!s.state().entrances_open);
    assert_eq!(s.state().spawn_countdown, 20);
    run(&mut s, &[], 1);
    assert!(s.state().entrances_open);
    assert_eq!(s.state().spawn_countdown, 19);
    run(&mut s, &[], 18);
    assert_eq!((s.state().released, s.state().spawn_countdown), (0, 1));
    run(&mut s, &[], 1);
    assert_eq!(s.state().released, 1);
}

#[test]
fn every_initial_rate_has_the_same_first_birth_and_the_reference_integer_interval() {
    for rr in 0..=99 {
        let mut s = StartupScheduler::new(StartupConfig {
            initial_release_rate: rr,
            total_lemmings: 2,
            ..config()
        })
        .unwrap();
        let events = run(&mut s, &[], 110);
        let b = births(&events);
        assert_eq!(b.len(), 2);
        assert_eq!(b[0].iteration, 54);
        assert_eq!(b[1].iteration, 54 + u64::from((99 - rr) / 2 + 4));
    }
}

#[test]
fn original_entrance_orders_and_stable_ids_repeat_without_skipping() {
    let orders = [[0, 0, 0, 0], [0, 1, 1, 0], [0, 1, 2, 1], [0, 1, 2, 3]];
    for count in 1..=4 {
        let mut s = StartupScheduler::new(StartupConfig {
            initial_release_rate: 99,
            entrance_count: count,
            total_lemmings: 8,
            ..config()
        })
        .unwrap();
        let b = births(&run(&mut s, &[], 90));
        assert_eq!(b.len(), 8);
        for (i, birth) in b.iter().enumerate() {
            assert_eq!(birth.actor_id, i as u32);
            assert_eq!(birth.entrance_index, orders[usize::from(count - 1)][i % 4]);
            assert_eq!(birth.iteration, 54 + i as u64 * 4);
        }
    }
}

#[test]
fn absent_entrances_and_zero_total_never_birth_but_keep_bounded_countdowns() {
    for (entrances, total) in [(0, 10), (4, 0)] {
        let mut s = StartupScheduler::new(StartupConfig {
            entrance_count: entrances,
            total_lemmings: total,
            ..config()
        })
        .unwrap();
        assert!(births(&run(&mut s, &[], 1000)).is_empty());
        assert_eq!(s.state().released, 0);
        assert!((1..=53).contains(&s.state().spawn_countdown));
        assert!(s.state().entrances_open);
        assert_eq!(s.step_state().control_step, 1000);
    }
}

#[test]
fn f02_rate_ninety_eight_and_ninety_nine_preserve_the_running_countdown() {
    for rr in [98, 99] {
        let mut s = StartupScheduler::new(config()).unwrap();
        let inputs = [rate(54, 0, rr)];
        let b = births(&run(&mut s, &inputs, 91));
        assert_eq!(
            b.iter().map(|b| b.iteration).collect::<Vec<_>>(),
            [54, 82, 86, 90]
        );
        assert_eq!(s.state().release_rate, rr);
    }
}

#[test]
fn rate_at_a_birth_boundary_changes_its_reload_and_invalid_inputs_are_acknowledged() {
    let mut s = StartupScheduler::new(config()).unwrap();
    let inputs = [rate(53, 0, 99), rate(53, 1, 49), rate(53, 2, 255)];
    let events = run(&mut s, &inputs, 59);
    let i = events
        .iter()
        .position(|e| matches!(e, StartupEvent::Input { .. }))
        .unwrap();
    assert_eq!(
        &events[i..i + 3],
        &[
            StartupEvent::Input {
                control_step: 53,
                sequence: 0,
                accepted: true
            },
            StartupEvent::Input {
                control_step: 53,
                sequence: 1,
                accepted: false
            },
            StartupEvent::Input {
                control_step: 53,
                sequence: 2,
                accepted: false
            }
        ]
    );
    assert_eq!(
        births(&events)
            .iter()
            .map(|b| b.iteration)
            .collect::<Vec<_>>(),
        [54, 58]
    );
    assert_eq!(s.state().release_rate, 99);
}

#[test]
fn ordered_rates_apply_while_paused_and_do_not_reset_startup_or_advance_time() {
    let mut s = StartupScheduler::new(config()).unwrap();
    let inputs = [
        input(0, 0, InputCommand::Pause),
        rate(0, 1, 99),
        rate(0, 2, 255),
        rate(0, 3, 98),
        input(0, 4, InputCommand::Pause),
    ];
    let events = run(&mut s, &inputs, 10);
    assert_eq!(events.len(), 5);
    assert!(matches!(
        events[2],
        StartupEvent::Input {
            accepted: false,
            ..
        }
    ));
    assert_eq!(
        s.step_state(),
        StepState {
            control_step: 10,
            iteration: 0,
            paused: true
        }
    );
    assert_eq!(
        (
            s.state().release_rate,
            s.state().spawn_countdown,
            s.state().clock_frame,
            s.state().glitch_pause_iterations
        ),
        (98, 20, 0, 34)
    );
}

#[test]
fn f03_pause_glitch_freezes_clock_after_resume_but_keeps_logical_entrance_and_birth_ticks() {
    let inputs = [
        input(16, 0, InputCommand::Pause),
        input(34, 0, InputCommand::Resume),
    ];
    for glitch in [false, true] {
        let mut s = StartupScheduler::new(StartupConfig {
            pause_glitch: glitch,
            ..config()
        })
        .unwrap();
        run(&mut s, &inputs, 34);
        assert_eq!(
            s.step_state(),
            StepState {
                control_step: 34,
                iteration: 16,
                paused: true
            }
        );
        assert_eq!(
            s.state().glitch_pause_iterations,
            if glitch { 18 } else { 0 }
        );
        run(&mut s, &inputs, 1);
        assert_eq!(s.step_state().iteration, 17);
        assert_eq!(
            (s.state().seconds_remaining, s.state().clock_frame),
            if glitch { (300, 16) } else { (299, 0) }
        );
        run(&mut s, &inputs, 17);
        assert_eq!(s.step_state().iteration, 34);
        assert_eq!(s.state().glitch_pause_iterations, 0);
        let events = run(&mut s, &inputs, 20);
        assert_eq!(
            births(&events),
            [SpawnRequest {
                actor_id: 0,
                entrance_index: 0,
                iteration: 54
            }]
        );
        assert_eq!(s.step_state().control_step, 72);
        assert_eq!(
            (s.state().seconds_remaining, s.state().clock_frame),
            if glitch { (298, 2) } else { (297, 3) }
        );
    }
}

#[test]
fn pause_glitch_arms_only_on_an_early_transition_and_consumes_only_running_ticks() {
    for (boundary, armed) in [(0, 34), (33, 1), (34, 0), (35, 0)] {
        let mut s = StartupScheduler::new(config()).unwrap();
        let inputs = [
            input(boundary, 0, InputCommand::Pause),
            input(boundary, 1, InputCommand::Pause),
            input(boundary + 3, 0, InputCommand::Resume),
        ];
        run(&mut s, &inputs, boundary as usize + 3);
        assert_eq!(s.state().glitch_pause_iterations, armed);
        let clock = s.state().clock_frame;
        run(&mut s, &inputs, 1);
        assert_eq!(s.state().glitch_pause_iterations, armed.saturating_sub(1));
        if armed > 0 {
            assert_eq!(s.state().clock_frame, clock);
        }
    }
}

#[test]
fn final_u32_actor_id_and_total_cap_never_wrap() {
    let mut s = StartupScheduler::new(StartupConfig {
        total_lemmings: u32::MAX,
        entrance_count: 4,
        initial_release_rate: 99,
        ..config()
    })
    .unwrap();
    s.state.released = u32::MAX - 1;
    s.state.entrances_open = true;
    s.state.spawn_countdown = 1;
    s.stepper = TickStepper::from_state(StepState {
        control_step: 53,
        iteration: 53,
        paused: false,
    })
    .unwrap();
    let b = births(&run(&mut s, &[], 10));
    assert_eq!(
        b,
        [SpawnRequest {
            actor_id: u32::MAX - 1,
            entrance_index: 2,
            iteration: 54
        }]
    );
    assert_eq!(s.state().released, u32::MAX);
}

#[test]
fn control_overflow_preserves_all_scheduler_state_without_input_or_tick_callbacks() {
    let mut s = StartupScheduler::new(config()).unwrap();
    s.stepper = TickStepper::from_state(StepState {
        control_step: u64::MAX,
        iteration: 0,
        paused: false,
    })
    .unwrap();
    let before = s;
    let inputs = [
        rate(u64::MAX, 0, 99),
        input(u64::MAX, 1, InputCommand::Pause),
    ];
    let mut calls = 0;
    assert_eq!(
        s.step(&InputSchedule::new(&inputs).unwrap(), |_| calls += 1),
        Err(StepOverflow)
    );
    assert_eq!(calls, 0);
    assert_eq!(s, before);
}

fn push_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn push_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn event_bytes(out: &mut Vec<u8>, event: StartupEvent) {
    match event {
        StartupEvent::Input {
            control_step,
            sequence,
            accepted,
        } => {
            out.push(0);
            push_u64(out, control_step);
            push_u32(out, sequence);
            out.push(u8::from(accepted));
        }
        StartupEvent::LetsGo => out.push(1),
        StartupEvent::EntranceSound => out.push(2),
        StartupEvent::EntrancesOpened => out.push(3),
        StartupEvent::Spawn(s) => {
            out.push(4);
            push_u32(out, s.actor_id);
            out.push(s.entrance_index);
            push_u64(out, s.iteration);
        }
        StartupEvent::Tick(i) => {
            out.push(5);
            push_u64(out, i);
        }
    }
}
fn snapshot_bytes(out: &mut Vec<u8>, step: StepState, state: StartupState) {
    out.push(6);
    push_u64(out, step.control_step);
    push_u64(out, step.iteration);
    out.push(u8::from(step.paused));
    push_u32(out, state.seconds_remaining);
    out.extend_from_slice(&[
        state.clock_frame,
        state.glitch_pause_iterations,
        state.release_rate,
        state.spawn_countdown,
        u8::from(state.entrances_open),
    ]);
    push_u32(out, state.released);
}
fn inputs(scenario: u8) -> Vec<TimedInput<StartupCommand>> {
    match scenario {
        1 => vec![rate(54, 0, 99)],
        2 => vec![rate(54, 0, 98)],
        3 => vec![
            input(16, 0, InputCommand::Pause),
            input(34, 0, InputCommand::Resume),
        ],
        _ => vec![],
    }
}

// Closed-form expectations: deadlines and clock frames are derived from absolute
// iteration counts, not by decrementing a second scheduler's mutable countdown.
fn expected(rr: u8, entrances: u8, glitch: bool, scenario: u8) -> Vec<u8> {
    let mut out = vec![7, rr, entrances, u8::from(glitch), scenario];
    let interval0 = u64::from((99 - rr) / 2 + 4);
    let candidate = match scenario {
        1 => 99,
        2 => 98,
        _ => rr,
    };
    let accepted = candidate >= rr;
    let interval1 = u64::from((99 - if accepted { candidate } else { rr }) / 2 + 4);
    let next_initial = 54 + interval0;
    let (mut previous_iteration, mut previous_released) = (0, 0u32);
    for control in 1..=340u64 {
        let boundary = control - 1;
        let paused = scenario == 3 && (16..34).contains(&boundary);
        let iteration = if scenario == 3 {
            if control <= 34 {
                control.min(16)
            } else {
                control - 18
            }
        } else {
            control
        };
        if (scenario == 1 || scenario == 2) && boundary == 54 {
            event_bytes(
                &mut out,
                StartupEvent::Input {
                    control_step: boundary,
                    sequence: 0,
                    accepted,
                },
            );
        }
        if scenario == 3 && (boundary == 16 || boundary == 34) {
            event_bytes(
                &mut out,
                StartupEvent::Input {
                    control_step: boundary,
                    sequence: 0,
                    accepted: true,
                },
            );
        }
        let rate = if (scenario == 1 || scenario == 2) && boundary >= 54 && accepted {
            candidate
        } else {
            rr
        };
        let lost = if scenario == 3 && glitch {
            iteration.saturating_sub(16).min(18)
        } else {
            0
        };
        let clock_ticks = iteration - lost;
        let remaining_glitch = if scenario == 3 && glitch && boundary >= 16 {
            34u64.saturating_sub(iteration) as u8
        } else {
            0
        };
        let countdown = if iteration < 35 {
            20
        } else if iteration < 54 {
            54 - iteration
        } else if iteration < next_initial {
            next_initial - iteration
        } else {
            next_initial + ((iteration - next_initial) / interval1 + 1) * interval1 - iteration
        };
        let opportunities = if iteration < 54 {
            0
        } else {
            1 + if iteration < next_initial {
                0
            } else {
                1 + (iteration - next_initial) / interval1
            }
        };
        let released = if entrances > 0 {
            opportunities.min(10) as u32
        } else {
            0
        };
        if iteration != previous_iteration {
            match iteration {
                15 => event_bytes(&mut out, StartupEvent::LetsGo),
                34 => event_bytes(&mut out, StartupEvent::EntranceSound),
                35 => event_bytes(&mut out, StartupEvent::EntrancesOpened),
                _ => {}
            }
            if released > previous_released {
                let orders = [
                    [0, 0, 0, 0],
                    [0, 0, 0, 0],
                    [0, 1, 1, 0],
                    [0, 1, 2, 1],
                    [0, 1, 2, 3],
                ];
                event_bytes(
                    &mut out,
                    StartupEvent::Spawn(SpawnRequest {
                        actor_id: previous_released,
                        entrance_index: orders[usize::from(entrances)]
                            [(previous_released % 4) as usize],
                        iteration,
                    }),
                );
            }
            event_bytes(&mut out, StartupEvent::Tick(iteration));
        }
        snapshot_bytes(
            &mut out,
            StepState {
                control_step: control,
                iteration,
                paused,
            },
            StartupState {
                seconds_remaining: 300 - (clock_ticks / 17) as u32,
                clock_frame: (clock_ticks % 17) as u8,
                glitch_pause_iterations: remaining_glitch,
                release_rate: rate,
                spawn_countdown: countdown as u8,
                entrances_open: iteration >= 35,
                released,
            },
        );
        previous_iteration = iteration;
        previous_released = released;
    }
    out
}

#[test]
fn exhaustive_startup_replays_match_closed_form_model_and_python_stream() {
    let sixty_hz: Vec<_> = (0..1200u64)
        .map(|i| (i + 1) * 1_000_000_000 / 60 - i * 1_000_000_000 / 60)
        .collect();
    let patterns = [
        vec![20_000_000_000],
        sixty_hz,
        [
            0,
            1,
            58_823_528,
            1,
            240_000_001,
            0,
            123_456_789,
            577_719_680,
        ]
        .repeat(20),
    ];
    for frames in &patterns {
        assert_eq!(frames.iter().sum::<u64>(), 20_000_000_000);
    }
    let mut cases = 0;
    let mut signature = 0xcbf29ce484222325u64;
    for rr in 0..=99 {
        for entrances in 0..=4 {
            for glitch in [false, true] {
                for scenario in 0..=3 {
                    let wanted = expected(rr, entrances, glitch, scenario);
                    let inputs = inputs(scenario);
                    let schedule = InputSchedule::new(&inputs).unwrap();
                    for frames in &patterns {
                        let mut s = StartupScheduler::new(StartupConfig {
                            initial_release_rate: rr,
                            entrance_count: entrances,
                            pause_glitch: glitch,
                            ..config()
                        })
                        .unwrap();
                        let mut clock = FixedTickClock::new();
                        let mut actual = vec![7, rr, entrances, u8::from(glitch), scenario];
                        for &ns in frames {
                            for _ in 0..clock.advance(Duration::from_nanos(ns)).unwrap() {
                                s.step(&schedule, |e| event_bytes(&mut actual, e)).unwrap();
                                snapshot_bytes(&mut actual, s.step_state(), s.state());
                            }
                        }
                        assert_eq!(
                            actual, wanted,
                            "rr={rr}, entrances={entrances}, glitch={glitch}, scenario={scenario}"
                        );
                        assert_eq!(clock.phase_numerator(), 0);
                        assert_eq!(s.step_state().control_step, 340);
                    }
                    for byte in wanted {
                        signature = (signature ^ u64::from(byte)).wrapping_mul(0x100000001b3);
                    }
                    cases += 1;
                }
            }
        }
    }
    assert_eq!(cases, 4000);
    assert_eq!(signature, 0x81365042e5aa5fed);
    std::println!(
        "4000 scenarios / 12000 replays / 4080000 control calls / fnv1a64={signature:016x}"
    );
}
