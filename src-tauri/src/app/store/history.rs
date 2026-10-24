//! History: sessions and their sets done, ends, events, jokers and durations, and their reading by day.

use super::Store;
use crate::app::{
    active_days, count_days, days_range, last_days, window_start, DayTotals, EventCount, Jokers,
    LogEntry, SessionDuration, SessionRecords,
};
use chrono::NaiveDate;
use rusqlite::{params, Params, Result, Row};
use std::collections::HashMap;
use uuid::Uuid;

/// Columns of a set done, in the order of `LogEntry`.
const SET_COLUMNS: &str = "at, session, game, event, exercise, unit, amount";

/// Sessions started between `?1` and `?2` (included).
const STARTED_BETWEEN: &str = "FROM sessions WHERE started_at BETWEEN ?1 AND ?2";

impl Store {
    /// Records the session `id` started at `started_at`, before anything of it is saved; already recorded, it stays.
    pub fn record_session(&self, id: &str, started_at: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO sessions (id, started_at) VALUES (?1, ?2)",
            params![id, started_at],
        )?;
        Ok(())
    }

    /// Appends sets done of recorded sessions, each with its own UUIDv7.
    pub fn append_history(&mut self, entries: &[LogEntry]) -> Result<()> {
        let tx = self.conn.savepoint()?;
        {
            let mut stmt = tx.prepare(&format!(
                "INSERT INTO sets (id, {SET_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
            ))?;
            for e in entries {
                stmt.execute(params![
                    Uuid::now_v7().to_string(),
                    e.at,
                    e.session,
                    e.game,
                    e.event,
                    e.exercise,
                    e.unit,
                    e.amount
                ])?;
            }
        }
        tx.commit()
    }

    /// Saves the end of a session, which closes the period it was open in.
    pub fn end_session(&mut self, session: &str, ended: i64) -> Result<()> {
        let tx = self.conn.savepoint()?;
        tx.execute(
            "INSERT OR REPLACE INTO session_ends (session, ended) VALUES (?1, ?2)",
            params![session, ended],
        )?;
        tx.execute(
            "UPDATE session_durations SET counted = counted + MAX(?2 - opened, 0), opened = NULL
             WHERE session = ?1 AND opened IS NOT NULL",
            params![session, ended],
        )?;
        tx.commit()
    }

    /// Replaces the events triggered and the jokers saved for a session.
    pub fn save_session_summary(
        &mut self,
        session: &str,
        events: &[EventCount],
        jokers: Jokers,
    ) -> Result<()> {
        let tx = self.conn.savepoint()?;
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

    /// Removes a session from the history, with its sets, end, events, jokers and duration.
    pub fn remove_session(&mut self, session: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM sessions WHERE id = ?1", [session])?;
        Ok(())
    }

    /// Totals of the last `days` days (today included), most recent first.
    pub fn history(&self, days: u32, today: NaiveDate) -> Result<Vec<DayTotals>> {
        let (entries, records) = self.sets_between(window_start(days, today), i64::MAX)?;
        Ok(last_days(&entries, &records, days, today))
    }

    /// Up to `days` days on which a session with sets done started, before `before` (from the
    /// most recent if `None`), most recent first.
    pub fn full_history(&self, before: Option<NaiveDate>, days: usize) -> Result<Vec<DayTotals>> {
        let Some((from, to)) = days_range(&self.session_starts()?, before, days) else {
            return Ok(Vec::new());
        };
        let (entries, records) = self.sets_between(from, to)?;
        Ok(active_days(&entries, &records))
    }

    /// Number of days on which a session with sets done started.
    pub fn active_day_count(&self) -> Result<usize> {
        Ok(count_days(&self.session_starts()?))
    }

    /// Starts of the sessions with sets done.
    fn session_starts(&self) -> Result<Vec<i64>> {
        self.conn
            .prepare("SELECT started_at FROM sessions WHERE id IN (SELECT session FROM sets)")?
            .query_map([], |r| r.get(0))?
            .collect()
    }

    /// Sets of the sessions started between `from` and `to` (included), and what is kept of
    /// those sessions.
    fn sets_between(&self, from: i64, to: i64) -> Result<(Vec<LogEntry>, SessionRecords)> {
        let entries = self.sets_where(
            &format!("session IN (SELECT id {STARTED_BETWEEN})"),
            [from, to],
        )?;
        let starts = self
            .conn
            .prepare(&format!("SELECT id, started_at {STARTED_BETWEEN}"))?
            .query_map([from, to], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<Result<_>>()?;
        let ends = self.session_rows("SELECT session, ended FROM session_ends", from, to, |r| {
            r.get(1)
        })?;
        let mut events: HashMap<String, Vec<EventCount>> = HashMap::new();
        for (session, event) in self.session_rows(
            "SELECT session, event, count FROM session_events",
            from,
            to,
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
            to,
            |r| {
                Ok(Jokers {
                    earned: r.get(1)?,
                    spent: r.get(2)?,
                })
            },
        )?;
        let durations = self.session_rows(
            "SELECT session, counted, opened FROM session_durations",
            from,
            to,
            |r| {
                Ok(SessionDuration {
                    counted: r.get(1)?,
                    opened: r.get(2)?,
                })
            },
        )?;
        let records = SessionRecords {
            starts,
            ends: ends.into_iter().collect(),
            events,
            jokers: jokers.into_iter().collect(),
            durations: durations.into_iter().collect(),
        };
        Ok((entries, records))
    }

    /// Sets of the session `id`, in the order they were done.
    pub(super) fn session_sets(&self, id: &str) -> Result<Vec<LogEntry>> {
        self.sets_where("session = ?1", [id])
    }

    /// Sets matching `condition`, in the order they were done: their UUIDv7 keep the order of a same second.
    fn sets_where(&self, condition: &str, params: impl Params) -> Result<Vec<LogEntry>> {
        self.conn
            .prepare(&format!(
                "SELECT {SET_COLUMNS} FROM sets WHERE {condition} ORDER BY at, id"
            ))?
            .query_map(params, |r| {
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
            .collect()
    }

    /// Rows of `select` (whose first column is the session) for sessions started between `from`
    /// and `to` (included), in their saved order.
    fn session_rows<T>(
        &self,
        select: &str,
        from: i64,
        to: i64,
        value: impl Fn(&Row) -> Result<T>,
    ) -> Result<Vec<(String, T)>> {
        self.conn
            .prepare(&format!(
                "{select} WHERE session IN (SELECT id {STARTED_BETWEEN}) ORDER BY rowid"
            ))?
            .query_map([from, to], |r| Ok((r.get(0)?, value(r)?)))?
            .collect()
    }
}
