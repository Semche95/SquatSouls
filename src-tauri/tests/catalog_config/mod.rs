//! Configuration the tests play with: the default configuration and every game of the catalog, read from `catalog/`,
//! and the ids of its games and events, found by file and English name.

// Each test crate uses only some of these.
#![allow(dead_code)]

use serde_json::Value;
use squat_souls_lib::app::*;
use std::{fs, path::PathBuf};

/// Files of the games of the catalog, in the order of their names, as `scripts/catalog.js` publishes them.
fn catalog_files() -> Vec<PathBuf> {
    let folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("catalog/games");
    let mut files: Vec<_> = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
}

/// Game of the catalog file `file` (`ds3` for `games/ds3.json`), as written.
fn catalog_game(file: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("catalog/games/{file}.json"));
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

/// Id of the game of the catalog file `file`.
pub fn game_id(file: &str) -> String {
    catalog_game(file)["id"].as_str().unwrap().into()
}

/// Id of the event named `name` in English of the game of the catalog file `file`.
pub fn event_id(file: &str, name: &str) -> String {
    let game = catalog_game(file);
    let found = game["events"]
        .as_array()
        .unwrap()
        .iter()
        .find(|ev| ev["name"]["en"] == name);
    found.unwrap_or_else(|| panic!("no event {name} in {file}"))["id"]
        .as_str()
        .unwrap()
        .into()
}

/// Default configuration in `language` with every game of the catalog.
pub fn catalog_config(language: Language) -> Config {
    let mut games: Value = catalog_files()
        .iter()
        .map(|path| serde_json::from_str::<Value>(&fs::read_to_string(path).unwrap()).unwrap())
        .collect();
    localize(&mut games, language);
    Config {
        games: serde_json::from_value(games).unwrap(),
        ..default_config(language)
    }
}
