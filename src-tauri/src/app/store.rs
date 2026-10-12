//! SQLite storage: configuration, history and settings, written row by row.

use super::{
    active_days, default_config, last_days, window_start, Config, DayTotals, EventCount, Exercise,
    Game, GameEvent, IgnoredGame, Jokers, LanguageSetting, LogEntry, SessionRecords, SteamName,
    SteamNames, DEFAULT_PAUSE, DEFAULT_STEAM_NOTIFY,
};
use chrono::NaiveDate;
use refinery::Target;
use rusqlite::{params, Connection, Result, Row, Transaction};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

// Schema migrations (`migrations/V<n>__<name>.sql`), dated in `refinery_schema_history`
// when applied. A migration is never modified once created, even before being committed:
// every schema change, fixes included, goes through a new migration.
refinery::embed_migrations!("migrations");

const LAST_GAME: &str = "last_game";
const LANGUAGE: &str = "language";
const PAUSE: &str = "pause";
const STEAM_DETECTION: &str = "steam_detection";
const STEAM_NOTIFY: &str = "steam_notify";
const TRAY: &str = "tray";
const CLOSE_TO_TRAY: &str = "close_to_tray";
const START_HIDDEN: &str = "start_hidden";
const STEAM_LAST_CALL: &str = "steam_last_call";
const STEAM_BLOCKED_UNTIL: &str = "steam_blocked_until";

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
        migrate(&mut conn)?;
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

    pub fn config(&self) -> Result<Config> {
        let exercises = self
            .conn
            .prepare(
                "SELECT id, name, unit, min, max, pause, enabled FROM exercises ORDER BY position",
            )?
            .query_map([], |r| {
                Ok(Exercise {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    unit: r.get(2)?,
                    min: r.get(3)?,
                    max: r.get(4)?,
                    pause: r.get(5)?,
                    enabled: r.get(6)?,
                })
            })?
            .collect::<Result<_>>()?;
        let mut events = self.conn.prepare(
            "SELECT id, name, draws, max_per_session, jokers FROM events WHERE game = ?1 ORDER BY position",
        )?;
        let games = self
            .conn
            .prepare("SELECT id, name, steam_app_id FROM games ORDER BY position")?
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get(1)?, r.get(2)?)))?
            .map(|row| {
                let (id, name, steam_app_id) = row?;
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
                Ok(Game {
                    id,
                    name,
                    steam_app_id,
                    events,
                })
            })
            .collect::<Result<_>>()?;
        let ignored_games = self
            .conn
            .prepare("SELECT app_id, name FROM ignored_games ORDER BY position")?
            .query_map([], |r| {
                Ok(IgnoredGame {
                    app_id: r.get(0)?,
                    name: r.get(1)?,
                })
            })?
            .collect::<Result<_>>()?;
        Ok(Config {
            pause: self.parsed_setting(PAUSE, DEFAULT_PAUSE)?,
            steam_detection: self.parsed_setting(STEAM_DETECTION, false)?,
            steam_notify: self.parsed_setting(STEAM_NOTIFY, DEFAULT_STEAM_NOTIFY)?,
            tray: self.parsed_setting(TRAY, false)?,
            close_to_tray: self.parsed_setting(CLOSE_TO_TRAY, false)?,
            start_hidden: self.parsed_setting(START_HIDDEN, false)?,
            ignored_games,
            exercises,
            games,
        })
    }

    /// Only writes added, modified or moved rows, and deletes those that are gone.
    pub fn save_config(&mut self, config: &Config) -> Result<()> {
        let old = self.config()?;
        let tx = self.conn.transaction()?;
        let settings = [
            (PAUSE, old.pause.to_string(), config.pause.to_string()),
            (
                STEAM_DETECTION,
                old.steam_detection.to_string(),
                config.steam_detection.to_string(),
            ),
            (
                STEAM_NOTIFY,
                old.steam_notify.to_string(),
                config.steam_notify.to_string(),
            ),
            (TRAY, old.tray.to_string(), config.tray.to_string()),
            (
                CLOSE_TO_TRAY,
                old.close_to_tray.to_string(),
                config.close_to_tray.to_string(),
            ),
            (
                START_HIDDEN,
                old.start_hidden.to_string(),
                config.start_hidden.to_string(),
            ),
        ];
        for (key, old, new) in settings {
            if old != new {
                set_setting(&tx, key, &new)?;
            }
        }
        // Short list: fully rewritten when it changes, in its order.
        if old.ignored_games != config.ignored_games {
            tx.execute("DELETE FROM ignored_games", [])?;
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO ignored_games (app_id, position, name) VALUES (?1, ?2, ?3)",
            )?;
            for (pos, g) in config.ignored_games.iter().enumerate() {
                stmt.execute(params![g.app_id, pos as i64, g.name])?;
            }
        }
        sync(
            &tx,
            &old.exercises,
            &config.exercises,
            |e| e.id.as_str(),
            "INSERT INTO exercises (id, position, name, unit, min, max, pause, enabled) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT (id) DO UPDATE SET position = ?2, name = ?3, unit = ?4, min = ?5, max = ?6, pause = ?7, enabled = ?8",
            |stmt, pos, e| stmt.execute(params![e.id, pos, e.name, e.unit, e.min, e.max, e.pause, e.enabled]),
            "DELETE FROM exercises WHERE id = ?1",
            |stmt, e| stmt.execute([&e.id]),
        )?;
        // Only the name and the AppId are compared: events are synced separately, game by game.
        let heads = |games: &[Game]| -> Vec<(String, String, Option<u32>)> {
            games
                .iter()
                .map(|g| (g.id.clone(), g.name.clone(), g.steam_app_id))
                .collect()
        };
        sync(
            &tx,
            &heads(&old.games),
            &heads(&config.games),
            |(id, ..)| id.as_str(),
            "INSERT INTO games (id, position, name, steam_app_id) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (id) DO UPDATE SET position = ?2, name = ?3, steam_app_id = ?4",
            |stmt, pos, (id, name, app_id)| stmt.execute(params![id, pos, name, app_id]),
            "DELETE FROM games WHERE id = ?1",
            |stmt, (id, ..)| stmt.execute([id]),
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

    /// Removes one history entry per element of `entries`.
    pub fn remove_history(&mut self, entries: &[LogEntry]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare(
                "DELETE FROM sets WHERE id = (SELECT id FROM sets
                 WHERE at = ?1 AND session = ?2 AND game = ?3 AND event = ?4
                 AND exercise = ?5 AND unit = ?6 AND amount = ?7 LIMIT 1)",
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

    /// Replaces the events triggered and the jokers saved for a session.
    pub fn save_session_summary(
        &mut self,
        session: i64,
        events: &[EventCount],
        jokers: Jokers,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM session_events WHERE session = ?1", [session])?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO session_events (session, position, event, count) VALUES (?1, ?2, ?3, ?4)",
            )?;
            for (position, e) in events.iter().enumerate() {
                stmt.execute(params![session, position as i64, e.event, e.count])?;
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO session_jokers (session, earned, spent) VALUES (?1, ?2, ?3)",
            params![session, jokers.earned, jokers.spent],
        )?;
        tx.commit()
    }

    /// Removes a session's sets, end, events and jokers from the history.
    pub fn remove_session(&mut self, session: i64) -> Result<()> {
        let tx = self.conn.transaction()?;
        for table in ["sets", "session_ends", "session_events", "session_jokers"] {
            tx.execute(
                &format!("DELETE FROM {table} WHERE session = ?1"),
                [session],
            )?;
        }
        tx.commit()
    }

    /// Totals of the last `days` days (today included), most recent first.
    pub fn history(&self, days: u32, today: NaiveDate) -> Result<Vec<DayTotals>> {
        let (entries, records) = self.sets_since(window_start(days, today))?;
        Ok(last_days(&entries, &records, days, today))
    }

    /// Every day on which sets were done, most recent first.
    pub fn full_history(&self) -> Result<Vec<DayTotals>> {
        let (entries, records) = self.sets_since(i64::MIN)?;
        Ok(active_days(&entries, &records))
    }

    /// Sets done since `from`, and what is kept of their sessions.
    fn sets_since(&self, from: i64) -> Result<(Vec<LogEntry>, SessionRecords)> {
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
        let ends = self.session_rows("SELECT session, ended FROM session_ends", from, |r| {
            r.get(1)
        })?;
        let mut events: HashMap<i64, Vec<EventCount>> = HashMap::new();
        for (session, event) in self.session_rows(
            "SELECT session, event, count FROM session_events",
            from,
            |r| {
                Ok(EventCount {
                    event: r.get(1)?,
                    count: r.get(2)?,
                })
            },
        )? {
            events.entry(session).or_default().push(event);
        }
        let jokers = self.session_rows(
            "SELECT session, earned, spent FROM session_jokers",
            from,
            |r| {
                Ok(Jokers {
                    earned: r.get(1)?,
                    spent: r.get(2)?,
                })
            },
        )?;
        let records = SessionRecords {
            ends: ends.into_iter().collect(),
            events,
            jokers: jokers.into_iter().collect(),
        };
        Ok((entries, records))
    }

    /// Rows of `select` (whose first column is the session) for sessions with sets since `from`,
    /// in their saved order.
    fn session_rows<T>(
        &self,
        select: &str,
        from: i64,
        value: impl Fn(&Row) -> Result<T>,
    ) -> Result<Vec<(i64, T)>> {
        self.conn
            .prepare(&format!(
                "{select} WHERE session IN (SELECT session FROM sets WHERE at >= ?1) ORDER BY rowid"
            ))?
            .query_map([from], |r| Ok((r.get(0)?, value(r)?)))?
            .collect()
    }

    /// Last chosen game, empty if there is none yet.
    pub fn last_game(&self) -> Result<String> {
        Ok(self.setting(LAST_GAME)?.unwrap_or_default())
    }

    pub fn set_last_game(&self, game_id: &str) -> Result<()> {
        set_setting(&self.conn, LAST_GAME, game_id)
    }

    /// Language setting, `System` if missing or unknown.
    pub fn language(&self) -> Result<LanguageSetting> {
        self.parsed_setting(LANGUAGE, LanguageSetting::System)
    }

    pub fn set_language(&self, language: LanguageSetting) -> Result<()> {
        set_setting(&self.conn, LANGUAGE, language.as_str())
    }

    /// Steam Store answers and the safeguards on its calls.
    pub fn steam_names(&self) -> Result<SteamNames> {
        let names = self
            .conn
            .prepare("SELECT app_id, name, retry_at FROM steam_names")?
            .query_map([], |r| {
                let name = match r.get(1)? {
                    Some(name) => SteamName::Known(name),
                    None => SteamName::RetryAt(r.get::<_, Option<i64>>(2)?.unwrap_or(0)),
                };
                Ok((r.get(0)?, name))
            })?
            .collect::<Result<_>>()?;
        Ok(SteamNames {
            names,
            last_call: self.parsed_setting(STEAM_LAST_CALL, 0)?,
            blocked_until: self.parsed_setting(STEAM_BLOCKED_UNTIL, 0)?,
        })
    }

    /// Short list, fully rewritten after each Steam Store call.
    pub fn save_steam_names(&mut self, steam: &SteamNames) -> Result<()> {
        let tx = self.conn.transaction()?;
        set_setting(&tx, STEAM_LAST_CALL, &steam.last_call.to_string())?;
        set_setting(&tx, STEAM_BLOCKED_UNTIL, &steam.blocked_until.to_string())?;
        tx.execute("DELETE FROM steam_names", [])?;
        {
            let mut stmt =
                tx.prepare("INSERT INTO steam_names (app_id, name, retry_at) VALUES (?1, ?2, ?3)")?;
            for (app_id, name) in &steam.names {
                match name {
                    SteamName::Known(name) => stmt.execute(params![app_id, name, None::<i64>])?,
                    SteamName::RetryAt(at) => stmt.execute(params![app_id, None::<String>, at])?,
                };
            }
        }
        tx.commit()
    }

    /// Setting parsed as `T`, `default` if missing or unreadable.
    fn parsed_setting<T: FromStr>(&self, key: &str, default: T) -> Result<T> {
        Ok(self
            .setting(key)?
            .and_then(|v| v.parse().ok())
            .unwrap_or(default))
    }

    fn setting(&self, key: &str) -> Result<Option<String>> {
        match self
            .conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
                r.get(0)
            }) {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            other => other.map(Some),
        }
    }
}

/// Applies in a transaction the migrations the database has not received yet,
/// and refuses a database modified by an unknown migration, from a newer version of the app.
fn migrate(conn: &mut Connection) -> Result<()> {
    let run = |conn: &mut Connection, target| {
        migrations::runner()
            .set_grouped(true)
            .set_target(target)
            .run(conn)
            .map(drop)
            .map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                    Some(format!("database schema not updated: {e}")),
                )
            })
    };
    // A database migrated before refinery records in `user_version` the number of migrations already
    // applied: they are added to the history without being replayed.
    let legacy: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if legacy > 0 {
        run(conn, Target::FakeVersion(legacy))?;
        conn.pragma_update(None, "user_version", 0)?;
    }
    update_translated_checksums(conn)?;
    run(conn, Target::Latest)
}

/// Checksums of the migrations as applied before their comments were translated to English, by version.
pub const PRE_TRANSLATION_CHECKSUMS: [(i32, u64); 2] =
    [(4, 13746842255044008563), (5, 15443599466271548012)];

/// Replaces in the history the pre-translation checksums with the current ones, so that refinery
/// does not refuse databases that applied those migrations before.
fn update_translated_checksums(conn: &Connection) -> Result<()> {
    let has_history: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE name = 'refinery_schema_history')",
        [],
        |r| r.get(0),
    )?;
    if !has_history {
        return Ok(());
    }
    let runner = migrations::runner();
    for (version, old) in PRE_TRANSLATION_CHECKSUMS {
        let current = runner
            .get_migrations()
            .iter()
            .find(|m| m.version() == version)
            .expect("translated migration")
            .checksum();
        conn.execute(
            "UPDATE refinery_schema_history SET checksum = ?1 WHERE version = ?2 AND checksum = ?3",
            (current.to_string(), version, old.to_string()),
        )?;
    }
    Ok(())
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        [key, value],
    )?;
    Ok(())
}

/// Syncs a table with an ordered list: a row is only rewritten if it differs from the one
/// that held the same position, and keys that are gone are deleted.
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
