use super::*;

/// French default configuration, Elden Ring first: the errors below name its first event.
fn config() -> Config {
    let mut cfg = default_config(Language::Fr);
    cfg.games.sort_by_key(|game| game.id != "elden-ring");
    cfg
}

#[test]
fn validation_catches_mistakes() {
    let mut cfg = config();
    cfg.exercises[0].min = 20;
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("exercise_range")
            .with("name", "Pompes")
            .with("min", 20)
            .with("max", 12)
            .with("limit", MAX_AMOUNT)
    );

    let mut cfg = config();
    cfg.games[0].events[0].draws = MAX_ITEMS as u32 + 1;
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("event_too_many_draws")
            .with("game", "Elden Ring")
            .with("event", "Mort")
            .with("max", MAX_ITEMS)
    );

    let mut cfg = config();
    cfg.games[1].id = "elden-ring".into();
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("game_duplicate").with("id", "elden-ring")
    );

    let mut cfg = config();
    cfg.games[1].name = format!(" {} ", cfg.games[0].name.to_uppercase());
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("game_name_duplicate").with("name", "ELDEN RING")
    );

    // A game without event is saved, the Play tab does not offer it until it has one.
    let mut cfg = config();
    cfg.games[0].events.clear();
    cfg.validate().unwrap();

    // Without any game, the configuration is saved: Play and Program say so.
    let mut cfg = default_config(Language::En);
    cfg.games.clear();
    cfg.validate().unwrap();

    // Several games can share a Steam AppId: the same game, configured for different challenges.
    let mut cfg = config();
    cfg.games[1].steam_app_id = cfg.games[0].steam_app_id;
    cfg.validate().unwrap();

    // Every exercise can be switched off or removed, the last one included.
    let mut cfg = config();
    cfg.exercises.iter_mut().for_each(|ex| ex.enabled = false);
    cfg.validate().unwrap();
    cfg.exercises.clear();
    cfg.validate().unwrap();

    let mut cfg = config();
    cfg.games[0].events[0].max_per_session = Some(0);
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("event_zero_limit")
            .with("game", "Elden Ring")
            .with("event", "Mort")
    );
}

#[test]
fn numbers_are_limited() {
    let mut cfg = config();
    cfg.recovery = MAX_SECONDS;
    cfg.timer_prep = MAX_SECONDS;
    cfg.rep_duration = MAX_SECONDS;
    cfg.exercises[0].min = MAX_AMOUNT;
    cfg.exercises[0].max = MAX_AMOUNT;
    cfg.exercises[0].recovery = Some(MAX_SECONDS);
    cfg.games[0].events[0].max_per_session = Some(MAX_AMOUNT);
    cfg.games[0].events[0].jokers = MAX_AMOUNT;
    cfg.validate().unwrap();

    let range = |min: u32, max: u32| {
        AppError::new("exercise_range")
            .with("name", "Pompes")
            .with("min", min)
            .with("max", max)
            .with("limit", MAX_AMOUNT)
    };
    for (min, max) in [(0, 5), (1, MAX_AMOUNT + 1)] {
        let mut cfg = config();
        cfg.exercises[0].min = min;
        cfg.exercises[0].max = max;
        assert_eq!(cfg.validate().unwrap_err(), range(min, max));
    }

    let too_long = |change: fn(&mut Config), error: AppError| {
        let mut cfg = config();
        change(&mut cfg);
        assert_eq!(cfg.validate().unwrap_err(), error.with("max", MAX_SECONDS));
    };
    too_long(
        |cfg| cfg.recovery = MAX_SECONDS + 1,
        AppError::new("recovery_too_long"),
    );
    too_long(
        |cfg| cfg.rep_duration = MAX_SECONDS + 1,
        AppError::new("rep_duration_range"),
    );
    too_long(
        |cfg| cfg.rep_duration = 0,
        AppError::new("rep_duration_range"),
    );
    too_long(
        |cfg| cfg.timer_prep = MAX_SECONDS + 1,
        AppError::new("timer_prep_too_long"),
    );
    too_long(
        |cfg| cfg.exercises[0].recovery = Some(MAX_SECONDS + 1),
        AppError::new("exercise_recovery_too_long").with("name", "Pompes"),
    );

    let too_high = |change: fn(&mut GameEvent), code: &'static str| {
        let mut cfg = config();
        change(&mut cfg.games[0].events[0]);
        assert_eq!(
            cfg.validate().unwrap_err(),
            AppError::new(code)
                .with("game", "Elden Ring")
                .with("event", "Mort")
                .with("max", MAX_AMOUNT)
        );
    };
    too_high(
        |ev| ev.max_per_session = Some(MAX_AMOUNT + 1),
        "event_limit_too_high",
    );
    too_high(|ev| ev.jokers = MAX_AMOUNT + 1, "event_too_many_jokers");
}

#[test]
fn names_units_and_counts_are_limited() {
    let mut cfg = config();
    cfg.games[0].name = "E".repeat(MAX_NAME);
    cfg.exercises[0].unit = "r".repeat(MAX_UNIT);
    cfg.validate().unwrap();

    cfg.games[0].name.push('E');
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("name_too_long")
            .with("name", format!("{}…", "E".repeat(40)))
            .with("max", MAX_NAME)
    );

    let mut cfg = config();
    cfg.exercises[0].unit.push_str(&"r".repeat(MAX_UNIT));
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("unit_too_long")
            .with("name", "Pompes")
            .with("max", MAX_UNIT)
    );

    for hidden in ['\n', '\t', '\u{200B}', '\u{202E}', '\u{FEFF}'] {
        let mut cfg = config();
        cfg.games[0].events[0].name = format!("Mort{hidden}");
        assert_eq!(cfg.validate().unwrap_err().code, "name_hidden_char");
        let mut cfg = config();
        cfg.exercises[0].unit.push(hidden);
        assert_eq!(
            cfg.validate().unwrap_err(),
            AppError::new("unit_hidden_char").with("name", "Pompes")
        );
    }

    // Emoji sequences join their characters with a zero width joiner.
    let mut cfg = config();
    cfg.games[0].name = "Elden Ring 👨‍👩‍👧".into();
    cfg.validate().unwrap();

    let mut cfg = config();
    let model = cfg.exercises[0].clone();
    cfg.exercises = (0..=MAX_ITEMS)
        .map(|i| Exercise {
            id: format!("ex-{i}"),
            ..model.clone()
        })
        .collect();
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("too_many_exercises").with("max", MAX_ITEMS)
    );

    let mut cfg = config();
    cfg.games[0].events = (0..=MAX_ITEMS)
        .map(|i| event(&format!("ev-{i}"), "Mort", 1, None))
        .collect();
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("too_many_events")
            .with("game", "Elden Ring")
            .with("max", MAX_ITEMS)
    );
}

#[test]
fn draws_are_not_limited_by_the_enabled_exercises() {
    let mut cfg = config();
    cfg.exercises
        .iter_mut()
        .skip(1)
        .for_each(|ex| ex.enabled = false);
    cfg.games[0].events[0].draws = MAX_ITEMS as u32;
    cfg.validate().unwrap();
}
