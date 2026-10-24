use super::*;

/// Seed where every exercise asks 10 repetitions, with a 10 s recovery: 40 s each.
fn even_config() -> Config {
    let mut cfg = seed_config(Language::Fr);
    cfg.recovery = 10;
    for ex in &mut cfg.exercises {
        (ex.min, ex.max, ex.recovery, ex.timed, ex.each_side) = (10, 10, None, None, false);
    }
    cfg
}

fn limited(kind: LimitKind, value: u32) -> Session {
    let mut s = Session::default();
    s.set_limit(Some(SessionLimit { kind, value })).unwrap();
    s
}

#[test]
fn a_limit_in_exercises_reduces_the_last_set_then_gives_none() {
    let cfg = even_config();
    let mut s = limited(LimitKind::Exercises, 3);
    let mut r = rng();
    assert_eq!(
        s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap().items.len(),
        2
    );
    let reduced = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    assert_eq!(reduced.items.len(), 1, "only the exercise left fits");
    assert!(!reduced.limit_reached);
    let refused = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    assert!(refused.items.is_empty());
    assert!(refused.limit_reached && !refused.capped);
    assert_eq!(refused.trigger_count, 2, "a refused trigger is not counted");
    assert!(s.limit_reached(0));
    assert_eq!(s.view().asked, 3);
    assert_eq!(s.view().pending.len(), 2, "the sets asked stay to do");
}

#[test]
fn dismissing_a_set_frees_its_room_and_stopping_one_does_not() {
    let cfg = even_config();
    let mut s = limited(LimitKind::Exercises, 2);
    let mut r = rng();
    let first = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    assert!(s.limit_reached(0));
    s.dismiss(first.id).unwrap();
    assert!(!s.limit_reached(0), "a dismissed set is no longer counted");
    let again = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    assert_eq!(again.items, first.items, "the reserve is given back");
    s.complete_exercise(again.id, &again.items[0].exercise_id, 1)
        .unwrap();
    s.dismiss(again.id).unwrap();
    assert!(s.limit_reached(1), "a stopped set stays counted");
}

#[test]
fn a_dismissed_set_frees_what_it_asked_even_once_a_saved_configuration_changed_it() {
    let mut cfg = even_config();
    let mut s = limited(LimitKind::Exercises, 20);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    event_mut(&mut cfg, "ds3", "boss").draws = 1;
    s.apply_config(&cfg, &mut r);
    assert_eq!(s.view().pending[0].items.len(), 1);
    assert_eq!(s.view().asked, 2, "a saved configuration never recounts");
    s.dismiss(boss.id).unwrap();
    assert_eq!(s.view().asked, 0, "the set frees all it asked");
}

#[test]
fn a_reserved_set_reduced_by_the_limit_gives_the_rest_back_to_the_round() {
    let mut cfg = even_config();
    for ex in cfg.exercises.iter_mut().skip(4) {
        ex.enabled = false;
    }
    let mut s = limited(LimitKind::Exercises, 3);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    s.dismiss(boss.id).unwrap();
    // The two other exercises of the round.
    s.draw(&cfg, "ds3", "boss-premier", 0, &mut r).unwrap();
    let reduced = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    assert_eq!(reduced.items, boss.items[..1], "the reserve, reduced");
    s.set_limit(None).unwrap();
    assert_eq!(
        s.draw(&cfg, "ds3", "mort", 0, &mut r).unwrap().items[0].exercise_id,
        boss.items[1].exercise_id,
        "the exercise left out comes out before the round starts over"
    );
}

#[test]
fn changing_the_limit_applies_to_the_next_triggers_only() {
    let cfg = even_config();
    let mut s = limited(LimitKind::Exercises, 2);
    let mut r = rng();
    s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    s.set_limit(Some(SessionLimit {
        kind: LimitKind::Exercises,
        value: 1,
    }))
    .unwrap();
    assert_eq!(
        s.view().pending[0].items.len(),
        2,
        "a lower limit removes nothing"
    );
    s.set_limit(Some(SessionLimit {
        kind: LimitKind::Exercises,
        value: 4,
    }))
    .unwrap();
    assert_eq!(
        s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap().items.len(),
        2
    );
    s.set_limit(None).unwrap();
    assert_eq!(
        s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap().items.len(),
        2
    );
}

#[test]
fn a_limit_out_of_bounds_is_refused() {
    let mut s = Session::default();
    for value in [0, 10000] {
        let limit = SessionLimit {
            kind: LimitKind::PlayTime,
            value,
        };
        assert_eq!(s.set_limit(Some(limit)).unwrap_err().code, "invalid_limit");
    }
    assert_eq!(s.view().limit, None);
}

#[test]
fn a_limit_in_play_time_counts_only_the_time_the_app_was_open() {
    let cfg = even_config();
    let mut s = limited(LimitKind::PlayTime, 10);
    let mut r = rng();
    s.start(&cfg, "ds3", 1000).unwrap();
    assert!(
        !s.draw(&cfg, "ds3", "boss", 1000 + 599, &mut r)
            .unwrap()
            .limit_reached
    );
    s.resume(1000 + 300, 5000);
    assert!(
        !s.limit_reached(5000 + 299),
        "the time closed does not count"
    );
    let refused = s.draw(&cfg, "ds3", "boss", 5000 + 300, &mut r).unwrap();
    assert!(refused.limit_reached && refused.items.is_empty());
}

#[test]
fn a_limit_in_training_time_keeps_the_exercises_that_fit() {
    let cfg = even_config();
    // 40 s each: 3 exercises fit in 2 minutes.
    let mut s = limited(LimitKind::TrainingTime, 2);
    let mut r = rng();
    let first = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    assert_eq!(first.items.len(), 2);
    let reduced = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    assert_eq!(reduced.items.len(), 1);
    assert_eq!(s.view().asked_time, 120);
    assert!(s.view().limit_full && s.limit_reached(0));
    assert!(
        s.draw(&cfg, "ds3", "boss", 0, &mut r)
            .unwrap()
            .limit_reached
    );
}

#[test]
fn a_set_that_does_not_fit_at_all_reaches_the_limit_in_training_time() {
    let cfg = even_config();
    // 40 s each: a single exercise fits in 1 minute, the second set finds no room.
    let mut s = limited(LimitKind::TrainingTime, 1);
    let mut r = rng();
    assert_eq!(
        s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap().items.len(),
        1
    );
    assert!(
        s.limit_reached(0),
        "a set that did not fit entirely reaches the limit"
    );
    assert!(
        s.draw(&cfg, "ds3", "boss", 0, &mut r)
            .unwrap()
            .limit_reached
    );
}

#[test]
fn the_estimate_counts_the_repetitions_the_timer_its_sides_and_the_recovery() {
    let mut cfg = even_config();
    cfg.timer_prep = 3;
    assert_eq!(cfg.rep_duration, 3, "3 s per repetition by default");
    let ex = &mut cfg.exercises[0];
    (ex.timed, ex.each_side, ex.recovery) = (Some(TimeUnit::Minutes), true, Some(30));
    let item = |id: &str, amount| DrawItem {
        exercise_id: id.into(),
        name: String::new(),
        unit: String::new(),
        amount,
        base: amount,
        halved: false,
        done: false,
        halve_cost: None,
        skip_cost: 2,
    };
    let timed = cfg.exercises[0].id.clone();
    assert_eq!(estimate(&cfg, &item(&timed, 2)), 2 * (3 + 120) + 30);
    let reps = cfg.exercises[1].id.clone();
    assert_eq!(estimate(&cfg, &item(&reps, 10)), 10 * 3 + 10);
    cfg.rep_duration = 5;
    assert_eq!(estimate(&cfg, &item(&reps, 10)), 10 * 5 + 10);
}

#[test]
fn a_saved_configuration_never_fills_a_reduced_set_again() {
    let cfg = even_config();
    let mut s = limited(LimitKind::Exercises, 3);
    let mut r = rng();
    s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    s.apply_config(&cfg, &mut r);
    let sizes: Vec<_> = s.view().pending.iter().map(|d| d.items.len()).collect();
    assert_eq!(sizes, [2, 1]);
}

#[test]
fn a_limit_reads_back_as_written() {
    for kind in [
        LimitKind::Exercises,
        LimitKind::PlayTime,
        LimitKind::TrainingTime,
    ] {
        let limit = SessionLimit { kind, value: 45 };
        assert_eq!(limit.to_string().parse(), Ok(limit));
    }
    assert!("playTime:0".parse::<SessionLimit>().is_err());
    assert!("hours:5".parse::<SessionLimit>().is_err());
}
