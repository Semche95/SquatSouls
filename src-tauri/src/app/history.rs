//! Summary: history sets grouped by day, by session and by game.

use super::{totals, EventCount, Jokers, LogEntry, Total};
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DayTotals {
    /// Date in YYYY-MM-DD format.
    pub date: String,
    pub sets: usize,
    /// Sessions of the day, most recent first.
    pub sessions: Vec<SessionTotals>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionTotals {
    /// Time of the first set done.
    pub start: i64,
    /// Seconds between the start of the session and its end (or its last set).
    pub duration: i64,
    pub game: String,
    pub sets: usize,
    pub totals: Vec<Total>,
    /// Events triggered, empty for sessions saved before they were recorded.
    pub events: Vec<EventCount>,
    /// Absent for sessions saved before they were recorded.
    pub jokers: Option<Jokers>,
}

/// What the history keeps of each session besides its sets, by session start.
#[derive(Debug, Default)]
pub struct SessionRecords {
    pub ends: HashMap<i64, i64>,
    pub events: HashMap<i64, Vec<EventCount>>,
    pub jokers: HashMap<i64, Jokers>,
}

/// Groups sets by session and by game.
pub fn sessions(entries: &[LogEntry], records: &SessionRecords) -> Vec<SessionTotals> {
    let mut groups: Vec<((i64, &str), Vec<LogEntry>)> = Vec::new();
    for e in entries {
        let key = (e.session, e.game.as_str());
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, list)) => list.push(e.clone()),
            None => groups.push((key, vec![e.clone()])),
        }
    }
    let mut out: Vec<SessionTotals> = groups
        .into_iter()
        .map(|((session, game), list)| {
            let start = list.iter().map(|e| e.at).min().unwrap_or_default();
            let last = list.iter().map(|e| e.at).max().unwrap_or_default();
            let end = records
                .ends
                .get(&session)
                .map_or(last, |&ended| ended.max(last));
            SessionTotals {
                start,
                duration: end - session.min(start),
                game: game.to_string(),
                sets: list.len(),
                totals: totals(&list),
                events: records.events.get(&session).cloned().unwrap_or_default(),
                jokers: records.jokers.get(&session).copied(),
            }
        })
        .collect();
    out.sort_by_key(|s| std::cmp::Reverse(s.start));
    out
}

type ByDay = BTreeMap<NaiveDate, Vec<LogEntry>>;

/// Sets grouped by local day.
fn by_day(entries: &[LogEntry]) -> ByDay {
    let mut days = ByDay::new();
    for e in entries {
        if let Some(d) = DateTime::from_timestamp(e.at, 0) {
            days.entry(d.with_timezone(&Local).date_naive())
                .or_default()
                .push(e.clone());
        }
    }
    days
}

fn day_totals(date: NaiveDate, entries: &[LogEntry], records: &SessionRecords) -> DayTotals {
    DayTotals {
        date: date.format("%Y-%m-%d").to_string(),
        sets: entries.len(),
        sessions: sessions(entries, records),
    }
}

/// First of the last `days` days (today included).
fn first_day(days: u32, today: NaiveDate) -> NaiveDate {
    today - Duration::days(i64::from(days.max(1) - 1))
}

/// Timestamp of the local start of the first of the last `days` days.
pub fn window_start(days: u32, today: NaiveDate) -> i64 {
    first_day(days, today)
        .and_time(NaiveTime::MIN)
        .and_local_timezone(Local)
        .earliest()
        .map_or(i64::MIN, |d| d.timestamp())
}

/// Totals of the last `days` days (today included), most recent first.
pub fn last_days(
    entries: &[LogEntry],
    records: &SessionRecords,
    days: u32,
    today: NaiveDate,
) -> Vec<DayTotals> {
    let mut by_day = by_day(entries);
    (0..days.max(1))
        .map(|offset| {
            let date = today - Duration::days(i64::from(offset));
            day_totals(date, &by_day.remove(&date).unwrap_or_default(), records)
        })
        .collect()
}

/// Every day on which sets were done, most recent first.
pub fn active_days(entries: &[LogEntry], records: &SessionRecords) -> Vec<DayTotals> {
    by_day(entries)
        .iter()
        .rev()
        .map(|(date, entries)| day_totals(*date, entries, records))
        .collect()
}
