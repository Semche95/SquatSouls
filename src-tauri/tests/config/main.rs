#[path = "../catalog_config/mod.rs"]
mod catalog_config;
mod repair;
mod schemas;
mod validation;

use catalog_config::{catalog_config, game_id};
use squat_souls_lib::app::*;

/// The games of the catalog, with the seed's exercises, make a valid configuration in every language.
#[test]
fn catalog_configs_are_valid_and_roundtrip() {
    for language in [Language::En, Language::Fr] {
        let cfg = catalog_config(language);
        cfg.validate().unwrap();
        let json = serde_json::to_string_pretty(&cfg).unwrap();
        assert!(json.contains("maxPerSession"));
        let back: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cfg);
    }
}

/// The seed and the catalog only differ between languages by their texts: names of exercises and events, and units.
#[test]
fn catalog_configs_have_the_same_structure() {
    let untranslated = |language| {
        let mut cfg = catalog_config(language);
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
    assert_ne!(catalog_config(Language::En), catalog_config(Language::Fr));
}

/// Games are chosen from the catalog or created: the default configuration only holds the seed's exercises.
#[test]
fn default_configs_hold_the_seed_without_games() {
    for language in [Language::En, Language::Fr] {
        let cfg = default_config(language);
        cfg.validate().unwrap();
        assert!(cfg.games.is_empty());
        assert!(!cfg.exercises.is_empty());
    }
}

#[test]
fn exercises_are_enabled_by_default() {
    let mut cfg = catalog_config(Language::Fr);
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
