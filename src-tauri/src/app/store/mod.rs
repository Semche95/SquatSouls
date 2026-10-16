//! SQLite storage, written row by row: configuration, history, settings, Steam Store names and schema migrations.

mod config;
mod history;
mod schema;
mod settings;
mod steam;

pub use schema::PRE_TRANSLATION_CHECKSUMS;

use super::default_config;
use rusqlite::{Connection, Result};
use std::path::Path;

pub struct Store {
    conn: Connection,
}

impl Store {
    /// `system_locale` chooses the language of the default configuration written to a new database.
    pub fn open(path: &Path, system_locale: Option<&str>) -> Result<Self> {
        Self::init(Connection::open(path)?, system_locale)
    }

    pub fn open_in_memory(system_locale: Option<&str>) -> Result<Self> {
        Self::init(Connection::open_in_memory()?, system_locale)
    }

    /// Updates the schema and fills a new database with the default configuration, in the effective language.
    fn init(mut conn: Connection, system_locale: Option<&str>) -> Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON")?;
        schema::migrate(&mut conn)?;
        let mut store = Store { conn };
        let empty: bool =
            store
                .conn
                .query_row("SELECT NOT EXISTS (SELECT 1 FROM games)", [], |r| r.get(0))?;
        if empty {
            let language = store.language()?.resolve(system_locale);
            store.save_config(&default_config(language))?;
        }
        Ok(store)
    }
}
