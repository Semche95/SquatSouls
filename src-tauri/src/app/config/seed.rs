//! Seed, `seed.json`: the exercises of the default configuration, whose texts are written in each language.

use crate::app::{Config, Language};
use serde_json::Value;

const SEED: &str = include_str!("seed.json");

/// Fields written in each language in the seed and the files of the catalog.
const TEXTS: [&str; 2] = ["name", "unit"];

/// Keeps only `language` in the texts of `value`, written in each language. Every language has the same ids, numbers
/// and AppIds: only names and units differ.
pub fn localize(value: &mut Value, language: Language) {
    match value {
        Value::Array(items) => items.iter_mut().for_each(|item| localize(item, language)),
        Value::Object(fields) => {
            for (key, field) in fields.iter_mut() {
                if TEXTS.contains(&key.as_str()) && field.is_object() {
                    *field = field[language.as_str()].take();
                } else {
                    localize(field, language);
                }
            }
        }
        _ => {}
    }
}

/// Default configuration in `language`, written to a new database and used if the saved configuration is invalid:
/// the seed's exercises, without any game, each chosen from the catalog or created.
pub fn default_config(language: Language) -> Config {
    let mut seed: Value = serde_json::from_str(SEED).expect("invalid seed");
    seed["games"] = Value::Array(Vec::new());
    localize(&mut seed, language);
    serde_json::from_value(seed).expect("invalid seed")
}
