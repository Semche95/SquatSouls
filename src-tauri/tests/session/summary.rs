use super::*;

#[test]
fn summary_counts_events_in_game_order_and_jokers() {
    let (mut s, boss) = session_with_jokers();
    s.dismiss(boss.id).unwrap();
    let cfg = seed_config(Language::Fr);
    let death = s.draw(&cfg, "ds3", "mort", 0, &mut rng()).unwrap();
    s.skip_set(death.id).unwrap();
    let names: Vec<_> = s
        .event_counts(&cfg)
        .into_iter()
        .map(|e| (e.event, e.count))
        .collect();
    assert_eq!(
        names,
        vec![
            ("Mort".to_string(), 1),
            ("Boss vaincu sans prendre un coup".to_string(), 1)
        ],
        "the dismissed boss no longer counts"
    );
    assert_eq!(
        s.jokers(),
        Jokers {
            earned: 2,
            spent: SKIP_COST
        }
    );
    assert!(Session::default().event_counts(&cfg).is_empty());
}

#[test]
fn summary_leaves_out_the_sets_to_do_not_started() {
    let cfg = seed_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let boss = s.draw(&cfg, "ds3", "boss", 0, &mut r).unwrap();
    s.draw(&cfg, "ds3", "mort", 0, &mut r).unwrap();
    let counts = |s: &Session| -> Vec<_> {
        s.event_counts(&cfg)
            .into_iter()
            .map(|e| (e.event, e.count))
            .collect()
    };
    assert!(counts(&s).is_empty(), "nothing started yet");
    let v = s.view();
    assert_eq!(v.triggers.len(), 2, "the limits still count both");
    assert!(v.counted_triggers.is_empty());
    s.complete_exercise(boss.id, &boss.items[0].exercise_id, 5)
        .unwrap();
    assert_eq!(counts(&s), vec![("Boss vaincu".to_string(), 1)]);
    let v = s.view();
    assert_eq!(v.counted_triggers.len(), 1);
    assert_eq!(v.counted_triggers[0].event_id, "boss");
}

#[test]
fn summary_counts_the_jokers_spent_on_a_set_once_started() {
    let (mut s, boss) = session_with_jokers();
    s.halve(boss.id, &boss.items[1].exercise_id).unwrap();
    assert_eq!(s.jokers().spent, 0, "the set is not started yet");
    assert_eq!(s.view().jokers_spent, 0);
    assert_eq!(s.view().jokers, 2 - HALVE_COST, "spent all the same");
    s.complete_exercise(boss.id, &boss.items[0].exercise_id, 20)
        .unwrap();
    assert_eq!(s.jokers().spent, HALVE_COST);
}

#[test]
fn the_jokers_spent_on_a_set_dismissed_before_being_started_never_count() {
    let (mut s, boss) = session_with_jokers();
    s.halve(boss.id, &boss.items[0].exercise_id).unwrap();
    s.dismiss(boss.id).unwrap();
    assert_eq!(
        s.jokers(),
        Jokers {
            earned: 2,
            spent: 0
        }
    );
}

#[test]
fn the_duration_counts_from_the_start_without_the_time_the_app_was_closed() {
    let cfg = seed_config(Language::Fr);
    let mut s = Session::default();
    assert_eq!(s.view().counted_from, None, "not started");
    s.start(&cfg, "ds3", 1000).unwrap();
    assert_eq!(s.view().counted_from, Some(1000));
    s.resume(1060, 5000);
    assert_eq!(s.view().counted_from, Some(5000 - 60));
    s.resume(5100, 9000);
    assert_eq!(s.view().counted_from, Some(9000 - 160));
}
