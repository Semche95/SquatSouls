//! Configuration: exercises, games and events.
//! - `defaults`: default values of its fields.
//! - `seed`: default configuration, read in one language from the seed `seed.json`.
//! - `validation`: limits and validation.

mod defaults;
mod seed;
mod validation;

pub use defaults::*;
pub use seed::*;
pub use validation::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    pub id: String,
    pub name: String,
    /// Unit shown after the drawn number: "reps", "s", "reps per leg"...
    pub unit: String,
    /// Range of the drawn number, bounds included.
    pub min: u32,
    pub max: u32,
    /// Exercise-specific recovery, in seconds; otherwise, the global recovery.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery: Option<u32>,
    /// A disabled exercise stays defined but is no longer drawn.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Time unit of the drawn number for an exercise run with the timer; otherwise, not timed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timed: Option<TimeUnit>,
    /// A timed exercise done on each side: its timer runs its duration once per side.
    #[serde(default)]
    pub each_side: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TimeUnit {
    Seconds,
    Minutes,
}

impl TimeUnit {
    pub fn as_str(self) -> &'static str {
        match self {
            TimeUnit::Seconds => "seconds",
            TimeUnit::Minutes => "minutes",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "seconds" => Some(TimeUnit::Seconds),
            "minutes" => Some(TimeUnit::Minutes),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    pub id: String,
    pub name: String,
    /// Number of distinct exercises drawn on each trigger.
    pub draws: u32,
    /// Beyond this number of triggers in the session, no more sets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_per_session: Option<u32>,
    /// Reward: jokers earned by completing the set of each trigger.
    #[serde(default)]
    pub jokers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub name: String,
    /// Steam AppId used to recognize the game while it runs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steam_app_id: Option<u32>,
    pub events: Vec<GameEvent>,
}

/// Detected Steam game the user no longer wants to be offered.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IgnoredGame {
    pub app_id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Recovery after each exercise done, in seconds, for exercises without their own recovery.
    #[serde(default = "default_recovery")]
    pub recovery: u32,
    /// Estimated duration of a repetition of the exercises not timed, in seconds, for a limit in training time.
    #[serde(default = "default_rep_duration")]
    pub rep_duration: u32,
    /// Detection of the running Steam game, disabled by default.
    #[serde(default)]
    pub steam_detection: bool,
    /// System notification for each launched Steam game, when detection is enabled.
    #[serde(default = "default_steam_notify")]
    pub steam_notify: bool,
    /// Icon in the notification area, disabled by default.
    #[serde(default)]
    pub tray: bool,
    /// With the icon, closing the window hides it instead of quitting.
    #[serde(default)]
    pub close_to_tray: bool,
    /// With the icon, the app starts with its window hidden.
    #[serde(default)]
    pub start_hidden: bool,
    /// The intensity chosen in Play is kept from one launch to the next, otherwise it starts at normal.
    #[serde(default)]
    pub remember_intensity: bool,
    /// The session limit chosen in Play is kept from one launch to the next, otherwise each launch starts without.
    #[serde(default)]
    pub remember_limit: bool,
    /// Sounds of the timer and of the end of recovery.
    #[serde(default = "default_true")]
    pub sound: bool,
    /// The timed exercise is marked done when its timer ends.
    #[serde(default = "default_true")]
    pub timer_auto_done: bool,
    /// Countdown before the timer starts, in seconds.
    #[serde(default = "default_timer_prep")]
    pub timer_prep: u32,
    /// Looks for a new version at launch.
    #[serde(default = "default_true")]
    pub update_check: bool,
    /// Downloads the new version found in the background without asking.
    #[serde(default = "default_true")]
    pub update_auto: bool,
    #[serde(default)]
    pub ignored_games: Vec<IgnoredGame>,
    pub exercises: Vec<Exercise>,
    pub games: Vec<Game>,
}

impl Config {
    /// Enabled exercises, the only ones that can be drawn.
    pub fn active_exercises(&self) -> impl Iterator<Item = &Exercise> {
        self.exercises.iter().filter(|ex| ex.enabled)
    }
}

pub fn event(id: &str, name: &str, draws: u32, max: Option<u32>) -> GameEvent {
    GameEvent {
        id: id.into(),
        name: name.into(),
        draws,
        max_per_session: max,
        jokers: 0,
    }
}
