use super::*;

#[test]
fn complete_dismiss_and_totals() {
    let cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let a = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    let b = s
        .draw(&cfg, &game_id("ds3"), &event_id("ds3", "Death"), 0, &mut r)
        .unwrap();
    let c = s
        .draw(&cfg, &game_id("ds3"), &event_id("ds3", "Death"), 0, &mut r)
        .unwrap();

    let logged = complete(&mut s, &a.id, 100).unwrap();
    assert_eq!(logged.len(), 2);
    complete(&mut s, &b.id, 101).unwrap();
    s.dismiss(&c.id).unwrap();
    assert_eq!(
        complete(&mut s, &c.id, 102).unwrap_err().code,
        "set_not_pending"
    );

    let v = s.view();
    assert!(v.pending.is_empty());
    assert_eq!(v.sets, 3);
    assert_eq!(v.log[0].event, "Mort");
    let count = v
        .triggers
        .iter()
        .find(|t| t.event_id == event_id("ds3", "Death"))
        .unwrap()
        .count;
    assert_eq!(count, 1, "dismissing removes the trigger");
    let sum: u32 = v.totals.iter().map(|t| t.sets).sum();
    assert_eq!(sum, 3);
}

#[test]
fn sets_are_done_in_the_order_they_were_drawn() {
    let (mut s, boss) = session_with_jokers();
    let cfg = catalog_config(Language::Fr);
    let death = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Death"),
            0,
            &mut rng(),
        )
        .unwrap();
    let id = &death.items[0].exercise_id;
    for refused in [
        s.complete_exercise(&death.id, id, 20).map(|_| ()),
        s.halve(&death.id, id),
        s.skip_exercise(&death.id, id),
        s.skip_set(&death.id),
    ] {
        assert_eq!(refused.unwrap_err().code, "set_not_current");
    }
    s.dismiss(&death.id).unwrap();
    s.complete_exercise(&boss.id, &boss.items[0].exercise_id, 30)
        .unwrap();
}

#[test]
fn exercises_are_done_one_by_one() {
    let (mut s, nohit) = session_with_jokers_on("Boss defeated without getting hit");
    let (first, second) = (&nohit.items[0], &nohit.items[1]);
    assert_eq!(
        s.complete_exercise(&nohit.id, &second.exercise_id, 20)
            .unwrap_err()
            .code,
        "exercise_not_next",
        "in the order of the set"
    );
    let entry = s
        .complete_exercise(&nohit.id, &first.exercise_id, 20)
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
        s.complete_exercise(&nohit.id, &first.exercise_id, 21)
            .map(|_| ()),
        s.halve(&nohit.id, &first.exercise_id),
        s.skip_exercise(&nohit.id, &first.exercise_id),
    ] {
        assert_eq!(again.unwrap_err().code, "exercise_already_done");
    }

    s.complete_exercise(&nohit.id, &second.exercise_id, 30)
        .unwrap();
    let v = s.view();
    assert_eq!(v.jokers, 4, "set finished: the reward is earned");
    assert!(v.pending.is_empty());
    assert_eq!(v.log[0].at, 30);

    let cfg = catalog_config(Language::Fr);
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut rng(),
        )
        .unwrap();
    s.complete_exercise(&boss.id, &boss.items[0].exercise_id, 40)
        .unwrap();
    s.skip_set(&boss.id).unwrap();
    let v = s.view();
    assert_eq!(
        v.jokers,
        4 - SKIP_COST,
        "skipping the rest only costs the remaining exercise"
    );
    assert_eq!(v.jokers_spent, SKIP_COST);
    assert_eq!(v.sets, 5);
}

#[test]
fn dismissing_a_started_set_keeps_its_done_exercises() {
    let cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let started = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated without getting hit"),
            0,
            &mut r,
        )
        .unwrap();
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated without getting hit"),
            0,
            &mut r,
        )
        .unwrap();
    let entry = s
        .complete_exercise(&started.id, &started.items[0].exercise_id, 10)
        .unwrap();
    s.dismiss(&started.id).unwrap();
    let v = s.view();
    assert_eq!(v.pending.len(), 1, "the rest of the set is dropped");
    assert_eq!(v.pending[0].id, boss.id);
    assert_eq!(v.log, vec![entry], "the exercise done stays counted");
    assert_eq!(v.triggers[0].count, 2, "the trigger stays counted");
    assert_eq!(v.jokers, 0, "a stopped set earns no jokers");
    let again = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated without getting hit"),
            0,
            &mut r,
        )
        .unwrap();
    assert_eq!(again.trigger_count, 3);
    assert_ne!(again.items, started.items, "nothing is reserved");
}

#[test]
fn a_dismissed_set_comes_back_on_the_next_trigger_of_its_event() {
    let cfg = catalog_config(Language::Fr);
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
    s.dismiss(&boss.id).unwrap();
    let other = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated on the first try"),
            0,
            &mut r,
        )
        .unwrap();
    assert_ne!(other.items, boss.items, "another event draws normally");
    let again = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    assert_eq!(again.items, boss.items, "same set, as drawn");
    assert_eq!(again.trigger_count, 1);
    s.dismiss(&again.id).unwrap();
    let mut changed = cfg.clone();
    changed
        .exercises
        .retain(|ex| boss.items.iter().all(|it| it.exercise_id != ex.id));
    s.apply_config(&changed, &mut r);
    let fresh = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    assert_ne!(
        fresh.items, boss.items,
        "the saved configuration replaces the removed exercises of the reserve"
    );
    assert_eq!(fresh.items.len(), 2);
}

#[test]
fn dismissing_every_set_keeps_only_the_exercises_done() {
    let cfg = catalog_config(Language::En);
    let mut s = Session::default();
    let mut r = rng();
    let game = &cfg.games[0];
    let event = game.events.iter().find(|e| e.draws > 1).unwrap();
    let started = s.draw(&cfg, &game.id, &event.id, 0, &mut r).unwrap();
    s.draw(&cfg, &game.id, &event.id, 0, &mut r).unwrap();
    let entry = s
        .complete_exercise(&started.id, &started.items[0].exercise_id, 10)
        .unwrap();
    assert!(s.dismiss_all(), "an exercise was done");
    let v = s.view();
    assert!(v.pending.is_empty());
    assert_eq!(v.log, vec![entry], "only the exercise done stays");
    assert_eq!(v.triggers[0].count, 1, "only the started set stays counted");
    assert_eq!(v.jokers, 0, "a stopped set earns no jokers");
    assert_eq!(
        s.event_counts(&cfg),
        vec![EventCount {
            event: event.name.clone(),
            count: 1
        }]
    );

    let mut untouched = Session::default();
    untouched
        .draw(&cfg, &game.id, &event.id, 0, &mut r)
        .unwrap();
    assert!(
        !untouched.dismiss_all(),
        "nothing done: nothing to end with"
    );
    assert!(untouched.view().triggers.is_empty());
}
