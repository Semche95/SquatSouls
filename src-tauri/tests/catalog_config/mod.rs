//! Configuration the tests play with: the default configuration and every game of the catalog, read from `catalog/`.

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
