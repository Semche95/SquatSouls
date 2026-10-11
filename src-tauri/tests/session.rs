use rand::rngs::StdRng;
use rand::SeedableRng;
use squat_souls_lib::fitness::*;
use std::collections::HashSet;

fn rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

/// Marks done, one by one, the remaining exercises of a set.
fn complete(s: &mut Session, id: u64, now: i64) -> Result<Vec<LogEntry>, String> {
    let draw = s
        .view()
        .pending
        .into_iter()
        .find(|d| d.id == id)
        .ok_or("This set is no longer pending.")?;
    draw.items
        .iter()
        .filter(|it| !it.done)
        .map(|it| s.complete_exercise(id, &it.exercise_id, now))
        .collect()
}

#[test]
fn draws_stay_in_range_and_are_distinct() {
    let cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    for _ in 0..200 {
        let d = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
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
    let cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let mut seen = HashSet::new();
    for _ in 0..100 {
        let d = s.draw(&cfg, "elden-ring", "repos", &mut r).unwrap();
        seen.insert(d.items[0].exercise_id.clone());
    }
    assert_eq!(seen.len(), cfg.exercises.len());
}

#[test]
fn each_exercise_comes_out_once_per_round() {
    let cfg = default_config();
    let n = cfg.exercises.len();
    let mut s = Session::new(0);
    let mut r = rng();
    for _ in 0..20 {
        let round: HashSet<_> = (0..n)
            .map(|_| {
                s.draw(&cfg, "elden-ring", "repos", &mut r).unwrap().items[0]
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
    let cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let mut last: Vec<String> = Vec::new();
    for _ in 0..200 {
        let event = if last.len() == 1 { "boss" } else { "repos" };
        let d = s.draw(&cfg, "ds3", event, &mut r).unwrap();
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
    let mut cfg = default_config();
    // A single drawn exercise shifts the round: with an even number of exercises, draws of 2 then straddle two rounds.
    cfg.exercises.truncate(cfg.exercises.len() / 2 * 2);
    let n = cfg.exercises.len();
    let mut s = Session::new(0);
    let mut r = rng();
    s.draw(&cfg, "ds3", "repos", &mut r).unwrap();
    let mut out: Vec<String> = Vec::new();
    for _ in 0..50 {
        let d = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
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
    let cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    for n in 1..=15 {
        let d = s.draw(&cfg, "ds1", "mort", &mut r).unwrap();
        assert!(!d.capped);
        assert_eq!(d.trigger_count, n);
    }
    for _ in 0..3 {
        let d = s.draw(&cfg, "ds1", "mort", &mut r).unwrap();
        assert!(d.capped && d.items.is_empty());
        assert_eq!(d.trigger_count, 15, "the counter stops at the limit");
    }
    assert_eq!(s.view().pending.len(), 15, "a capped draw is not queued");
    // The session stays on its game.
    assert!(s.draw(&cfg, "ds2", "mort", &mut r).is_err());
    assert_eq!(s.view().game_id.as_deref(), Some("ds1"));
    // A new session resets everything.
    let mut s2 = Session::new(1);
    assert!(!s2.draw(&cfg, "ds1", "mort", &mut r).unwrap().capped);
}

#[test]
fn removed_game_is_detected() {
    let mut cfg = default_config();
    let mut s = Session::new(0);
    assert!(!s.game_removed(&cfg), "session not started yet");
    s.draw(&cfg, "ds1", "mort", &mut rng()).unwrap();
    assert!(!s.game_removed(&cfg));
    cfg.games.retain(|g| g.id != "ds2");
    assert!(!s.game_removed(&cfg), "another removed game does not count");
    cfg.games.retain(|g| g.id != "ds1");
    assert!(s.game_removed(&cfg));
}

#[test]
fn complete_dismiss_and_totals() {
    let cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let a = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    let b = s.draw(&cfg, "ds3", "mort", &mut r).unwrap();
    let c = s.draw(&cfg, "ds3", "mort", &mut r).unwrap();

    let logged = complete(&mut s, a.id, 100).unwrap();
    assert_eq!(logged.len(), 2);
    complete(&mut s, b.id, 101).unwrap();
    s.dismiss(c.id).unwrap();
    assert!(complete(&mut s, c.id, 102).is_err());

    let v = s.view();
    assert!(v.pending.is_empty());
    assert_eq!(v.sets, 3);
    assert_eq!(v.log[0].event, "Mort");
    let count = v
        .triggers
        .iter()
        .find(|t| t.event_id == "mort")
        .unwrap()
        .count;
    assert_eq!(count, 1, "dismissing removes the trigger");
    let sum: u32 = v.totals.iter().map(|t| t.sets).sum();
    assert_eq!(sum, 3);
}

/// Session where a completed "boss no hit" earned 2 jokers, with a two-exercise boss set pending.
fn session_with_jokers() -> (Session, Draw) {
    let cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    assert!(s.skip_set(boss.id, 5).is_err(), "no joker at first");

    let nohit = s.draw(&cfg, "ds3", "boss-nohit", &mut r).unwrap();
    assert_eq!(nohit.items.len(), 2, "the reward comes with a set");
    assert_eq!(nohit.jokers, 2);
    assert_eq!(s.view().jokers, 0, "nothing is earned before completion");
    assert_eq!(complete(&mut s, nohit.id, 10).unwrap().len(), 2);
    assert_eq!(s.view().jokers, 2);
    (s, boss)
}

#[test]
fn skipping_a_set_costs_its_exercises() {
    let (mut s, boss) = session_with_jokers();
    assert_eq!(skip_set_cost(&boss), 2 * SKIP_COST);
    assert!(
        s.skip_set(boss.id, 20).is_err(),
        "two exercises cost 4 jokers"
    );

    let cfg = default_config();
    let death = s.draw(&cfg, "ds3", "mort", &mut rng()).unwrap();
    s.skip_set(death.id, 20).unwrap();
    let v = s.view();
    assert_eq!(v.jokers, 0);
    assert_eq!(v.jokers_earned, 2);
    assert_eq!(v.skipped.len(), 1);
    let k = &v.skipped[0];
    assert_eq!(
        (k.at, k.event.as_str(), k.jokers, k.halved),
        (20, "Mort", SKIP_COST, false)
    );
    assert_eq!(k.items, death.items);
    assert_eq!(v.pending.len(), 1, "only the boss set remains");
    assert_eq!(v.sets, 2, "only the no hit set is done");
    let count = v
        .triggers
        .iter()
        .find(|t| t.event_id == "mort")
        .unwrap()
        .count;
    assert_eq!(count, 1, "a skipped death still counts toward the limit");
}

#[test]
fn skipping_an_exercise_keeps_the_rest_of_the_set() {
    let (mut s, boss) = session_with_jokers();
    let (first, second) = (&boss.items[0], &boss.items[1]);
    s.skip_exercise(boss.id, &first.exercise_id, 20).unwrap();
    let v = s.view();
    assert_eq!(v.jokers, 0);
    assert_eq!(v.pending[0].items, vec![second.clone()]);
    assert_eq!(v.skipped[0].items, vec![first.clone()]);
    assert_eq!(v.skipped[0].jokers, SKIP_COST);
    assert!(
        s.skip_exercise(boss.id, &second.exercise_id, 30).is_err(),
        "plus de joker"
    );
    assert!(
        s.skip_exercise(boss.id, &first.exercise_id, 30).is_err(),
        "already skipped"
    );

    let entries = complete(&mut s, boss.id, 40).unwrap();
    assert_eq!(entries.len(), 1, "only the remaining exercise is done");
    assert_eq!(entries[0].amount, second.amount);
}

#[test]
fn halving_then_skipping_costs_a_full_skip() {
    let (mut s, boss) = session_with_jokers();
    let item = &boss.items[0];
    assert!(item.amount >= 2);
    assert_eq!(item.halve_cost, Some(HALVE_COST));
    s.halve(boss.id, &item.exercise_id, 20).unwrap();
    let v = s.view();
    let halved = &v.pending[0].items[0];
    assert_eq!(halved.amount, item.amount.div_ceil(2), "rounded up");
    assert!(halved.halved);
    assert_eq!(halved.halve_cost, None, "a single half per exercise");
    assert_eq!(halved.skip_cost, SKIP_COST - HALVE_COST);
    assert_eq!(v.jokers, 2 - HALVE_COST);
    let k = &v.skipped[0];
    assert!(k.halved);
    assert_eq!(k.jokers, HALVE_COST);
    assert_eq!(k.items[0].amount, item.amount / 2, "the removed half");
    assert!(s.halve(boss.id, &item.exercise_id, 30).is_err());

    s.skip_exercise(boss.id, &item.exercise_id, 30).unwrap();
    assert_eq!(
        s.view().jokers,
        0,
        "the rest of a halved exercise costs the difference"
    );

    let other = &boss.items[1];
    assert!(
        s.halve(boss.id, &other.exercise_id, 40).is_err(),
        "plus de joker"
    );
}

#[test]
fn spending_jokers_on_a_rewarding_set_forfeits_its_reward() {
    let cfg = default_config();
    for spend in [Session::halve, Session::skip_exercise] {
        let (mut s, _) = session_with_jokers();
        let nohit = s.draw(&cfg, "ds3", "boss-nohit", &mut rng()).unwrap();
        let first = nohit
            .items
            .iter()
            .find(|it| it.halve_cost.is_some())
            .unwrap();
        spend(&mut s, nohit.id, &first.exercise_id, 20).unwrap();
        assert_eq!(s.view().pending[1].jokers, 0, "the reward is lost");
        let left = s.view().jokers;
        complete(&mut s, nohit.id, 30).unwrap();
        let v = s.view();
        assert_eq!(v.jokers, left, "nothing is earned on completion");
        assert_eq!(v.jokers_earned, 2);
    }
}

#[test]
fn skipping_every_exercise_removes_the_set() {
    let (mut s, _) = session_with_jokers();
    let cfg = default_config();
    let death = s.draw(&cfg, "ds3", "mort", &mut rng()).unwrap();
    s.skip_exercise(death.id, &death.items[0].exercise_id, 20)
        .unwrap();
    assert!(s.view().pending.iter().all(|d| d.id != death.id));
}

#[test]
fn a_single_unit_cannot_be_halved() {
    let mut cfg = default_config();
    for ex in &mut cfg.exercises {
        ex.min = 1;
        ex.max = 1;
    }
    let d = Session::new(0)
        .draw(&cfg, "ds3", "mort", &mut rng())
        .unwrap();
    assert_eq!(d.items[0].halve_cost, None);
}

#[test]
fn exercises_are_done_one_by_one() {
    let (mut s, boss) = session_with_jokers();
    let cfg = default_config();
    let nohit = s.draw(&cfg, "ds3", "boss-nohit", &mut rng()).unwrap();
    let (first, second) = (&nohit.items[0], &nohit.items[1]);
    let entry = s
        .complete_exercise(nohit.id, &first.exercise_id, 20)
        .unwrap();
    assert_eq!((entry.at, entry.amount), (20, first.amount));
    let v = s.view();
    let left = v.pending.iter().find(|d| d.id == nohit.id).unwrap();
    assert!(left.items[0].done && !left.items[1].done);
    assert_eq!(
        skip_set_cost(left),
        SKIP_COST,
        "only the remaining exercise counts"
    );
    assert_eq!(v.jokers, 2, "the reward waits for the end of the set");
    for again in [
        s.complete_exercise(nohit.id, &first.exercise_id, 21)
            .map(|_| ()),
        s.halve(nohit.id, &first.exercise_id, 21),
        s.skip_exercise(nohit.id, &first.exercise_id, 21),
    ] {
        assert!(again.is_err(), "already done");
    }

    s.complete_exercise(nohit.id, &second.exercise_id, 30)
        .unwrap();
    let v = s.view();
    assert_eq!(v.jokers, 4, "set finished: the reward is earned");
    assert_eq!(v.pending.len(), 1, "only the boss set remains");
    assert_eq!(v.log[0].at, 30);

    s.complete_exercise(boss.id, &boss.items[0].exercise_id, 40)
        .unwrap();
    s.skip_set(boss.id, 50).unwrap();
    let v = s.view();
    assert_eq!(
        v.jokers,
        4 - SKIP_COST,
        "skipping the rest only costs the remaining exercise"
    );
    assert_eq!(v.skipped[0].items, vec![boss.items[1].clone()]);
    assert_eq!(v.sets, 5);
}

#[test]
fn dismissing_a_started_set_drops_its_done_exercises() {
    let cfg = default_config();
    let mut s = Session::new(0);
    let mort = s.draw(&cfg, "ds3", "mort", &mut rng()).unwrap();
    let kept = complete(&mut s, mort.id, 5).unwrap();
    let boss = s.draw(&cfg, "ds3", "boss", &mut rng()).unwrap();
    let entry = s
        .complete_exercise(boss.id, &boss.items[0].exercise_id, 10)
        .unwrap();
    assert_eq!(
        s.dismiss(boss.id).unwrap(),
        vec![entry],
        "to remove from the history"
    );
    let v = s.view();
    assert!(v.pending.is_empty());
    assert_eq!(
        v.log,
        kept.into_iter().rev().collect::<Vec<_>>(),
        "nothing of the dismissed set is counted"
    );
    assert_eq!(v.triggers.len(), 1, "the trigger is uncounted");
}

#[test]
fn a_dismissed_set_comes_back_on_the_next_trigger_of_its_event() {
    let cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    s.complete_exercise(boss.id, &boss.items[1].exercise_id, 2)
        .unwrap();
    s.dismiss(boss.id).unwrap();
    let other = s.draw(&cfg, "ds3", "boss-premier", &mut r).unwrap();
    assert_ne!(other.items, boss.items, "another event draws normally");
    let again = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    assert_eq!(again.items, boss.items, "same set, as drawn");
    assert_eq!(again.trigger_count, 1);
    s.dismiss(again.id).unwrap();
    let mut changed = cfg.clone();
    changed
        .exercises
        .retain(|ex| boss.items.iter().all(|it| it.exercise_id != ex.id));
    s.apply_config(&changed, &mut r);
    let fresh = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    assert_ne!(
        fresh.items, boss.items,
        "the saved configuration replaces the removed exercises of the reserve"
    );
    assert_eq!(fresh.items.len(), 2);
}

/// Event of the default configuration, to modify.
fn event_mut<'a>(cfg: &'a mut Config, game: &str, event: &str) -> &'a mut GameEvent {
    cfg.games
        .iter_mut()
        .find(|g| g.id == game)
        .and_then(|g| g.events.iter_mut().find(|e| e.id == event))
        .unwrap()
}

fn pending(s: &Session, id: u64) -> Option<Draw> {
    s.view().pending.into_iter().find(|d| d.id == id)
}

#[test]
fn a_saved_configuration_keeps_the_sets_to_do() {
    let mut cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    event_mut(&mut cfg, "ds3", "boss").name = "Boss defeated".into();
    cfg.pause = 30;
    s.apply_config(&cfg, &mut r);
    let kept = pending(&s, boss.id).expect("the set remains to do");
    assert_eq!(kept.items, boss.items, "same exercises, same numbers");
    assert_eq!(kept.event_name, "Boss defeated");
}

#[test]
fn changing_the_draws_adjusts_the_sets_to_do() {
    let mut cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    event_mut(&mut cfg, "ds3", "boss").draws = 4;
    s.apply_config(&cfg, &mut r);
    let more = pending(&s, boss.id).unwrap();
    assert_eq!(more.items.len(), 4);
    assert_eq!(
        more.items[..2],
        boss.items[..],
        "the drawn exercises remain"
    );
    let ids: HashSet<_> = more.items.iter().map(|it| &it.exercise_id).collect();
    assert_eq!(ids.len(), 4, "the added exercises are distinct");

    s.complete_exercise(boss.id, &more.items[3].exercise_id, 1)
        .unwrap();
    event_mut(&mut cfg, "ds3", "boss").draws = 2;
    s.apply_config(&cfg, &mut r);
    let fewer = pending(&s, boss.id).unwrap();
    assert_eq!(fewer.items.len(), 2);
    assert!(
        fewer.items.iter().any(|it| it.done),
        "the done exercise remains"
    );

    event_mut(&mut cfg, "ds3", "boss").draws = 1;
    s.apply_config(&cfg, &mut r);
    assert!(
        pending(&s, boss.id).is_none(),
        "nothing left to do: the set is finished"
    );
}

#[test]
fn new_bounds_clamp_the_amounts_to_do() {
    let mut cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    let id = boss.items[0].exercise_id.clone();
    let ex = cfg.exercises.iter_mut().find(|ex| ex.id == id).unwrap();
    ex.min = 100;
    ex.max = 200;
    s.apply_config(&cfg, &mut r);
    assert_eq!(pending(&s, boss.id).unwrap().items[0].amount, 100);
    let ex = cfg.exercises.iter_mut().find(|ex| ex.id == id).unwrap();
    ex.min = 1;
    ex.max = 1;
    s.apply_config(&cfg, &mut r);
    let item = &pending(&s, boss.id).unwrap().items[0];
    assert_eq!(item.amount, 1);
    assert_eq!(item.halve_cost, None, "a single one cannot be halved");
}

#[test]
fn a_removed_exercise_is_replaced_in_the_sets_to_do() {
    let mut cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    let removed = boss.items[0].exercise_id.clone();
    cfg.exercises.retain(|ex| ex.id != removed);
    s.apply_config(&cfg, &mut r);
    let draw = pending(&s, boss.id).unwrap();
    assert_eq!(draw.items.len(), 2);
    assert_eq!(draw.items[1], boss.items[1]);
    assert!(draw.items.iter().all(|it| it.exercise_id != removed));
    assert_ne!(draw.items[0].exercise_id, draw.items[1].exercise_id);
}

#[test]
fn an_unchecked_exercise_is_never_drawn() {
    let mut cfg = default_config();
    let unchecked = cfg.exercises[0].id.clone();
    cfg.exercises[0].enabled = false;
    let mut s = Session::new(0);
    let mut r = rng();
    for _ in 0..100 {
        let d = s.draw(&cfg, "elden-ring", "repos", &mut r).unwrap();
        assert_ne!(d.items[0].exercise_id, unchecked);
    }
}

#[test]
fn an_unchecked_exercise_is_replaced_in_the_sets_to_do() {
    let mut cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    let unchecked = boss.items[0].exercise_id.clone();
    cfg.exercises
        .iter_mut()
        .find(|ex| ex.id == unchecked)
        .unwrap()
        .enabled = false;
    s.apply_config(&cfg, &mut r);
    let draw = pending(&s, boss.id).unwrap();
    assert_eq!(draw.items.len(), 2);
    assert_eq!(draw.items[1], boss.items[1]);
    assert!(draw.items.iter().all(|it| it.exercise_id != unchecked));
}

#[test]
fn removed_events_and_new_jokers_apply_to_the_sets_to_do() {
    let mut cfg = default_config();
    let mut s = Session::new(0);
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    let first = s.draw(&cfg, "ds3", "boss-premier", &mut r).unwrap();
    event_mut(&mut cfg, "ds3", "boss-premier").jokers = 3;
    let ds3 = cfg.games.iter_mut().find(|g| g.id == "ds3").unwrap();
    ds3.events.retain(|e| e.id != "boss");
    s.apply_config(&cfg, &mut r);
    assert!(
        pending(&s, boss.id).is_none(),
        "the removed event takes its set along"
    );
    assert_eq!(pending(&s, first.id).unwrap().jokers, 3);
}

#[test]
fn a_set_with_spent_jokers_still_earns_nothing() {
    let (mut s, boss) = session_with_jokers();
    let mut cfg = default_config();
    s.halve(boss.id, &boss.items[0].exercise_id, 20).unwrap();
    event_mut(&mut cfg, "ds3", "boss").jokers = 2;
    s.apply_config(&cfg, &mut rng());
    assert_eq!(pending(&s, boss.id).unwrap().jokers, 0);
}
