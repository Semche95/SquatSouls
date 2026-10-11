//! Stockage SQLite : configuration, historique et réglages, écrits ligne par ligne.

use super::{
    active_days, default_config, last_days, window_start, Config, DayTotals, Exercise, Game,
    GameEvent, LogEntry,
};
use chrono::NaiveDate;
use rusqlite::{params, Connection, Result, Transaction};
use std::collections::HashMap;
use std::path::Path;

const SCHEMA: &str = "
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS exercises (
    id TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    name TEXT NOT NULL,
    unit TEXT NOT NULL,
    min INTEGER NOT NULL,
    max INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    name TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS events (
    game TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    id TEXT NOT NULL,
    position INTEGER NOT NULL,
    name TEXT NOT NULL,
    draws INTEGER NOT NULL,
    max_per_session INTEGER,
    jokers INTEGER NOT NULL,
    PRIMARY KEY (game, id)
);
CREATE TABLE IF NOT EXISTS sets (
    id INTEGER PRIMARY KEY,
    at INTEGER NOT NULL,
    session INTEGER NOT NULL,
    game TEXT NOT NULL,
    event TEXT NOT NULL,
    exercise TEXT NOT NULL,
    unit TEXT NOT NULL,
    amount INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS sets_at ON sets(at);
CREATE INDEX IF NOT EXISTS sets_session ON sets(session);
CREATE TABLE IF NOT EXISTS session_ends (
    session INTEGER PRIMARY KEY,
    ended INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

const LAST_GAME: &str = "last_game";

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    /// Crée les tables au besoin et remplit une base neuve avec la configuration par défaut.
    fn init(conn: Connection) -> Result<Self> {
        conn.execute_batch(SCHEMA)?;
        let mut store = Store { conn };
        let empty: bool =
            store
                .conn
                .query_row("SELECT NOT EXISTS (SELECT 1 FROM games)", [], |r| r.get(0))?;
        if empty {
            store.save_config(&default_config())?;
        }
        Ok(store)
    }

    pub fn config(&self) -> Result<Config> {
        let exercises = self
            .conn
            .prepare("SELECT id, name, unit, min, max FROM exercises ORDER BY position")?
            .query_map([], |r| {
                Ok(Exercise {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    unit: r.get(2)?,
                    min: r.get(3)?,
                    max: r.get(4)?,
                })
            })?
            .collect::<Result<_>>()?;
        let mut events = self.conn.prepare(
            "SELECT id, name, draws, max_per_session, jokers FROM events WHERE game = ?1 ORDER BY position",
        )?;
        let games = self
            .conn
            .prepare("SELECT id, name FROM games ORDER BY position")?
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get(1)?)))?
            .map(|row| {
                let (id, name) = row?;
                let events = events
                    .query_map([&id], |r| {
                        Ok(GameEvent {
                            id: r.get(0)?,
                            name: r.get(1)?,
                            draws: r.get(2)?,
                            max_per_session: r.get(3)?,
                            jokers: r.get(4)?,
                        })
                    })?
                    .collect::<Result<_>>()?;
                Ok(Game { id, name, events })
            })
            .collect::<Result<_>>()?;
        Ok(Config { exercises, games })
    }

    /// N'écrit que les lignes ajoutées, modifiées ou déplacées, et efface celles qui ont disparu.
    pub fn save_config(&mut self, config: &Config) -> Result<()> {
        let old = self.config()?;
        let tx = self.conn.transaction()?;
        sync(
            &tx,
            &old.exercises,
            &config.exercises,
            |e| e.id.as_str(),
            "INSERT INTO exercises (id, position, name, unit, min, max) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT (id) DO UPDATE SET position = ?2, name = ?3, unit = ?4, min = ?5, max = ?6",
            |stmt, pos, e| stmt.execute(params![e.id, pos, e.name, e.unit, e.min, e.max]),
            "DELETE FROM exercises WHERE id = ?1",
            |stmt, e| stmt.execute([&e.id]),
        )?;
        // Le nom seul est comparé : les événements sont synchronisés à part, jeu par jeu.
        let heads = |games: &[Game]| -> Vec<(String, String)> {
            games
                .iter()
                .map(|g| (g.id.clone(), g.name.clone()))
                .collect()
        };
        sync(
            &tx,
            &heads(&old.games),
            &heads(&config.games),
            |(id, _)| id.as_str(),
            "INSERT INTO games (id, position, name) VALUES (?1, ?2, ?3)
             ON CONFLICT (id) DO UPDATE SET position = ?2, name = ?3",
            |stmt, pos, (id, name)| stmt.execute(params![id, pos, name]),
            "DELETE FROM games WHERE id = ?1",
            |stmt, (id, _)| stmt.execute([id]),
        )?;
        for game in &config.games {
            let before = old.games.iter().find(|g| g.id == game.id);
            sync(
                &tx,
                before.map_or(&[][..], |g| &g.events),
                &game.events,
                |e| e.id.as_str(),
                "INSERT INTO events (game, id, position, name, draws, max_per_session, jokers)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT (game, id) DO UPDATE SET
                 position = ?3, name = ?4, draws = ?5, max_per_session = ?6, jokers = ?7",
                |stmt, pos, e| {
                    stmt.execute(params![
                        game.id,
                        e.id,
                        pos,
                        e.name,
                        e.draws,
                        e.max_per_session,
                        e.jokers
                    ])
                },
                "DELETE FROM events WHERE game = ?1 AND id = ?2",
                |stmt, e| stmt.execute([&game.id, &e.id]),
            )?;
        }
        tx.commit()
    }

    pub fn append_history(&mut self, entries: &[LogEntry]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO sets (at, session, game, event, exercise, unit, amount)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;
            for e in entries {
                stmt.execute(params![
                    e.at, e.session, e.game, e.event, e.exercise, e.unit, e.amount
                ])?;
            }
        }
        tx.commit()
    }

    pub fn end_session(&self, session: i64, ended: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO session_ends (session, ended) VALUES (?1, ?2)",
            params![session, ended],
        )?;
        Ok(())
    }

    /// Retire de l'historique les séries et la fin d'une séance.
    pub fn remove_session(&mut self, session: i64) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM sets WHERE session = ?1", [session])?;
        tx.execute("DELETE FROM session_ends WHERE session = ?1", [session])?;
        tx.commit()
    }

    /// Totaux des `days` derniers jours (aujourd'hui compris), du plus récent au plus ancien.
    pub fn history(&self, days: u32, today: NaiveDate) -> Result<Vec<DayTotals>> {
        let (entries, ends) = self.sets_since(window_start(days, today))?;
        Ok(last_days(&entries, &ends, days, today))
    }

    /// Tous les jours où des séries ont été faites, du plus récent au plus ancien.
    pub fn full_history(&self) -> Result<Vec<DayTotals>> {
        let (entries, ends) = self.sets_since(i64::MIN)?;
        Ok(active_days(&entries, &ends))
    }

    /// Séries faites depuis `from`, et heures de fin de leurs séances terminées.
    fn sets_since(&self, from: i64) -> Result<(Vec<LogEntry>, HashMap<i64, i64>)> {
        let entries = self
            .conn
            .prepare(
                "SELECT at, session, game, event, exercise, unit, amount FROM sets
                 WHERE at >= ?1 ORDER BY at, id",
            )?
            .query_map([from], |r| {
                Ok(LogEntry {
                    at: r.get(0)?,
                    session: r.get(1)?,
                    game: r.get(2)?,
                    event: r.get(3)?,
                    exercise: r.get(4)?,
                    unit: r.get(5)?,
                    amount: r.get(6)?,
                })
            })?
            .collect::<Result<_>>()?;
        let ends = self
            .conn
            .prepare(
                "SELECT session, ended FROM session_ends
                 WHERE session IN (SELECT session FROM sets WHERE at >= ?1)",
            )?
            .query_map([from], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_>>()?;
        Ok((entries, ends))
    }

    /// Dernier jeu choisi, vide s'il n'y en a pas encore.
    pub fn last_game(&self) -> Result<String> {
        match self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [LAST_GAME],
            |r| r.get(0),
        ) {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(String::new()),
            other => other,
        }
    }

    pub fn set_last_game(&self, game_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            [LAST_GAME, game_id],
        )?;
        Ok(())
    }
}

/// Synchronise une table avec une liste ordonnée : une ligne n'est réécrite que si elle
/// diffère de celle qui occupait la même position, et les clés disparues sont effacées.
#[allow(clippy::too_many_arguments)]
fn sync<T: PartialEq>(
    tx: &Transaction,
    old: &[T],
    new: &[T],
    key: impl Fn(&T) -> &str,
    upsert_sql: &str,
    upsert: impl Fn(&mut rusqlite::Statement, i64, &T) -> Result<usize>,
    delete_sql: &str,
    delete: impl Fn(&mut rusqlite::Statement, &T) -> Result<usize>,
) -> Result<()> {
    let mut stmt = tx.prepare(delete_sql)?;
    for item in old.iter().filter(|o| !new.iter().any(|n| key(n) == key(o))) {
        delete(&mut stmt, item)?;
    }
    let mut stmt = tx.prepare(upsert_sql)?;
    for (pos, item) in new.iter().enumerate() {
        if old.get(pos) != Some(item) {
            upsert(&mut stmt, pos as i64, item)?;
        }
    }
    Ok(())
}
