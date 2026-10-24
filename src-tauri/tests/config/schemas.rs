//! Schemas of the seed and of the catalog, written apart from the structures they describe: they must follow their
//! fields and limits.

use serde_json::{json, Map, Value};
use squat_souls_lib::app::*;

fn seed() -> Value {
    serde_json::from_str(include_str!("../../src/app/config/seed.schema.json")).unwrap()
}

fn catalog() -> Value {
    serde_json::from_str(include_str!("../../catalog/schema.json")).unwrap()
}

fn def<'a>(schema: &'a Value, name: &str) -> &'a Value {
    &schema["$defs"][name]
}

fn keys(object: &Map<String, Value>) -> Vec<String> {
    let mut keys: Vec<String> = object.keys().cloned().collect();
    keys.sort();
    keys
}

/// The object, with every optional field set, has exactly the fields of `def`; it is read back with only the
/// required fields of `def`, and refused without any one of them.
fn check<T: serde::de::DeserializeOwned>(name: &str, def: &Value, full: Value) {
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
    check::<Exercise>(
        "exercise",
        def(&seed(), "exercise"),
        serde_json::to_value(exercise).unwrap(),
    );
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
    let catalog = catalog();
    check::<GameEvent>(
        "event",
        def(&catalog, "event"),
        serde_json::to_value(event).unwrap(),
    );
    check::<Game>("game", &catalog, serde_json::to_value(game).unwrap());
}

/// The seed only holds the program: its fields are fields of the configuration.
#[test]
fn seed_fields_belong_to_the_configuration() {
    let config = serde_json::to_value(default_config(Language::En)).unwrap();
    for key in seed()["properties"].as_object().unwrap().keys() {
        assert!(config.get(key).is_some(), "{key}");
    }
}

#[test]
fn limits_follow_the_validation() {
    let (seed, catalog) = (seed(), catalog());
    assert_eq!(def(&seed, "name")["maxLength"], json!(MAX_NAME));
    assert_eq!(
        def(&seed, "unit")["additionalProperties"]["maxLength"],
        json!(MAX_UNIT)
    );
    assert_eq!(
        seed["properties"]["exercises"]["maxItems"],
        json!(MAX_ITEMS)
    );
    assert_eq!(
        catalog["properties"]["events"]["maxItems"],
        json!(MAX_ITEMS)
    );
    for (schema, bound, min, max) in [
        (&catalog, "positive", 1, u32::MAX),
        (&seed, "amount", 1, MAX_AMOUNT),
        (&catalog, "count", 0, MAX_AMOUNT),
        (&seed, "seconds", 0, MAX_SECONDS),
        (&catalog, "draws", 1, MAX_ITEMS as u32),
    ] {
        assert_eq!(def(schema, bound)["minimum"], json!(min), "{bound}");
        assert_eq!(def(schema, bound)["maximum"], json!(max), "{bound}");
    }
}

/// Every text of the seed and of the catalog is written in each language of the interface.
#[test]
fn texts_follow_the_languages() {
    let schema = seed();
    let languages = json!([Language::En.as_str(), Language::Fr.as_str()]);
    assert_eq!(def(&schema, "languages")["required"], languages);
    assert_eq!(
        def(&schema, "languages")["propertyNames"]["enum"],
        languages
    );
}
