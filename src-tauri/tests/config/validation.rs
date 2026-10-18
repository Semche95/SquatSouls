use super::*;

#[test]
fn validation_catches_mistakes() {
    let mut cfg = default_config(Language::Fr);
    cfg.exercises[0].min = 20;
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("exercise_range")
            .with("name", "Pompes")
            .with("min", 20)
            .with("max", 12)
    );

    let mut cfg = default_config(Language::Fr);
    let draws = cfg.exercises.len() as u32 + 1;
    cfg.games[0].events[0].draws = draws;
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("event_too_many_draws")
            .with("game", "Elden Ring")
            .with("event", "Mort")
            .with("draws", draws)
            .with("active", cfg.exercises.len())
    );

    let mut cfg = default_config(Language::Fr);
    cfg.games[1].id = "elden-ring".into();
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("game_duplicate").with("id", "elden-ring")
    );

    let mut cfg = default_config(Language::Fr);
    cfg.games[1].name = format!(" {} ", cfg.games[0].name.to_uppercase());
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("game_name_duplicate").with("name", "ELDEN RING")
    );

    // A game without event is saved, the Play tab does not offer it until it has one.
    let mut cfg = default_config(Language::Fr);
    cfg.games[0].events.clear();
    cfg.validate().unwrap();

    // Several games can share a Steam AppId: the same game, configured for different challenges.
    let mut cfg = default_config(Language::Fr);
    cfg.games[1].steam_app_id = cfg.games[0].steam_app_id;
    cfg.validate().unwrap();

    let mut cfg = default_config(Language::Fr);
    cfg.exercises.clear();
    assert_eq!(cfg.validate().unwrap_err(), AppError::new("no_exercise"));

    let mut cfg = default_config(Language::Fr);
    cfg.exercises.iter_mut().for_each(|ex| ex.enabled = false);
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("no_active_exercise")
    );

    let mut cfg = default_config(Language::Fr);
    cfg.games[0].events[0].max_per_session = Some(0);
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("event_zero_limit")
            .with("game", "Elden Ring")
            .with("event", "Mort")
    );
}

#[test]
fn names_units_and_counts_are_limited() {
    let mut cfg = default_config(Language::Fr);
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

    let mut cfg = default_config(Language::Fr);
    cfg.exercises[0].unit.push_str(&"r".repeat(MAX_UNIT));
    assert_eq!(
        cfg.validate().unwrap_err(),
        AppError::new("unit_too_long")
            .with("name", "Pompes")
            .with("max", MAX_UNIT)
    );

    for hidden in ['\n', '\t', '\u{200B}', '\u{202E}', '\u{FEFF}'] {
        let mut cfg = default_config(Language::Fr);
        cfg.games[0].events[0].name = format!("Mort{hidden}");
        assert_eq!(cfg.validate().unwrap_err().code, "name_hidden_char");
        let mut cfg = default_config(Language::Fr);
        cfg.exercises[0].unit.push(hidden);
        assert_eq!(
            cfg.validate().unwrap_err(),
            AppError::new("unit_hidden_char").with("name", "Pompes")
        );
    }

    // Emoji sequences join their characters with a zero width joiner.
    let mut cfg = default_config(Language::Fr);
    cfg.games[0].name = "Elden Ring 👨‍👩‍👧".into();
    cfg.validate().unwrap();

    let mut cfg = default_config(Language::Fr);
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

    let mut cfg = default_config(Language::Fr);
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
fn draws_are_limited_by_the_enabled_exercises() {
    let mut cfg = default_config(Language::Fr);
    let draws = cfg
        .games
        .iter()
        .flat_map(|g| &g.events)
        .map(|ev| ev.draws as usize)
        .max()
        .unwrap();
    let disabled = cfg.exercises.len() - draws;
    cfg.exercises
        .iter_mut()
        .take(disabled)
        .for_each(|ex| ex.enabled = false);
    cfg.validate().unwrap();
    cfg.exercises[disabled].enabled = false;
    assert_eq!(cfg.validate().unwrap_err().code, "event_too_many_draws");
}
