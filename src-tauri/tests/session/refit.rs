use super::*;
use std::collections::HashSet;

#[test]
fn removed_game_is_detected() {
    let mut cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    assert_eq!(s.end_cause(&cfg), None, "session not started yet");
    s.draw(
        &cfg,
        &game_id("ds1"),
        &event_id("ds1", "Death"),
        0,
        &mut rng(),
    )
    .unwrap();
    assert_eq!(s.end_cause(&cfg), None);
    cfg.games.retain(|g| g.id != game_id("ds2"));
    assert_eq!(
        s.end_cause(&cfg),
        None,
        "another removed game does not count"
    );
    cfg.games.retain(|g| g.id != game_id("ds1"));
    assert_eq!(s.end_cause(&cfg), Some(EndCause::GameRemoved));
}

#[test]
fn removed_game_is_detected_for_a_session_started_by_its_button() {
    let mut cfg = catalog_config(Language::En);
    let mut s = Session::default();
    s.start(&cfg, &game_id("ds1"), 5).unwrap();
    cfg.games.retain(|g| g.id != game_id("ds2"));
    assert_eq!(
        s.end_cause(&cfg),
        None,
        "another removed game does not count"
    );
    cfg.games.clear();
    assert_eq!(s.end_cause(&cfg), Some(EndCause::GameRemoved));
}

#[test]
fn a_session_without_included_exercise_is_detected() {
    let mut cfg = catalog_config(Language::En);
    let mut s = Session::default();
    cfg.exercises.iter_mut().for_each(|ex| ex.enabled = false);
    assert_eq!(s.end_cause(&cfg), None, "session not started yet");
    let cfg_on = catalog_config(Language::En);
    s.start(&cfg_on, &game_id("ds1"), 5).unwrap();
    cfg.exercises[0].enabled = true;
    assert_eq!(s.end_cause(&cfg), None, "one included exercise is enough");
    cfg.exercises[0].enabled = false;
    assert_eq!(s.end_cause(&cfg), Some(EndCause::NoExercise));
    cfg.exercises.clear();
    assert_eq!(s.end_cause(&cfg), Some(EndCause::NoExercise));
}

#[test]
fn an_event_asking_for_more_than_the_included_exercises_gets_all_of_them() {
    let mut cfg = catalog_config(Language::En);
    cfg.exercises
        .iter_mut()
        .skip(3)
        .for_each(|ex| ex.enabled = false);
    let ev = &mut cfg.games[0].events[0];
    ev.draws = 4;
    let (game, event) = (cfg.games[0].id.clone(), cfg.games[0].events[0].id.clone());
    let draw = Session::default()
        .draw(&cfg, &game, &event, 0, &mut rng())
        .unwrap();
    let ids: HashSet<_> = draw
        .items
        .iter()
        .map(|it| it.exercise_id.as_str())
        .collect();
    let included: HashSet<_> = cfg.active_exercises().map(|ex| ex.id.as_str()).collect();
    assert_eq!(ids, included, "each included exercise once");
}

#[test]
fn a_saved_configuration_keeps_the_sets_to_do() {
    let mut cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    event_mut(&mut cfg, &game_id("ds3"), &event_id("ds3", "Boss defeated")).name =
        "Boss defeated".into();
    cfg.recovery = 30;
    s.apply_config(&cfg, &mut r);
    let kept = pending(&s, &boss.id).expect("the set remains to do");
    assert_eq!(kept.items, boss.items, "same exercises, same numbers");
    assert_eq!(kept.event_name, "Boss defeated");
}

#[test]
fn changing_the_draws_adjusts_the_sets_to_do() {
    let mut cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    event_mut(&mut cfg, &game_id("ds3"), &event_id("ds3", "Boss defeated")).draws = 4;
    s.apply_config(&cfg, &mut r);
    let more = pending(&s, &boss.id).unwrap();
    assert_eq!(more.items.len(), 4);
    assert_eq!(
        more.items[..2],
        boss.items[..],
        "the drawn exercises remain"
    );
    let ids: HashSet<_> = more.items.iter().map(|it| &it.exercise_id).collect();
    assert_eq!(ids.len(), 4, "the added exercises are distinct");

    s.complete_exercise(&boss.id, &more.items[0].exercise_id, 1)
        .unwrap();
    event_mut(&mut cfg, &game_id("ds3"), &event_id("ds3", "Boss defeated")).draws = 2;
    s.apply_config(&cfg, &mut r);
    let fewer = pending(&s, &boss.id).unwrap();
    assert_eq!(fewer.items.len(), 2);
    assert!(
        fewer.items.iter().any(|it| it.done),
        "the done exercise remains"
    );

    event_mut(&mut cfg, &game_id("ds3"), &event_id("ds3", "Boss defeated")).draws = 1;
    s.apply_config(&cfg, &mut r);
    assert!(
        pending(&s, &boss.id).is_none(),
        "nothing left to do: the set is finished"
    );
}

#[test]
fn new_bounds_clamp_the_amounts_to_do() {
    let mut cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    let id = boss.items[0].exercise_id.clone();
    let ex = cfg.exercises.iter_mut().find(|ex| ex.id == id).unwrap();
    ex.min = 100;
    ex.max = 200;
    s.apply_config(&cfg, &mut r);
    assert_eq!(pending(&s, &boss.id).unwrap().items[0].amount, 100);
    let ex = cfg.exercises.iter_mut().find(|ex| ex.id == id).unwrap();
    ex.min = 1;
    ex.max = 1;
    s.apply_config(&cfg, &mut r);
    let item = &pending(&s, &boss.id).unwrap().items[0];
    assert_eq!(item.amount, 1);
    assert_eq!(item.halve_cost, None, "a single one cannot be halved");
    assert_eq!(item.skip_cost, SKIP_COST - HALVE_COST);
}

#[test]
fn new_bounds_clamp_the_amounts_to_do_at_the_session_intensity() {
    let mut cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    s.set_intensity(Intensity::Hard);
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    let id = boss.items[0].exercise_id.clone();
    let ex = cfg.exercises.iter_mut().find(|ex| ex.id == id).unwrap();
    ex.min = 100;
    ex.max = 200;
    s.apply_config(&cfg, &mut r);
    assert_eq!(pending(&s, &boss.id).unwrap().items[0].amount, 150);
}

#[test]
fn a_removed_exercise_is_replaced_in_the_sets_to_do() {
    let mut cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    let removed = boss.items[0].exercise_id.clone();
    cfg.exercises.retain(|ex| ex.id != removed);
    s.apply_config(&cfg, &mut r);
    let draw = pending(&s, &boss.id).unwrap();
    assert_eq!(draw.items.len(), 2);
    assert_eq!(draw.items[1], boss.items[1]);
    assert!(draw.items.iter().all(|it| it.exercise_id != removed));
    assert_ne!(draw.items[0].exercise_id, draw.items[1].exercise_id);
}

#[test]
fn a_disabled_exercise_is_replaced_in_the_sets_to_do() {
    let mut cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    let disabled = boss.items[0].exercise_id.clone();
    cfg.exercises
        .iter_mut()
        .find(|ex| ex.id == disabled)
        .unwrap()
        .enabled = false;
    s.apply_config(&cfg, &mut r);
    let draw = pending(&s, &boss.id).unwrap();
    assert_eq!(draw.items.len(), 2);
    assert_eq!(draw.items[1], boss.items[1]);
    assert!(draw.items.iter().all(|it| it.exercise_id != disabled));
}

#[test]
fn removed_events_and_new_jokers_apply_to_the_sets_to_do() {
    let mut cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    let first = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated on the first try"),
            0,
            &mut r,
        )
        .unwrap();
    event_mut(
        &mut cfg,
        &game_id("ds3"),
        &event_id("ds3", "Boss defeated on the first try"),
    )
    .jokers = 3;
    let ds3 = cfg
        .games
        .iter_mut()
        .find(|g| g.id == game_id("ds3"))
        .unwrap();
    ds3.events
        .retain(|e| e.id != event_id("ds3", "Boss defeated"));
    s.apply_config(&cfg, &mut r);
    assert!(
        pending(&s, &boss.id).is_none(),
        "the removed event takes its set along"
    );
    assert_eq!(pending(&s, &first.id).unwrap().jokers, 3);
}

#[test]
fn a_set_with_spent_jokers_still_earns_nothing() {
    let (mut s, boss) = session_with_jokers();
    let mut cfg = catalog_config(Language::Fr);
    s.halve(&boss.id, &boss.items[0].exercise_id).unwrap();
    event_mut(&mut cfg, &game_id("ds3"), &event_id("ds3", "Boss defeated")).jokers = 2;
    s.apply_config(&cfg, &mut rng());
    assert_eq!(pending(&s, &boss.id).unwrap().jokers, 0);
}

#[test]
fn an_exercise_switched_off_without_replacement_leaves_the_set() {
    let mut cfg = catalog_config(Language::En);
    cfg.exercises
        .iter_mut()
        .skip(3)
        .for_each(|ex| ex.enabled = false);
    cfg.games[0].events[0].draws = 3;
    let (game, event) = (cfg.games[0].id.clone(), cfg.games[0].events[0].id.clone());
    let mut s = Session::default();
    let draw = s.draw(&cfg, &game, &event, 0, &mut rng()).unwrap();
    let (first, off) = (&draw.items[0], &draw.items[2]);
    s.complete_exercise(&draw.id, &first.exercise_id, 5)
        .unwrap();
    for id in [&first.exercise_id, &off.exercise_id] {
        cfg.exercises
            .iter_mut()
            .find(|ex| &ex.id == id)
            .unwrap()
            .enabled = false;
    }
    s.apply_config(&cfg, &mut rng());
    let ids: Vec<_> = pending(&s, &draw.id)
        .unwrap()
        .items
        .into_iter()
        .map(|it| it.exercise_id)
        .collect();
    assert_eq!(
        ids,
        vec![first.exercise_id.clone(), draw.items[1].exercise_id.clone()],
        "the exercise done stays, the one switched off leaves"
    );
}
