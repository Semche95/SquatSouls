//! Historique : une ligne JSON par série faite, bilan par jour, par séance et par jeu.

use super::{totals, LogEntry, Total};
use chrono::{DateTime, Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs::OpenOptions;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

/// Fin d'une séance terminée, enregistrée dans l'historique à côté de ses séries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionEnd {
    pub session: i64,
    pub ended: i64,
}

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
    /// Secondes entre le début de la séance (ou la première série) et sa fin (ou la dernière série).
    pub duration: i64,
    pub game: String,
    pub sets: usize,
    pub totals: Vec<Total>,
}

/// Regroupe les séries par séance et par jeu (les anciennes entrées sans séance sont regroupées par jeu).
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
            let (begin, end) = if session > 0 {
                let end = ends.get(&session).map_or(last, |&ended| ended.max(last));
                (session.min(start), end)
            } else {
                (start, last)
            };
            SessionTotals {
                start,
                duration: end - begin,
                game: game.to_string(),
                sets: list.len(),
                totals: totals(&list),
            }
        })
        .collect();
    out.sort_by_key(|s| std::cmp::Reverse(s.start));
    out
}

pub fn append_history(path: &Path, entries: &[LogEntry]) -> io::Result<()> {
    if entries.is_empty() {
        return Ok(());
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    for e in entries {
        writeln!(file, "{}", serde_json::to_string(e)?)?;
    }
    Ok(())
}

pub fn append_session_end(path: &Path, session: i64, ended: i64) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(&SessionEnd { session, ended })?
    )
}

/// Retire de l'historique les séries d'une séance (les lignes illisibles sont gardées telles quelles).
pub fn remove_session(path: &Path, session: i64) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    let kept: String = std::fs::read_to_string(path)?
        .lines()
        .filter(|line| {
            let line_session = serde_json::from_str::<LogEntry>(line)
                .map(|e| e.session)
                .or_else(|_| serde_json::from_str::<SessionEnd>(line).map(|e| e.session));
            line_session.map_or(true, |s| s != session)
        })
        .map(|line| format!("{line}\n"))
        .collect();
    std::fs::write(path, kept)
}

type ByDay = BTreeMap<NaiveDate, Vec<LogEntry>>;

/// Séries de l'historique regroupées par jour local, et heures de fin des séances terminées.
fn read_by_day(path: &Path) -> io::Result<(ByDay, HashMap<i64, i64>)> {
    let mut by_day = ByDay::new();
    let mut ends = HashMap::new();
    if path.exists() {
        let file = std::fs::File::open(path)?;
        for line in BufReader::new(file).lines() {
            let line = line?;
            if let Ok(end) = serde_json::from_str::<SessionEnd>(&line) {
                ends.insert(end.session, end.ended);
                continue;
            }
            // Une ligne abîmée ne doit pas empêcher de lire le reste.
            let Ok(entry) = serde_json::from_str::<LogEntry>(&line) else {
                continue;
            };
            let Some(date) =
                DateTime::from_timestamp(entry.at, 0).map(|d| d.with_timezone(&Local).date_naive())
            else {
                continue;
            };
            by_day.entry(date).or_default().push(entry);
        }
    }
    Ok((by_day, ends))
}

fn day_totals(date: NaiveDate, entries: &[LogEntry], ends: &HashMap<i64, i64>) -> DayTotals {
    DayTotals {
        date: date.format("%Y-%m-%d").to_string(),
        sets: entries.len(),
        sessions: sessions(entries, ends),
    }
}

/// Totaux des `days` derniers jours (aujourd'hui compris), du plus récent au plus ancien.
pub fn read_history(path: &Path, days: u32, today: NaiveDate) -> io::Result<Vec<DayTotals>> {
    let (mut by_day, ends) = read_by_day(path)?;
    Ok((0..days.max(1))
        .map(|offset| {
            let date = today - Duration::days(i64::from(offset));
            day_totals(date, &by_day.remove(&date).unwrap_or_default(), &ends)
        })
        .collect())
}

/// Tous les jours où des séries ont été faites, du plus récent au plus ancien.
pub fn read_full_history(path: &Path) -> io::Result<Vec<DayTotals>> {
    let (by_day, ends) = read_by_day(path)?;
    Ok(by_day
        .iter()
        .rev()
        .map(|(date, entries)| day_totals(*date, entries, &ends))
        .collect())
}
