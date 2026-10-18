//! Schema of the seeds, written apart from the structures it describes: it must follow their fields and limits.

use serde_json::{json, Map, Value};
use squat_souls_lib::app::*;

fn schema() -> Value {
    serde_json::from_str(include_str!("../../src/app/seed.schema.json")).unwrap()
}

fn def<'a>(schema: &'a Value, name: &str) -> &'a Value {
    &schema["$defs"][name]
}

fn keys(object: &Map<String, Value>) -> Vec<String> {
    let mut keys: Vec<String> = object.keys().cloned().collect();
    keys.sort();
    keys
}

/// The object, with every optional field set, has exactly the fields of the schema; it is read back
/// with only the required fields of the schema, and refused without any one of them.
fn check<T: serde::de::DeserializeOwned>(name: &str, full: Value) {
    let schema = schema();
    let def = def(&schema, name);
    let full = full.as_object().unwrap();
    let properties = def["properties"].as_object().unwrap();
    assert_eq!(keys(full), keys(properties), "{name}: fields");
    let required: Vec<&str> = def["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| k.as_str().unwrap())
        .collect();
    let minimal: Map<String, Value> = required
        .iter()
        .map(|k| (k.to_string(), full[*k].clone()))
        .collect();
    assert!(
        serde_json::from_value::<T>(Value::Object(minimal.clone())).is_ok(),
        "{name}: optional field required by the backend"
    );
    for key in &required {
        let mut missing = minimal.clone();
        missing.remove(*key);
        assert!(
            serde_json::from_value::<T>(Value::Object(missing)).is_err(),
            "{name}: {key} is optional for the backend"
        );
    }
}

#[test]
fn exercise_follows_the_structure() {
    let exercise = Exercise {
        id: "squat".into(),
        name: "Squats".into(),
        unit: "reps".into(),
        min: 1,
        max: 2,
        recovery: Some(30),
        enabled: true,
        timed: Some(TimeUnit::Seconds),
        each_side: true,
    };
    check::<Exercise>("exercise", serde_json::to_value(exercise).unwrap());
}

#[test]
fn game_and_event_follow_the_structure() {
    let event = GameEvent {
        id: "death".into(),
        name: "Death".into(),
        draws: 1,
        max_per_session: Some(15),
        jokers: 1,
    };
    let game = Game {
        id: "er".into(),
        name: "Elden Ring".into(),
        steam_app_id: Some(1245620),
        events: vec![event.clone()],
    };
    check::<GameEvent>("event", serde_json::to_value(event).unwrap());
    check::<Game>("game", serde_json::to_value(game).unwrap());
}

/// A seed only holds the program: its fields are fields of the configuration.
#[test]
fn seed_fields_belong_to_the_configuration() {
    let schema = schema();
    let config = serde_json::to_value(default_config(Language::En)).unwrap();
    for key in schema["properties"].as_object().unwrap().keys() {
        assert!(config.get(key).is_some(), "{key}");
    }
}

#[test]
fn limits_follow_the_validation() {
    let schema = schema();
    assert_eq!(def(&schema, "name")["maxLength"], json!(MAX_NAME));
    assert_eq!(
        def(&schema, "exercise")["properties"]["unit"]["maxLength"],
        json!(MAX_UNIT)
    );
    assert_eq!(
        schema["properties"]["exercises"]["maxItems"],
        json!(MAX_ITEMS)
    );
    assert_eq!(
        def(&schema, "game")["properties"]["events"]["maxItems"],
        json!(MAX_ITEMS)
    );
    for (bound, min, max) in [
        ("positive", 1, u32::MAX),
        ("amount", 1, MAX_AMOUNT),
        ("count", 0, MAX_AMOUNT),
        ("seconds", 0, MAX_SECONDS),
        ("draws", 1, MAX_ITEMS as u32),
    ] {
        assert_eq!(def(&schema, bound)["minimum"], json!(min), "{bound}");
        assert_eq!(def(&schema, bound)["maximum"], json!(max), "{bound}");
    }
}
