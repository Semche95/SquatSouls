//! Configuration: exercises, games and events, validation and default configuration (`seed.json`).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    /// Unit shown after the drawn number: "rép.", "s", "rép. par jambe"...
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
    /// Pause after each completed set, in seconds, for exercises without their own pause.
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
    pub fn validate(&self) -> Result<(), String> {
        if self.exercises.is_empty() {
            return Err("No exercise defined.".into());
        }
        let mut exercise_ids = HashSet::new();
        for ex in &self.exercises {
            if ex.id.trim().is_empty() || ex.name.trim().is_empty() {
                return Err("An exercise has an empty id or name.".into());
            }
            if !exercise_ids.insert(ex.id.as_str()) {
                return Err(format!("Exercise \"{}\" is defined twice.", ex.id));
            }
            if ex.max == 0 || ex.min > ex.max {
                return Err(format!(
                    "Invalid range for \"{}\" ({} to {}).",
                    ex.name, ex.min, ex.max
                ));
            }
        }

        let active = self.active_exercises().count();
        if active == 0 {
            return Err("No exercise checked for the draw.".into());
        }

        if self.games.is_empty() {
            return Err("No game defined.".into());
        }
        let mut game_ids = HashSet::new();
        let mut game_names = HashSet::new();
        for game in &self.games {
            if game.id.trim().is_empty() || game.name.trim().is_empty() {
                return Err("A game has an empty id or name.".into());
            }
            if !game_ids.insert(game.id.as_str()) {
                return Err(format!("Game \"{}\" is defined twice.", game.id));
            }
            if !game_names.insert(game.name.trim().to_lowercase()) {
                return Err(format!("Two games are named \"{}\".", game.name.trim()));
            }
            if game.events.is_empty() {
                return Err(format!("Game \"{}\" has no event.", game.name));
            }
            let mut event_ids = HashSet::new();
            for ev in &game.events {
                let at = format!("{} › {}", game.name, ev.name);
                if ev.id.trim().is_empty() || ev.name.trim().is_empty() {
                    return Err(format!("{}: empty event id or name.", game.name));
                }
                if !event_ids.insert(ev.id.as_str()) {
                    return Err(format!(
                        "{}: event \"{}\" is defined twice.",
                        game.name, ev.id
                    ));
                }
                if ev.draws == 0 {
                    return Err(format!("{at}: at least 1 exercise must be drawn."));
                }
                if ev.draws as usize > active {
                    return Err(format!(
                        "{at}: {} exercises to draw but only {active} checked.",
                        ev.draws
                    ));
                }
                if ev.max_per_session == Some(0) {
                    return Err(format!(
                        "{at}: the per-session limit must be at least 1 (or empty)."
                    ));
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

/// Default configuration, written to a new database and used if the saved configuration is invalid.
pub fn default_config() -> Config {
    serde_json::from_str(include_str!("seed.json")).expect("invalid seed.json")
}
