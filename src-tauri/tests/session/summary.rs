use super::*;

#[test]
fn summary_counts_events_in_game_order_and_jokers() {
    let (mut s, boss) = session_with_jokers();
    s.dismiss(boss.id).unwrap();
    let cfg = default_config(Language::Fr);
    let death = s.draw(&cfg, "ds3", "mort", &mut rng()).unwrap();
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
