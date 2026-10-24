mod config;
mod current;
mod history;
mod launch;
mod schema;
mod settings;
mod steam;

use squat_souls_lib::app::*;

/// System locale of the tests: new databases receive the French default configuration.
const LOCALE: Option<&str> = Some("fr_FR.UTF-8");

fn store() -> Store {
    Store::open_in_memory(LOCALE).unwrap()
}

/// Steam Store language of the names asked in the tests.
const LANGUAGE: &str = "english";

/// Path of a temporary database specific to test `name`, deleted beforehand.
fn temp_db(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("squatsouls-{name}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn connect(path: &std::path::Path) -> rusqlite::Connection {
    rusqlite::Connection::open(path).unwrap()
}

#[test]
fn new_store_holds_default_config() {
    assert_eq!(store().config().unwrap(), default_config(Language::Fr));
}

#[test]
fn new_store_default_config_follows_the_system_language() {
    let english = Store::open_in_memory(Some("de-DE")).unwrap();
    assert_eq!(english.config().unwrap(), default_config(Language::En));
    let english = Store::open_in_memory(None).unwrap();
    assert_eq!(english.config().unwrap(), default_config(Language::En));
}

#[test]
fn existing_store_keeps_its_config_whatever_the_language() {
    let path = temp_db("language");
    Store::open(&path, LOCALE).unwrap();
    let reopened = Store::open(&path, Some("en-US")).unwrap();
    assert_eq!(reopened.config().unwrap(), default_config(Language::Fr));
}
