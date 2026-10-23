use super::*;

/// A configuration saved before the limits is brought within them at load rather than refused.
#[test]
fn repair_brings_a_configuration_within_the_limits() {
    let mut cfg = default_config(Language::Fr);
    assert!(!cfg.repair());

    cfg.games[0].name = format!("Elden\u{200B} Ring\n{}", "!".repeat(MAX_NAME));
    cfg.games[0].events[0].name = "\u{202E}\t".into();
    cfg.exercises[0].unit = "r\u{FEFF}".repeat(MAX_UNIT);
    cfg.ignored_games = vec![IgnoredGame {
        app_id: 1,
        name: "Nioh\u{200B}".into(),
    }];
    let event = cfg.games[0].events[0].clone();
    cfg.games[0].events = (0..=MAX_ITEMS)
        .map(|i| GameEvent {
            id: format!("ev-{i}"),
            ..event.clone()
        })
        .collect();
    assert!(cfg.repair());
    cfg.validate().unwrap();
    assert_eq!(
        cfg.games[0].name,
        format!("Elden Ring{}", "!".repeat(MAX_NAME - 10))
    );
    assert_eq!(cfg.games[0].events[0].name, "ev-0");
    assert_eq!(cfg.games[0].events.len(), MAX_ITEMS);
    assert_eq!(cfg.exercises[0].unit, "r".repeat(MAX_UNIT));
    assert_eq!(cfg.ignored_games[0].name, "Nioh");

    // Numbers out of bounds are brought to the nearest bound, the maximum never below the minimum.
    let mut cfg = default_config(Language::Fr);
    cfg.recovery = u32::MAX;
    cfg.timer_prep = u32::MAX;
    cfg.rep_duration = 0;
    cfg.exercises[0].min = 0;
    cfg.exercises[0].max = u32::MAX;
    cfg.exercises[1].min = u32::MAX;
    cfg.exercises[1].max = u32::MAX;
    cfg.exercises[2].recovery = Some(u32::MAX);
    cfg.games[0].events[0].max_per_session = Some(u32::MAX);
    cfg.games[0].events[0].jokers = u32::MAX;
    assert!(cfg.repair());
    cfg.validate().unwrap();
    assert_eq!((cfg.recovery, cfg.timer_prep), (MAX_SECONDS, MAX_SECONDS));
    assert_eq!(cfg.rep_duration, 1);
    assert_eq!(
        (cfg.exercises[0].min, cfg.exercises[0].max),
        (1, MAX_AMOUNT)
    );
    assert_eq!(
        (cfg.exercises[1].min, cfg.exercises[1].max),
        (MAX_AMOUNT, MAX_AMOUNT)
    );
    assert_eq!(cfg.exercises[2].recovery, Some(MAX_SECONDS));
    assert_eq!(cfg.games[0].events[0].max_per_session, Some(MAX_AMOUNT));
    assert_eq!(cfg.games[0].events[0].jokers, MAX_AMOUNT);
}
