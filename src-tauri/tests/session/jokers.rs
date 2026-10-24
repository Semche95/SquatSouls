use super::*;

#[test]
fn skipping_a_set_costs_its_exercises() {
    let (mut s, boss) = session_with_jokers();
    assert_eq!(skip_set_cost(&boss), 2 * SKIP_COST);
    assert!(s.skip_set(&boss.id).is_err(), "two exercises cost 4 jokers");

    s.dismiss(&boss.id).unwrap();
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
    s.skip_set(&death.id).unwrap();
    let v = s.view();
    assert_eq!(v.jokers, 0);
    assert_eq!(v.jokers_earned, 2);
    assert!(v.pending.is_empty());
    assert_eq!(v.sets, 2, "only the no hit set is done");
    let count = v
        .triggers
        .iter()
        .find(|t| t.event_id == event_id("ds3", "Death"))
        .unwrap()
        .count;
    assert_eq!(count, 1, "a skipped death still counts toward the limit");
}

#[test]
fn skipping_an_exercise_keeps_the_rest_of_the_set() {
    let (mut s, boss) = session_with_jokers();
    let (first, second) = (&boss.items[0], &boss.items[1]);
    s.skip_exercise(&boss.id, &first.exercise_id).unwrap();
    let v = s.view();
    assert_eq!(v.jokers, 0);
    assert_eq!(v.pending[0].items, vec![second.clone()]);
    assert!(
        s.skip_exercise(&boss.id, &second.exercise_id).is_err(),
        "plus de joker"
    );
    assert!(
        s.skip_exercise(&boss.id, &first.exercise_id).is_err(),
        "already skipped"
    );

    let entries = complete(&mut s, &boss.id, 40).unwrap();
    assert_eq!(entries.len(), 1, "only the remaining exercise is done");
    assert_eq!(entries[0].amount, second.amount);
}

#[test]
fn halving_then_skipping_costs_a_full_skip() {
    let (mut s, boss) = session_with_jokers();
    let item = &boss.items[0];
    assert!(item.amount >= 2);
    assert_eq!(item.halve_cost, Some(HALVE_COST));
    s.halve(&boss.id, &item.exercise_id).unwrap();
    let v = s.view();
    let halved = &v.pending[0].items[0];
    assert_eq!(halved.amount, item.amount.div_ceil(2), "rounded up");
    assert!(halved.halved);
    assert_eq!(halved.halve_cost, None, "a single half per exercise");
    assert_eq!(halved.skip_cost, SKIP_COST - HALVE_COST);
    assert_eq!(v.jokers, 2 - HALVE_COST);
    assert_eq!(
        s.halve(&boss.id, &item.exercise_id).unwrap_err().code,
        "exercise_not_reducible"
    );

    s.skip_exercise(&boss.id, &item.exercise_id).unwrap();
    assert_eq!(
        s.view().jokers,
        0,
        "the rest of a halved exercise costs the difference"
    );

    let other = &boss.items[1];
    assert!(
        s.halve(&boss.id, &other.exercise_id).is_err(),
        "plus de joker"
    );
}

#[test]
fn spending_jokers_on_a_rewarding_set_forfeits_its_reward() {
    for spend in [Session::halve, Session::skip_exercise] {
        let (mut s, nohit) = session_with_jokers_on("Boss defeated without getting hit");
        let first = nohit
            .items
            .iter()
            .find(|it| it.halve_cost.is_some())
            .unwrap();
        spend(&mut s, &nohit.id, &first.exercise_id).unwrap();
        assert_eq!(s.view().pending[0].jokers, 0, "the reward is lost");
        let left = s.view().jokers;
        complete(&mut s, &nohit.id, 30).unwrap();
        let v = s.view();
        assert_eq!(v.jokers, left, "nothing is earned on completion");
        assert_eq!(v.jokers_earned, 2);
    }
}

#[test]
fn skipping_every_exercise_removes_the_set() {
    let (mut s, death) = session_with_jokers_on("Death");
    s.skip_exercise(&death.id, &death.items[0].exercise_id)
        .unwrap();
    assert!(s.view().pending.iter().all(|d| d.id != death.id));
}

#[test]
fn a_single_unit_cannot_be_halved() {
    let mut cfg = catalog_config(Language::Fr);
    for ex in &mut cfg.exercises {
        ex.min = 1;
        ex.max = 1;
    }
    let d = Session::default()
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Death"),
            0,
            &mut rng(),
        )
        .unwrap();
    assert_eq!(d.items[0].halve_cost, None);
    assert_eq!(
        d.items[0].skip_cost,
        SKIP_COST - HALVE_COST,
        "skipping costs as much as after a half"
    );
}
