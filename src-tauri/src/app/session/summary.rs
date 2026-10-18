//! What the session shows: sets done, totals and triggers.

use super::{Draw, Intensity, Session};
use crate::app::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub amount: u64,
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
    /// Triggers of the events, sets to do included, against which their limits apply.
    pub triggers: Vec<TriggerCount>,
    /// Triggers the summary counts: those of the sets to do not started yet are left out, as when the session
    /// stops there.
    pub counted_triggers: Vec<TriggerCount>,
    pub jokers: u32,
    /// Jokers earned and spent as the summary counts them (see `Session::jokers`).
    pub jokers_earned: u32,
    pub jokers_spent: u32,
}

impl Session {
    /// Triggers the summary counts: a set to do not started yet is left out, as if it were dismissed.
    fn counted_triggers(&self) -> HashMap<(String, String), u32> {
        let mut counted = self.triggers.clone();
        for draw in self.pending.iter().filter(|d| !d.started()) {
            if let Some(count) = counted.get_mut(&(draw.game_id.clone(), draw.event_id.clone())) {
                *count = count.saturating_sub(1);
            }
        }
        counted
    }

    /// Events triggered in the session's game as the summary counts them, in the order of its events.
    pub fn event_counts(&self, config: &Config) -> Vec<EventCount> {
        let counted = self.counted_triggers();
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
                let count = *counted.get(&(game.id.clone(), ev.id.clone()))?;
                (count > 0).then(|| EventCount {
                    event: ev.name.clone(),
                    count,
                })
            })
            .collect()
    }

    pub fn view(&self) -> SessionView {
        SessionView {
            started_at: self.started_at,
            game_id: self.game_id.clone(),
            intensity: self.intensity,
            pending: self.pending.clone(),
            sets: self.done.len(),
            totals: totals(&self.done),
            log: self.done.iter().rev().cloned().collect(),
            triggers: trigger_counts(&self.triggers),
            counted_triggers: trigger_counts(&self.counted_triggers()),
            jokers: self.jokers,
            jokers_earned: self.jokers().earned,
            jokers_spent: self.jokers().spent,
        }
    }
}

/// Positive triggers, by game then event.
fn trigger_counts(triggers: &HashMap<(String, String), u32>) -> Vec<TriggerCount> {
    let mut counts: Vec<TriggerCount> = triggers
        .iter()
        .filter(|(_, &count)| count > 0)
        .map(|((g, e), &count)| TriggerCount {
            game_id: g.clone(),
            event_id: e.clone(),
            count,
        })
        .collect();
    counts.sort_by(|a, b| (&a.game_id, &a.event_id).cmp(&(&b.game_id, &b.event_id)));
    counts
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
                t.amount += u64::from(e.amount);
                t.sets += 1;
            }
            None => out.push(Total {
                exercise: e.exercise.clone(),
                unit: e.unit.clone(),
                amount: u64::from(e.amount),
                sets: 1,
            }),
        }
    }
    out
}
