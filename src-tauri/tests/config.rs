use squat_souls_lib::fitness::*;

#[test]
fn default_config_is_valid_and_roundtrips() {
    let cfg = default_config();
    cfg.validate().unwrap();
    let json = serde_json::to_string_pretty(&cfg).unwrap();
    assert!(json.contains("maxPerSession"));
    let back: Config = serde_json::from_str(&json).unwrap();
    assert_eq!(back, cfg);
}

#[test]
fn validation_catches_mistakes() {
    let mut cfg = default_config();
    cfg.exercises[0].min = 20;
    assert!(cfg.validate().unwrap_err().contains("Invalid range"));

    let mut cfg = default_config();
    cfg.games[0].events[0].draws = cfg.exercises.len() as u32 + 1;
    assert!(cfg.validate().is_err());

    let mut cfg = default_config();
    cfg.games[1].id = "elden-ring".into();
    assert!(cfg.validate().unwrap_err().contains("defined twice"));

    let mut cfg = default_config();
    cfg.games[1].name = format!(" {} ", cfg.games[0].name.to_uppercase());
    assert!(cfg.validate().unwrap_err().contains("Two games"));

    // Several games can share a Steam AppId: the same game, configured for different challenges.
    let mut cfg = default_config();
    cfg.games[1].steam_app_id = cfg.games[0].steam_app_id;
    cfg.validate().unwrap();

    let mut cfg = default_config();
    cfg.exercises.clear();
    assert!(cfg.validate().unwrap_err().contains("No exercise"));

    let mut cfg = default_config();
    cfg.exercises.iter_mut().for_each(|ex| ex.enabled = false);
    assert!(cfg.validate().unwrap_err().contains("No exercise checked"));
}

#[test]
fn draws_are_limited_by_the_checked_exercises() {
    let mut cfg = default_config();
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
    assert!(cfg.validate().unwrap_err().contains("but only"));
}

#[test]
fn exercises_are_checked_by_default() {
    let mut cfg = default_config();
    assert!(cfg.exercises.iter().all(|ex| ex.enabled));
    cfg.exercises[0].enabled = false;
    let json = serde_json::to_value(&cfg).unwrap();
    let mut ex = json["exercises"][0].clone();
    assert_eq!(ex["enabled"], false);
    ex.as_object_mut().unwrap().remove("enabled");
    assert!(serde_json::from_value::<Exercise>(ex).unwrap().enabled);
}
