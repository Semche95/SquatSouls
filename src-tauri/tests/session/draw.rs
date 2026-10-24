use super::*;
use std::collections::HashSet;

#[test]
fn draws_stay_in_range_and_are_distinct() {
    let cfg = seed_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    for _ in 0..200 {
        let d = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
        assert_eq!(d.items.len(), 2);
        let ids: HashSet<_> = d.items.iter().map(|i| i.exercise_id.as_str()).collect();
        assert_eq!(
            ids.len(),
            2,
            "the exercises of a single draw must be different"
        );
        for it in &d.items {
            let ex = cfg
                .exercises
                .iter()
                .find(|e| e.id == it.exercise_id)
                .unwrap();
            assert!(it.amount >= ex.min && it.amount <= ex.max);
        }
    }
}

#[test]
fn every_option_can_be_drawn() {
    let cfg = seed_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let mut seen = HashSet::new();
    for _ in 0..100 {
        let d = s.draw(&cfg, "elden-ring", "repos", 0, &mut r).unwrap();
        seen.insert(d.items[0].exercise_id.clone());
    }
    assert_eq!(seen.len(), cfg.exercises.len());
}

#[test]
fn each_exercise_comes_out_once_per_round() {
    let cfg = seed_config(Language::Fr);
    let n = cfg.exercises.len();
    let mut s = Session::default();
    let mut r = rng();
    for _ in 0..20 {
        let round: HashSet<_> = (0..n)
            .map(|_| {
                s.draw(&cfg, "elden-ring", "repos", 0, &mut r)
                    .unwrap()
                    .items[0]
                    .exercise_id
                    .clone()
            })
            .collect();
        assert_eq!(
            round.len(),
            n,
            "no exercise comes back before the end of the round"
        );
    }
}

#[test]
fn a_new_round_does_not_start_with_the_last_set() {
    let cfg = seed_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let mut last: Vec<String> = Vec::new();
    for _ in 0..200 {
        let event = if last.len() == 1 { "boss" } else { "repos" };
        let d = s.draw(&cfg, "ds3", event, 0, &mut r).unwrap();
        let ids: Vec<String> = d.items.into_iter().map(|it| it.exercise_id).collect();
        assert!(
            ids.iter().all(|id| !last.contains(id)),
            "an exercise of the previous set does not come out again right away"
        );
        last = ids;
    }
}

#[test]
fn a_draw_across_two_rounds_stays_distinct() {
    let mut cfg = seed_config(Language::Fr);
    // A single drawn exercise shifts the round: with an even number of exercises, draws of 2 then straddle two rounds.
    cfg.exercises.truncate(cfg.exercises.len() / 2 * 2);
    let n = cfg.exercises.len();
    let mut s = Session::default();
    let mut r = rng();
    s.draw(&cfg, "ds3", "repos", 0, &mut r).unwrap();
    let mut out: Vec<String> = Vec::new();
    for _ in 0..50 {
        let d = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
        assert_ne!(d.items[0].exercise_id, d.items[1].exercise_id);
        out.extend(d.items.into_iter().map(|it| it.exercise_id));
    }
    // After the first single drawn exercise, each full round counts each exercise once.
    for round in out[n - 1..].chunks_exact(n) {
        assert_eq!(round.iter().collect::<HashSet<_>>().len(), n);
    }
}

#[test]
fn death_cap_applies_per_game_and_session() {
    let cfg = seed_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    for n in 1..=15 {
        let d = s.draw(&cfg, "ds1", "mort", 0, &mut r).unwrap();
        assert!(!d.capped);
        assert_eq!(d.trigger_count, n);
    }
    for _ in 0..3 {
        let d = s.draw(&cfg, "ds1", "mort", 0, &mut r).unwrap();
        assert!(d.capped && d.items.is_empty());
        assert_eq!(d.trigger_count, 15, "the counter stops at the limit");
    }
    assert_eq!(s.view().pending.len(), 15, "a capped draw is not queued");
    // The session stays on its game.
    assert_eq!(
        s.draw(&cfg, "ds2", "mort", 0, &mut r).unwrap_err().code,
        "session_other_game"
    );
    assert_eq!(s.view().game_id.as_deref(), Some("ds1"));
    // A new session resets everything.
    let mut s2 = Session::default();
    assert!(!s2.draw(&cfg, "ds1", "mort", 0, &mut r).unwrap().capped);
}

#[test]
fn a_disabled_exercise_is_never_drawn() {
    let mut cfg = seed_config(Language::Fr);
    let disabled = cfg.exercises[0].id.clone();
    cfg.exercises[0].enabled = false;
    let mut s = Session::default();
    let mut r = rng();
    for _ in 0..100 {
        let d = s.draw(&cfg, "elden-ring", "repos", 0, &mut r).unwrap();
        assert_ne!(d.items[0].exercise_id, disabled);
    }
}
