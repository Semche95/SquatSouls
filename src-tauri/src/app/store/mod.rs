//! SQLite storage, written row by row: configuration, history, session in progress, settings, Steam Store names,
//! schema migrations and the opening of the file at launch, with its copy from the previous launch.

mod config;
mod current;
mod history;
mod launch;
mod schema;
mod settings;
mod steam;

pub use launch::{previous_path, LaunchError, Launched, OpenFailure};
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

    /// Updates the schema and fills a database just created with the default configuration, in the effective
    /// language. An existing database is never filled.
    fn init(mut conn: Connection, system_locale: Option<&str>) -> Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON")?;
        let created: bool =
            conn.query_row("SELECT NOT EXISTS (SELECT 1 FROM sqlite_master)", [], |r| {
                r.get(0)
            })?;
        schema::migrate(&mut conn)?;
        let mut store = Store { conn };
        if created {
            let language = store.language()?.resolve(system_locale);
            store.save_config(&default_config(language))?;
        }
        Ok(store)
    }

    /// Runs `change` as a whole: on failure, nothing it wrote is kept. The store's own writes nest in it.
    pub fn atomically<T>(&mut self, change: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.conn.execute_batch("SAVEPOINT atomically")?;
        match change(self) {
            Ok(value) => {
                self.conn.execute_batch("RELEASE atomically")?;
                Ok(value)
            }
            Err(e) => {
                if let Err(undo) = self
                    .conn
                    .execute_batch("ROLLBACK TO atomically; RELEASE atomically")
                {
                    log::error!("Changes not undone: {undo}");
                }
                Err(e)
            }
        }
    }
}
