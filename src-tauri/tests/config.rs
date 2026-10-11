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
    assert!(cfg.validate().unwrap_err().contains("Intervalle"));

    let mut cfg = default_config();
    cfg.games[0].events[0].draws = cfg.exercises.len() as u32 + 1;
    assert!(cfg.validate().is_err());

    let mut cfg = default_config();
    cfg.games[1].id = "elden-ring".into();
    assert!(cfg.validate().unwrap_err().contains("deux fois"));

    let mut cfg = default_config();
    cfg.exercises.clear();
    assert!(cfg.validate().unwrap_err().contains("Aucun exercice"));
}
