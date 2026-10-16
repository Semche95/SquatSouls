//! History: sets done, session ends, events and jokers, and their reading by day.

use super::Store;
use crate::app::{
    active_days, count_days, days_range, last_days, window_start, DayTotals, EventCount, Jokers,
    LogEntry, SessionRecords,
};
use chrono::NaiveDate;
use rusqlite::{params, Result, Row};
use std::collections::HashMap;

impl Store {
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
            .prepare("SELECT DISTINCT session FROM sets")?
            .query_map([], |r| r.get(0))?
            .collect()
    }

    /// Sets of the sessions started between `from` and `to` (included), and what is kept of
    /// those sessions.
    fn sets_between(&self, from: i64, to: i64) -> Result<(Vec<LogEntry>, SessionRecords)> {
        let entries = self
            .conn
            .prepare(
                "SELECT at, session, game, event, exercise, unit, amount FROM sets
                 WHERE session BETWEEN ?1 AND ?2 ORDER BY at, id",
            )?
            .query_map([from, to], |r| {
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
        let ends = self.session_rows("SELECT session, ended FROM session_ends", from, to, |r| {
            r.get(1)
        })?;
        let mut events: HashMap<i64, Vec<EventCount>> = HashMap::new();
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
        let records = SessionRecords {
            ends: ends.into_iter().collect(),
            events,
            jokers: jokers.into_iter().collect(),
        };
        Ok((entries, records))
    }

    /// Rows of `select` (whose first column is the session) for sessions started between `from`
    /// and `to` (included), in their saved order.
    fn session_rows<T>(
        &self,
        select: &str,
        from: i64,
        to: i64,
        value: impl Fn(&Row) -> Result<T>,
    ) -> Result<Vec<(i64, T)>> {
        self.conn
            .prepare(&format!(
                "{select} WHERE session BETWEEN ?1 AND ?2 ORDER BY rowid"
            ))?
            .query_map([from, to], |r| Ok((r.get(0)?, value(r)?)))?
            .collect()
    }
}
