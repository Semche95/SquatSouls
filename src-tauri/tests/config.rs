use squat_souls_lib::app::*;

#[test]
fn default_configs_are_valid_and_roundtrip() {
    for language in [Language::En, Language::Fr] {
        let cfg = default_config(language);
        cfg.validate().unwrap();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        assert!(json.contains("maxPerSession"));
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cfg);
    }
}

/// Both seeds only differ by their texts: names of exercises and events, and units.
#[test]
fn default_configs_have_the_same_structure() {
    let untranslated = |language| {
        let mut cfg = default_config(language);
        for ex in &mut cfg.exercises {
            ex.name.clear();
            ex.unit.clear();
        }
        for ev in cfg.games.iter_mut().flat_map(|g| &mut g.events) {
            ev.name.clear();
        }
        cfg
    };
    assert_eq!(untranslated(Language::En), untranslated(Language::Fr));
    assert_ne!(default_config(Language::En), default_config(Language::Fr));
}

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
fn draws_are_limited_by_the_checked_exercises() {
    let mut cfg = default_config(Language::Fr);
    let draws = cfg
        .games
        .iter()
        .flat_map(|g| &g.events)
        .map(|ev| ev.draws as usize)
        .max()
        .unwrap();
    let unchecked = cfg.exercises.len() - draws;
    cfg.exercises
        .iter_mut()
        .take(unchecked)
        .for_each(|ex| ex.enabled = false);
    cfg.validate().unwrap();
    cfg.exercises[unchecked].enabled = false;
    assert_eq!(cfg.validate().unwrap_err().code, "event_too_many_draws");
}

#[test]
fn exercises_are_checked_by_default() {
    let mut cfg = default_config(Language::Fr);
    assert!(cfg.exercises.iter().all(|ex| ex.enabled));
    cfg.exercises[0].enabled = false;
    let json = serde_json::to_value(&cfg).unwrap();
    let mut ex = json["exercises"][0].clone();
    assert_eq!(ex["enabled"], false);
    ex.as_object_mut().unwrap().remove("enabled");
    assert!(serde_json::from_value::<Exercise>(ex).unwrap().enabled);
}

#[test]
fn errors_serialize_as_code_and_params() {
    let error = AppError::new("exercise_range")
        .with("name", "Squats")
        .with("min", 3);
    assert_eq!(
        serde_json::to_value(&error).unwrap(),
        serde_json::json!({ "code": "exercise_range", "params": { "name": "Squats", "min": 3 } })
    );
}
