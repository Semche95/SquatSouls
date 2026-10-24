//! Default configuration, assembled from `app/seed/`: `exercises.json` and one file per game, whose texts are
//! written in each language.

use crate::app::{Config, Language};
use serde_json::Value;

const EXERCISES: &str = include_str!("../seed/exercises.json");
/// Every file of `app/seed/games/`, listed by `build.rs`.
const GAMES: &[&str] = include!(concat!(env!("OUT_DIR"), "/seed_games.rs"));

/// Fields written in each language in the seed files.
const TEXTS: [&str; 2] = ["name", "unit"];

/// Keeps only `language` in the texts of `value`.
fn localize(value: &mut Value, language: &str) {
    match value {
        Value::Array(items) => items.iter_mut().for_each(|item| localize(item, language)),
        Value::Object(fields) => {
            for (key, field) in fields.iter_mut() {
                if TEXTS.contains(&key.as_str()) && field.is_object() {
                    *field = field[language].take();
                } else {
                    localize(field, language);
                }
            }
        }
        _ => {}
    }
}

/// Default configuration in `language`, written to a new database and used if the saved
/// configuration is invalid. Every language has the same ids, numbers and AppIds: only names and units differ.
pub fn default_config(language: Language) -> Config {
    let mut seed: Value = serde_json::from_str(EXERCISES).expect("invalid seed exercises");
    seed["games"] = GAMES
        .iter()
        .map(|game| serde_json::from_str::<Value>(game).expect("invalid seed game"))
        .collect();
    localize(&mut seed, language.as_str());
    serde_json::from_value(seed).expect("invalid seed")
}
