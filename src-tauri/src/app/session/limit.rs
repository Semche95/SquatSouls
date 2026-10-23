//! Limit of the session, set by whoever trains: in exercises asked, in play time or in estimated training time.

use super::{DrawItem, Session};
use crate::app::{AppError, Config, Exercise, TimeUnit, MAX_AMOUNT};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LimitKind {
    /// Exercises asked, counted from the trigger of their set.
    Exercises,
    /// Time the session lasted, in minutes.
    PlayTime,
    /// Estimated time of the exercises asked, recovery included, in minutes.
    TrainingTime,
}

impl LimitKind {
    pub fn as_str(self) -> &'static str {
        match self {
            LimitKind::Exercises => "exercises",
            LimitKind::PlayTime => "playTime",
            LimitKind::TrainingTime => "trainingTime",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionLimit {
    pub kind: LimitKind,
    /// Exercises, or minutes.
    pub value: u32,
}

impl SessionLimit {
    /// Refuses a value out of the bounds of the other limits.
    pub fn checked(self) -> Result<Self, AppError> {
        if (1..=MAX_AMOUNT).contains(&self.value) {
            Ok(self)
        } else {
            Err(AppError::new("invalid_limit").with("value", self.value.to_string().as_str()))
        }
    }
}

/// Saved as `kind:value`, so that it can be remembered as a setting.
impl fmt::Display for SessionLimit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.kind.as_str(), self.value)
    }
}

impl FromStr for SessionLimit {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        let (kind, value) = value.split_once(':').ok_or(())?;
        let kind = match kind {
            "exercises" => LimitKind::Exercises,
            "playTime" => LimitKind::PlayTime,
            "trainingTime" => LimitKind::TrainingTime,
            _ => return Err(()),
        };
        let value = value.parse().map_err(|_| ())?;
        SessionLimit { kind, value }.checked().map_err(drop)
    }
}

/// Estimated time of an exercise to do, in seconds: its duration on each side with the countdown before the timer
/// when it is timed, otherwise its repetitions at the configured duration, then its recovery.
pub fn estimate(config: &Config, item: &DrawItem) -> u32 {
    let ex = config.exercises.iter().find(|ex| ex.id == item.exercise_id);
    estimate_of(config, ex, item.amount)
}

/// Estimated time of `amount` of the exercise `ex`, as `estimate` gives it; an unknown exercise counts as
/// repetitions followed by the global recovery.
pub(super) fn estimate_of(config: &Config, ex: Option<&Exercise>, amount: u32) -> u32 {
    let work = match ex.and_then(|ex| ex.timed.map(|unit| (unit, ex.each_side))) {
        Some((unit, each_side)) => {
            let unit = match unit {
                TimeUnit::Seconds => 1,
                TimeUnit::Minutes => 60,
            };
            let side = config
                .timer_prep
                .saturating_add(amount.saturating_mul(unit));
            side.saturating_mul(if each_side { 2 } else { 1 })
        }
        None => amount.saturating_mul(config.rep_duration),
    };
    work.saturating_add(ex.and_then(|ex| ex.recovery).unwrap_or(config.recovery))
}

impl Session {
    /// Sets the limit of the next draws, none to remove it. A higher limit lets the events give sets again; a lower
    /// one removes nothing already asked.
    pub fn set_limit(&mut self, limit: Option<SessionLimit>) -> Result<(), AppError> {
        let limit = limit.map(SessionLimit::checked).transpose()?;
        if limit != self.limit {
            self.limit = limit;
            self.limit_full = false;
        }
        Ok(())
    }

    /// Seconds the session lasted at `now`, without the time the app was closed.
    pub fn played(&self, now: i64) -> i64 {
        match self.started_at {
            Some(_) => self.counted + (now - self.opened_at).max(0),
            None => 0,
        }
    }

    /// True once the limit no longer lets the events give a set.
    pub fn limit_reached(&self, now: i64) -> bool {
        let Some(limit) = self.limit else {
            return false;
        };
        match limit.kind {
            LimitKind::Exercises => self.asked >= limit.value,
            LimitKind::PlayTime => self.played(now) >= i64::from(limit.value) * 60,
            LimitKind::TrainingTime => {
                self.limit_full || self.asked_time >= limit.value.saturating_mul(60)
            }
        }
    }

    /// Exercises the limit still lets a set ask: all of them, except for a limit in exercises.
    pub(super) fn limit_room(&self) -> usize {
        match self.limit {
            Some(SessionLimit {
                kind: LimitKind::Exercises,
                value,
            }) => value.saturating_sub(self.asked) as usize,
            _ => usize::MAX,
        }
    }

    /// Keeps the first exercises of a set that fit within a limit in training time, the others are returned; the
    /// limit is reached as soon as one does not fit.
    pub(super) fn fit(
        &mut self,
        config: &Config,
        mut items: Vec<DrawItem>,
    ) -> (Vec<DrawItem>, Vec<DrawItem>) {
        let Some(SessionLimit {
            kind: LimitKind::TrainingTime,
            value,
        }) = self.limit
        else {
            return (items, Vec::new());
        };
        let mut left = value.saturating_mul(60).saturating_sub(self.asked_time);
        let fitting = items
            .iter()
            .take_while(|it| {
                let time = estimate(config, it);
                let fits = time <= left;
                left = left.saturating_sub(time);
                fits
            })
            .count();
        let rest = items.split_off(fitting);
        self.limit_full = !rest.is_empty();
        (items, rest)
    }

    /// Counts the exercises of a set asked, against the limit.
    pub(super) fn ask(&mut self, count: u32, time: u32) {
        self.asked = self.asked.saturating_add(count);
        self.asked_time = self.asked_time.saturating_add(time);
    }

    /// Uncounts the exercises of a dismissed set: the room they took is free again.
    pub(super) fn unask(&mut self, count: u32, time: u32) {
        self.asked = self.asked.saturating_sub(count);
        self.asked_time = self.asked_time.saturating_sub(time);
        self.limit_full = false;
    }
}
