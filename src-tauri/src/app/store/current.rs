//! Session in progress: saved at each change and read back at launch, so that it resumes after quitting as after
//! a crash. Its sets done are those of the history.

use super::Store;
use crate::app::{Draw, DrawItem, Intensity, Session};
use rusqlite::types::Type;
use rusqlite::{params, Connection, OptionalExtension, Result, Row, ToSql};
use std::collections::HashMap;

/// Columns of an exercise of a set, after those telling where it belongs and its position.
const ITEM_COLUMNS: &str =
    "exercise, name, unit, amount, base, halved, done, halve_cost, skip_cost";

impl Store {
    /// Replaces the saved session in progress with `session`, the app being open at `seen`, and its duration in
    /// the history. A session not started yet has nothing to save.
    pub fn save_current_session(&mut self, session: &Session, seen: i64) -> Result<()> {
        let (Some(started_at), Some(game)) = (session.started_at, &session.game_id) else {
            return Ok(());
        };
        let tx = self.conn.savepoint()?;
        clear(&tx)?;
        tx.execute(
            "INSERT INTO current_session
             (id, started_at, game, intensity, jokers, jokers_earned, jokers_spent, next_id, seen)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                started_at,
                game,
                session.intensity.as_str(),
                session.jokers,
                session.jokers_earned,
                session.jokers_spent,
                session.next_id as i64,
                seen
            ],
        )?;
        {
            let mut stmt = tx
                .prepare("INSERT INTO current_triggers (game, event, count) VALUES (?1, ?2, ?3)")?;
            for ((game, event), count) in &session.triggers {
                stmt.execute(params![game, event, count])?;
            }
            let mut stmt = tx.prepare(
                "INSERT INTO current_sets (id, position, game, game_name, event, event_name, jokers,
                 trigger_count, max_per_session, spent) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )?;
            for (position, d) in session.pending.iter().enumerate() {
                // SQLite integers are signed; ids stay far from their limit.
                let id = d.id as i64;
                stmt.execute(params![
                    id,
                    position as i64,
                    d.game_id,
                    d.game_name,
                    d.event_id,
                    d.event_name,
                    d.jokers,
                    d.trigger_count,
                    d.max_per_session,
                    d.spent
                ])?;
                for (drawn, items) in [(false, &d.items), (true, &d.drawn)] {
                    insert_items(
                        &tx,
                        "current_set_items",
                        [("set_id", &id), ("drawn", &drawn)],
                        items,
                    )?;
                }
            }
            for ((game, event), items) in &session.reserved {
                insert_items(
                    &tx,
                    "current_reserved",
                    [("game", game), ("event", event)],
                    items,
                )?;
            }
            let mut stmt = tx.prepare(
                "INSERT INTO current_round (last, position, exercise) VALUES (?1, ?2, ?3)",
            )?;
            for (last, exercises) in [(false, &session.bag), (true, &session.last)] {
                for (position, exercise) in exercises.iter().enumerate() {
                    stmt.execute(params![last, position as i64, exercise])?;
                }
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO session_durations (session, counted, opened) VALUES (?1, ?2, ?3)",
            params![started_at, session.counted, session.opened_at],
        )?;
        tx.commit()
    }

    /// Records that the app is open at `now`, while a session is in progress.
    pub fn mark_seen(&self, now: i64) -> Result<()> {
        self.conn
            .execute("UPDATE current_session SET seen = ?1", [now])?;
        Ok(())
    }

    /// Forgets the session in progress, once it ended or was cancelled.
    pub fn clear_current_session(&self) -> Result<()> {
        clear(&self.conn)
    }

    /// Ends the session in progress that could not be read back, at the last moment the app was known to be open,
    /// and forgets it. Its start and that moment are read on their own, so that the rest being unreadable does not
    /// keep it from ending; if they are unreadable too, it is only forgotten.
    pub fn end_unreadable_session(&mut self) -> Result<()> {
        self.atomically(|store| {
            let times = store
                .conn
                .query_row("SELECT started_at, seen FROM current_session", [], |r| {
                    Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
                })
                .optional();
            match times {
                Ok(Some((started_at, seen))) => store.end_session(started_at, seen)?,
                Ok(None) => {}
                Err(e) => log::error!("Unreadable session not ended in the history: {e}"),
            }
            store.clear_current_session()
        })
    }

    /// Session in progress, with the last moment the app was known to be open; none without one.
    pub fn current_session(&self) -> Result<Option<(Session, i64)>> {
        let Some((mut session, seen)) = self
            .conn
            .query_row(
                "SELECT started_at, game, intensity, jokers, jokers_earned, jokers_spent, next_id, seen
                 FROM current_session",
                [],
                |r| {
                    let session = Session {
                        started_at: Some(r.get(0)?),
                        game_id: Some(r.get(1)?),
                        intensity: r.get::<_, String>(2)?.parse::<Intensity>().map_err(|_| {
                            rusqlite::Error::FromSqlConversionFailure(
                                2,
                                Type::Text,
                                "unknown intensity".into(),
                            )
                        })?,
                        jokers: r.get(3)?,
                        jokers_earned: r.get(4)?,
                        jokers_spent: r.get(5)?,
                        next_id: r.get::<_, i64>(6)? as u64,
                        ..Session::default()
                    };
                    Ok((session, r.get(7)?))
                },
            )
            .optional()?
        else {
            return Ok(None);
        };
        let started_at = session.started_at.unwrap_or_default();
        session.triggers = self
            .conn
            .prepare("SELECT game, event, count FROM current_triggers")?
            .query_map([], |r| Ok(((r.get(0)?, r.get(1)?), r.get(2)?)))?
            .collect::<Result<_>>()?;
        session.pending = self
            .conn
            .prepare(
                "SELECT id, game, game_name, event, event_name, jokers, trigger_count, max_per_session, spent
                 FROM current_sets ORDER BY position",
            )?
            .query_map([], |r| {
                Ok(Draw {
                    id: r.get::<_, i64>(0)? as u64,
                    game_id: r.get(1)?,
                    game_name: r.get(2)?,
                    event_id: r.get(3)?,
                    event_name: r.get(4)?,
                    items: Vec::new(),
                    jokers: r.get(5)?,
                    // A set over the event's limit is never kept to do.
                    capped: false,
                    trigger_count: r.get(6)?,
                    max_per_session: r.get(7)?,
                    drawn: Vec::new(),
                    spent: r.get(8)?,
                })
            })?
            .collect::<Result<_>>()?;
        let mut items = self.conn.prepare(&format!(
            "SELECT {ITEM_COLUMNS} FROM current_set_items WHERE set_id = ?1 AND drawn = ?2 ORDER BY position"
        ))?;
        for d in &mut session.pending {
            let id = d.id as i64;
            d.items = items
                .query_map(params![id, false], |r| item(r, 0))?
                .collect::<Result<_>>()?;
            d.drawn = items
                .query_map(params![id, true], |r| item(r, 0))?
                .collect::<Result<_>>()?;
        }
        let reserved: Vec<((String, String), DrawItem)> = self
            .conn
            .prepare(&format!(
                "SELECT game, event, {ITEM_COLUMNS} FROM current_reserved ORDER BY game, event, position"
            ))?
            .query_map([], |r| Ok(((r.get(0)?, r.get(1)?), item(r, 2)?)))?
            .collect::<Result<_>>()?;
        let mut by_event: HashMap<(String, String), Vec<DrawItem>> = HashMap::new();
        for (key, it) in reserved {
            by_event.entry(key).or_default().push(it);
        }
        session.reserved = by_event;
        for row in self
            .conn
            .prepare("SELECT last, exercise FROM current_round ORDER BY last, position")?
            .query_map([], |r| Ok((r.get::<_, bool>(0)?, r.get::<_, String>(1)?)))?
        {
            let (last, exercise) = row?;
            if last {
                &mut session.last
            } else {
                &mut session.bag
            }
            .push(exercise);
        }
        session.done = self.sets_of(started_at, started_at)?;
        session.recorded = session.done.len();
        (session.counted, session.opened_at) = self
            .conn
            .query_row(
                "SELECT counted, opened FROM session_durations WHERE session = ?1",
                [started_at],
                |r| Ok((r.get(0)?, r.get::<_, Option<i64>>(1)?.unwrap_or(seen))),
            )
            .optional()?
            .unwrap_or((0, started_at));
        Ok(Some((session, seen)))
    }
}

fn clear(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "DELETE FROM current_session;
         DELETE FROM current_triggers;
         DELETE FROM current_set_items;
         DELETE FROM current_sets;
         DELETE FROM current_reserved;
         DELETE FROM current_round;",
    )
}

/// Writes `items` in their order into `table`, under the two columns of `owner` telling where they belong.
fn insert_items(
    conn: &Connection,
    table: &str,
    owner: [(&str, &dyn ToSql); 2],
    items: &[DrawItem],
) -> Result<()> {
    let [(first, first_value), (second, second_value)] = owner;
    let mut stmt = conn.prepare(&format!(
        "INSERT INTO {table} ({first}, {second}, position, {ITEM_COLUMNS})
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"
    ))?;
    for (position, it) in items.iter().enumerate() {
        stmt.execute(params![
            first_value,
            second_value,
            position as i64,
            it.exercise_id,
            it.name,
            it.unit,
            it.amount,
            it.base,
            it.halved,
            it.done,
            it.halve_cost,
            it.skip_cost
        ])?;
    }
    Ok(())
}

/// Exercise of a set whose `ITEM_COLUMNS` start at column `from`.
fn item(r: &Row, from: usize) -> Result<DrawItem> {
    Ok(DrawItem {
        exercise_id: r.get(from)?,
        name: r.get(from + 1)?,
        unit: r.get(from + 2)?,
        amount: r.get(from + 3)?,
        base: r.get(from + 4)?,
        halved: r.get(from + 5)?,
        done: r.get(from + 6)?,
        halve_cost: r.get(from + 7)?,
        skip_cost: r.get(from + 8)?,
    })
}
