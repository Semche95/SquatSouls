//! Configuration: settings, ignored games, exercises, games and events, synced row by row.

use super::settings::set_setting;
use super::Store;
use crate::app::{
    Config, Exercise, Game, GameEvent, IgnoredGame, TimeUnit, DEFAULT_RECOVERY,
    DEFAULT_STEAM_NOTIFY, DEFAULT_TIMER_PREP,
};
use rusqlite::{params, Result, Transaction};

const RECOVERY: &str = "recovery";
const STEAM_DETECTION: &str = "steam_detection";
const STEAM_NOTIFY: &str = "steam_notify";
const TRAY: &str = "tray";
const CLOSE_TO_TRAY: &str = "close_to_tray";
const START_HIDDEN: &str = "start_hidden";
const REMEMBER_INTENSITY: &str = "remember_intensity";
const SOUND: &str = "sound";
const TIMER_AUTO_DONE: &str = "timer_auto_done";
const TIMER_PREP: &str = "timer_prep";

impl Store {
    pub fn config(&self) -> Result<Config> {
        let exercises = self
            .conn
            .prepare(
                "SELECT id, name, unit, min, max, recovery, enabled, timed, each_side FROM exercises ORDER BY position",
            )?
            .query_map([], |r| {
                Ok(Exercise {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    unit: r.get(2)?,
                    min: r.get(3)?,
                    max: r.get(4)?,
                    recovery: r.get(5)?,
                    enabled: r.get(6)?,
                    timed: r
                        .get::<_, Option<String>>(7)?
                        .and_then(|t| TimeUnit::parse(&t)),
                    each_side: r.get(8)?,
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
            recovery: self.parsed_setting(RECOVERY, DEFAULT_RECOVERY)?,
            steam_detection: self.parsed_setting(STEAM_DETECTION, false)?,
            steam_notify: self.parsed_setting(STEAM_NOTIFY, DEFAULT_STEAM_NOTIFY)?,
            tray: self.parsed_setting(TRAY, false)?,
            close_to_tray: self.parsed_setting(CLOSE_TO_TRAY, false)?,
            start_hidden: self.parsed_setting(START_HIDDEN, false)?,
            remember_intensity: self.parsed_setting(REMEMBER_INTENSITY, false)?,
            sound: self.parsed_setting(SOUND, true)?,
            timer_auto_done: self.parsed_setting(TIMER_AUTO_DONE, true)?,
            timer_prep: self.parsed_setting(TIMER_PREP, DEFAULT_TIMER_PREP)?,
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
            (
                RECOVERY,
                old.recovery.to_string(),
                config.recovery.to_string(),
            ),
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
            (
                REMEMBER_INTENSITY,
                old.remember_intensity.to_string(),
                config.remember_intensity.to_string(),
            ),
            (SOUND, old.sound.to_string(), config.sound.to_string()),
            (
                TIMER_AUTO_DONE,
                old.timer_auto_done.to_string(),
                config.timer_auto_done.to_string(),
            ),
            (
                TIMER_PREP,
                old.timer_prep.to_string(),
                config.timer_prep.to_string(),
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
            "INSERT INTO exercises (id, position, name, unit, min, max, recovery, enabled, timed, each_side) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT (id) DO UPDATE SET position = ?2, name = ?3, unit = ?4, min = ?5, max = ?6, recovery = ?7, enabled = ?8, timed = ?9, each_side = ?10",
            |stmt, pos, e| {
                let timed = e.timed.map(TimeUnit::as_str);
                stmt.execute(params![e.id, pos, e.name, e.unit, e.min, e.max, e.recovery, e.enabled, timed, e.each_side])
            },
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
