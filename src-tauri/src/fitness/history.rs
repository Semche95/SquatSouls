//! Bilan : séries de l'historique regroupées par jour, par séance et par jeu.

use super::{totals, LogEntry, Total};
use chrono::{DateTime, Duration, Local, NaiveDate, NaiveTime};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DayTotals {
    /// Date au format AAAA-MM-JJ.
    pub date: String,
    pub sets: usize,
    /// Séances du jour, de la plus récente à la plus ancienne.
    pub sessions: Vec<SessionTotals>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SessionTotals {
    /// Heure de la première série faite.
    pub start: i64,
    /// Secondes entre le début de la séance et sa fin (ou sa dernière série).
    pub duration: i64,
    pub game: String,
    pub sets: usize,
    pub totals: Vec<Total>,
}

/// Regroupe les séries par séance et par jeu.
pub fn sessions(entries: &[LogEntry], ends: &HashMap<i64, i64>) -> Vec<SessionTotals> {
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
            let end = ends.get(&session).map_or(last, |&ended| ended.max(last));
            SessionTotals {
                start,
                duration: end - session.min(start),
                game: game.to_string(),
                sets: list.len(),
                totals: totals(&list),
            }
        })
        .collect();
    out.sort_by_key(|s| std::cmp::Reverse(s.start));
    out
}

type ByDay = BTreeMap<NaiveDate, Vec<LogEntry>>;

/// Séries regroupées par jour local.
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

fn day_totals(date: NaiveDate, entries: &[LogEntry], ends: &HashMap<i64, i64>) -> DayTotals {
    DayTotals {
        date: date.format("%Y-%m-%d").to_string(),
        sets: entries.len(),
        sessions: sessions(entries, ends),
    }
}

/// Premier des `days` derniers jours (aujourd'hui compris).
fn first_day(days: u32, today: NaiveDate) -> NaiveDate {
    today - Duration::days(i64::from(days.max(1) - 1))
}

/// Horodatage du début local du premier des `days` derniers jours.
pub fn window_start(days: u32, today: NaiveDate) -> i64 {
    first_day(days, today)
        .and_time(NaiveTime::MIN)
        .and_local_timezone(Local)
        .earliest()
        .map_or(i64::MIN, |d| d.timestamp())
}

/// Totaux des `days` derniers jours (aujourd'hui compris), du plus récent au plus ancien.
pub fn last_days(
    entries: &[LogEntry],
    ends: &HashMap<i64, i64>,
    days: u32,
    today: NaiveDate,
) -> Vec<DayTotals> {
    let mut by_day = by_day(entries);
    (0..days.max(1))
        .map(|offset| {
            let date = today - Duration::days(i64::from(offset));
            day_totals(date, &by_day.remove(&date).unwrap_or_default(), ends)
        })
        .collect()
}

/// Tous les jours où des séries ont été faites, du plus récent au plus ancien.
pub fn active_days(entries: &[LogEntry], ends: &HashMap<i64, i64>) -> Vec<DayTotals> {
    by_day(entries)
        .iter()
        .rev()
        .map(|(date, entries)| day_totals(*date, entries, ends))
        .collect()
}
