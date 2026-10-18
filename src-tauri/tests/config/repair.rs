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
}
