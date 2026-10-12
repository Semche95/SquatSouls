//! Configuration: exercises, games and events, validation and default configuration
//! (`seed.en.json`, `seed.fr.json`).

use super::{AppError, Language};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    /// Unit shown after the drawn number: "reps", "s", "reps per leg"...
    pub unit: String,
    /// Range of the drawn number, bounds included.
    pub min: u32,
    pub max: u32,
    /// Exercise-specific pause, in seconds; otherwise, the global pause.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause: Option<u32>,
    /// An unchecked exercise stays defined but is no longer drawn.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
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
    /// Pause after each exercise done, in seconds, for exercises without their own pause.
    #[serde(default = "default_pause")]
    pub pause: u32,
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
    #[serde(default)]
    pub ignored_games: Vec<IgnoredGame>,
    pub exercises: Vec<Exercise>,
    pub games: Vec<Game>,
}

impl Config {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.exercises.is_empty() {
            return Err(AppError::new("no_exercise"));
        }
        let mut exercise_ids = HashSet::new();
        for ex in &self.exercises {
            if ex.id.trim().is_empty() || ex.name.trim().is_empty() {
                return Err(AppError::new("exercise_empty"));
            }
            if !exercise_ids.insert(ex.id.as_str()) {
                return Err(AppError::new("exercise_duplicate").with("id", ex.id.as_str()));
            }
            if ex.max == 0 || ex.min > ex.max {
                return Err(AppError::new("exercise_range")
                    .with("name", ex.name.as_str())
                    .with("min", ex.min)
                    .with("max", ex.max));
            }
        }

        let active = self.active_exercises().count();
        if active == 0 {
            return Err(AppError::new("no_active_exercise"));
        }

        if self.games.is_empty() {
            return Err(AppError::new("no_game"));
        }
        let mut game_ids = HashSet::new();
        let mut game_names = HashSet::new();
        for game in &self.games {
            if game.id.trim().is_empty() || game.name.trim().is_empty() {
                return Err(AppError::new("game_empty"));
            }
            if !game_ids.insert(game.id.as_str()) {
                return Err(AppError::new("game_duplicate").with("id", game.id.as_str()));
            }
            if !game_names.insert(game.name.trim().to_lowercase()) {
                return Err(AppError::new("game_name_duplicate").with("name", game.name.trim()));
            }
            let in_game = |code| AppError::new(code).with("game", game.name.as_str());
            if game.events.is_empty() {
                return Err(in_game("game_no_event"));
            }
            let mut event_ids = HashSet::new();
            for ev in &game.events {
                if ev.id.trim().is_empty() || ev.name.trim().is_empty() {
                    return Err(in_game("event_empty"));
                }
                if !event_ids.insert(ev.id.as_str()) {
                    return Err(in_game("event_duplicate").with("id", ev.id.as_str()));
                }
                let at = |code| in_game(code).with("event", ev.name.as_str());
                if ev.draws == 0 {
                    return Err(at("event_no_draw"));
                }
                if ev.draws as usize > active {
                    return Err(at("event_too_many_draws")
                        .with("draws", ev.draws)
                        .with("active", active));
                }
                if ev.max_per_session == Some(0) {
                    return Err(at("event_zero_limit"));
                }
            }
        }
        Ok(())
    }

    /// Checked exercises, the only ones that can be drawn.
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

pub const DEFAULT_PAUSE: u32 = 60;

fn default_pause() -> u32 {
    DEFAULT_PAUSE
}

fn default_enabled() -> bool {
    true
}

pub const DEFAULT_STEAM_NOTIFY: bool = true;

fn default_steam_notify() -> bool {
    DEFAULT_STEAM_NOTIFY
}

/// Default configuration in `language`, written to a new database and used if the saved
/// configuration is invalid. Both seeds have the same ids, numbers and AppIds: only names and units differ.
pub fn default_config(language: Language) -> Config {
    let seed = match language {
        Language::En => include_str!("seed.en.json"),
        Language::Fr => include_str!("seed.fr.json"),
    };
    serde_json::from_str(seed).expect("invalid seed")
}
