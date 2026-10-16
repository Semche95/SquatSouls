//! What the session shows: sets done, totals and triggers.

use super::{Draw, Intensity, Session};
use crate::app::Config;
use serde::{Deserialize, Serialize};

/// A set done, as saved in the history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub at: i64,
    /// Start of the session.
    pub session: i64,
    pub game: String,
    pub event: String,
    pub exercise: String,
    pub unit: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Total {
    pub exercise: String,
    pub unit: String,
    pub amount: u32,
    pub sets: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TriggerCount {
    pub game_id: String,
    pub event_id: String,
    pub count: u32,
}

/// Times an event was triggered in a session, by its name.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EventCount {
    pub event: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    /// Absent until the session is started.
    pub started_at: Option<i64>,
    /// Game of the session, set on the first event.
    pub game_id: Option<String>,
    /// Intensity of the session.
    pub intensity: Intensity,
    pub pending: Vec<Draw>,
    pub sets: usize,
    pub totals: Vec<Total>,
    /// Sets done, most recent first.
    pub log: Vec<LogEntry>,
    pub triggers: Vec<TriggerCount>,
    pub jokers: u32,
    pub jokers_earned: u32,
    pub jokers_spent: u32,
}

impl Session {
    /// Events triggered in the session's game, in the order of its events.
    pub fn event_counts(&self, config: &Config) -> Vec<EventCount> {
        let Some(game) = config
            .games
            .iter()
            .find(|g| Some(&g.id) == self.game_id.as_ref())
        else {
            return Vec::new();
        };
        game.events
            .iter()
            .filter_map(|ev| {
                let count = *self.triggers.get(&(game.id.clone(), ev.id.clone()))?;
                (count > 0).then(|| EventCount {
                    event: ev.name.clone(),
                    count,
                })
            })
            .collect()
    }

    pub fn view(&self) -> SessionView {
        let mut triggers: Vec<TriggerCount> = self
            .triggers
            .iter()
            .filter(|(_, &count)| count > 0)
            .map(|((g, e), &count)| TriggerCount {
                game_id: g.clone(),
                event_id: e.clone(),
                count,
            })
            .collect();
        triggers.sort_by(|a, b| (&a.game_id, &a.event_id).cmp(&(&b.game_id, &b.event_id)));
        SessionView {
            started_at: self.started_at,
            game_id: self.game_id.clone(),
            intensity: self.intensity,
            pending: self.pending.clone(),
            sets: self.done.len(),
            totals: totals(&self.done),
            log: self.done.iter().rev().cloned().collect(),
            triggers,
            jokers: self.jokers,
            jokers_earned: self.jokers_earned,
            jokers_spent: self.jokers_spent,
        }
    }
}

/// Sums sets by exercise, in order of first appearance.
pub fn totals(entries: &[LogEntry]) -> Vec<Total> {
    let mut out: Vec<Total> = Vec::new();
    for e in entries {
        match out
            .iter_mut()
            .find(|t| t.exercise == e.exercise && t.unit == e.unit)
        {
            Some(t) => {
                t.amount += e.amount;
                t.sets += 1;
            }
            None => out.push(Total {
                exercise: e.exercise.clone(),
                unit: e.unit.clone(),
                amount: e.amount,
                sets: 1,
            }),
        }
    }
    out
}
