use lemmings_core::{CompletionError, CompletionProgress, CompletionRules, EndReason};

fn progress() -> CompletionProgress {
    CompletionProgress {
        seconds_before_tick: 1,
        released: 10,
        removed: 0,
        rescued: 0,
        max_lemmings: 10,
        nuked: false,
        particle_timer: 0,
    }
}

#[test]
fn quota_does_not_finish_while_an_actor_remains() {
    let rules = CompletionRules::new(10, 5).unwrap();
    assert_eq!(
        rules.evaluate(CompletionProgress {
            removed: 5,
            rescued: 5,
            ..progress()
        }),
        Ok(None)
    );
    assert_eq!(
        rules.evaluate(CompletionProgress {
            released: 5,
            removed: 5,
            rescued: 5,
            ..progress()
        }),
        Ok(None)
    );
}

#[test]
fn exhausting_population_wins_or_loses_by_integer_percentage() {
    let rules = CompletionRules::new(10, 5).unwrap();
    for saved in 0..=10 {
        let result = rules
            .evaluate(CompletionProgress {
                removed: 10,
                rescued: saved,
                ..progress()
            })
            .unwrap()
            .unwrap();
        assert_eq!(result.reason, EndReason::PopulationExhausted);
        assert_eq!(result.rescued_percent, saved * 10);
        assert_eq!(result.target_percent, 50);
        assert_eq!(result.success, saved >= 5);
    }
}

#[test]
fn timeout_can_succeed_and_does_not_wait_for_explosion_particles() {
    let rules = CompletionRules::new(10, 5).unwrap();
    for rescued in [4, 5] {
        let result = rules
            .evaluate(CompletionProgress {
                seconds_before_tick: 0,
                removed: rescued,
                rescued,
                particle_timer: 52,
                ..progress()
            })
            .unwrap()
            .unwrap();
        assert_eq!(result.reason, EndReason::TimeExpired);
        assert_eq!(result.success, rescued == 5);
    }
}

#[test]
fn last_particles_delay_population_finish_and_exhaustion_precedes_timeout() {
    let rules = CompletionRules::new(10, 5).unwrap();
    assert_eq!(
        rules.evaluate(CompletionProgress {
            removed: 10,
            particle_timer: 1,
            ..progress()
        }),
        Ok(None)
    );
    let result = rules
        .evaluate(CompletionProgress {
            seconds_before_tick: 0,
            removed: 10,
            ..progress()
        })
        .unwrap()
        .unwrap();
    assert_eq!(result.reason, EndReason::PopulationExhausted);
}

#[test]
fn nuke_changes_done_denominator_but_keeps_original_target() {
    let rules = CompletionRules::new(80, 40).unwrap();
    let p = CompletionProgress {
        released: 2,
        removed: 2,
        rescued: 1,
        max_lemmings: 2,
        nuked: true,
        ..progress()
    };
    let result = rules.evaluate(p).unwrap().unwrap();
    assert_eq!(result.lemming_count, 2);
    assert_eq!(result.rescued, 1);
    assert_eq!(
        (
            result.rescued_percent,
            result.target_percent,
            result.success
        ),
        (50, 50, true)
    );
    let result = rules
        .evaluate(CompletionProgress {
            max_lemmings: 80,
            ..p
        })
        .unwrap()
        .unwrap();
    assert_eq!((result.rescued_percent, result.success), (1, false));
}

#[test]
fn zero_population_and_early_nuke_are_defined_without_division() {
    let result = CompletionRules::new(0, 0)
        .unwrap()
        .evaluate(CompletionProgress {
            released: 0,
            removed: 0,
            max_lemmings: 0,
            ..progress()
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            result.target_percent,
            result.rescued_percent,
            result.success
        ),
        (0, 0, true)
    );
    let result = CompletionRules::new(80, 40)
        .unwrap()
        .evaluate(CompletionProgress {
            released: 0,
            removed: 0,
            max_lemmings: 0,
            nuked: true,
            ..progress()
        })
        .unwrap()
        .unwrap();
    assert_eq!((result.rescued_percent, result.success), (0, false));
}

#[test]
fn both_target_and_result_are_floored_before_comparison() {
    let rules = CompletionRules::new(200, 3).unwrap();
    let result = rules
        .evaluate(CompletionProgress {
            released: 200,
            removed: 200,
            max_lemmings: 200,
            rescued: 2,
            ..progress()
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            result.target_percent,
            result.rescued_percent,
            result.success
        ),
        (1, 1, true)
    );
    let result = CompletionRules::new(200, 1)
        .unwrap()
        .evaluate(CompletionProgress {
            released: 200,
            removed: 200,
            max_lemmings: 200,
            ..progress()
        })
        .unwrap()
        .unwrap();
    assert!(result.success);
}

#[test]
fn large_counts_do_not_overflow_percentage_multiplication() {
    let result = CompletionRules::new(u32::MAX, u32::MAX)
        .unwrap()
        .evaluate(CompletionProgress {
            released: u32::MAX,
            removed: u32::MAX,
            rescued: u32::MAX,
            max_lemmings: u32::MAX,
            ..progress()
        })
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            result.target_percent,
            result.rescued_percent,
            result.success
        ),
        (100, 100, true)
    );
}

#[test]
fn malformed_config_and_count_subsets_return_errors() {
    assert_eq!(
        CompletionRules::new(10, 11),
        Err(CompletionError::RequiredExceedsTotal)
    );
    let rules = CompletionRules::new(10, 5).unwrap();
    for p in [
        CompletionProgress {
            released: 11,
            ..progress()
        },
        CompletionProgress {
            removed: 11,
            ..progress()
        },
        CompletionProgress {
            rescued: 1,
            ..progress()
        },
        CompletionProgress {
            max_lemmings: 11,
            ..progress()
        },
        CompletionProgress {
            max_lemmings: 9,
            ..progress()
        },
        CompletionProgress {
            particle_timer: 53,
            ..progress()
        },
    ] {
        assert_eq!(rules.evaluate(p), Err(CompletionError::InvalidProgress));
    }
}

#[test]
fn exhaustive_small_denominators_use_original_target_and_current_count() {
    for total in 1..=32 {
        for required in 0..=total {
            let rules = CompletionRules::new(total, required).unwrap();
            for available in 1..=total {
                for rescued in 0..=available {
                    let result = rules
                        .evaluate(CompletionProgress {
                            released: available,
                            removed: available,
                            rescued,
                            max_lemmings: available,
                            nuked: available != total,
                            ..progress()
                        })
                        .unwrap()
                        .unwrap();
                    assert_eq!(
                        result.success,
                        rescued * 100 / available >= required * 100 / total
                    );
                }
            }
        }
    }
}

#[test]
fn percentage_helper_uses_wide_output_and_defines_zero_denominators() {
    assert_eq!(lemmings_core::percentage(u32::MAX, 1), 429_496_729_500);
    assert_eq!(lemmings_core::percentage(u32::MAX, 0), 0);
}
