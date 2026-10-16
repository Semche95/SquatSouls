//! Validation of the configuration, and the name and count limits it checks or repairs.

use super::Config;
use crate::app::AppError;
use std::collections::HashSet;

impl Config {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.exercises.is_empty() {
            return Err(AppError::new("no_exercise"));
        }
        let mut exercise_ids = HashSet::new();
        if self.exercises.len() > MAX_ITEMS {
            return Err(AppError::new("too_many_exercises").with("max", MAX_ITEMS));
        }
        for ex in &self.exercises {
            if ex.id.trim().is_empty() || ex.name.trim().is_empty() {
                return Err(AppError::new("exercise_empty"));
            }
            NAME.check(&ex.name, &ex.name)?;
            UNIT.check(&ex.unit, &ex.name)?;
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
            NAME.check(&game.name, &game.name)?;
            if !game_names.insert(game.name.trim().to_lowercase()) {
                return Err(AppError::new("game_name_duplicate").with("name", game.name.trim()));
            }
            let in_game = |code| AppError::new(code).with("game", game.name.as_str());
            if game.events.len() > MAX_ITEMS {
                return Err(in_game("too_many_events").with("max", MAX_ITEMS));
            }
            let mut event_ids = HashSet::new();
            for ev in &game.events {
                if ev.id.trim().is_empty() || ev.name.trim().is_empty() {
                    return Err(in_game("event_empty"));
                }
                NAME.check(&ev.name, &ev.name)?;
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

    /// Brings a configuration saved before the name and count limits within them, so it stays usable:
    /// hidden characters removed, names and units cut, exercises and events beyond the limit dropped,
    /// a name left empty replaced by its id. Ignored games, named by the Steam Store, are cleaned too. Returns whether something changed.
    pub fn repair(&mut self) -> bool {
        let mut changed = self.exercises.len() > MAX_ITEMS;
        self.exercises.truncate(MAX_ITEMS);
        for ex in &mut self.exercises {
            changed |= NAME.repair(&mut ex.name, &ex.id) | UNIT.repair(&mut ex.unit, "");
        }
        for ignored in &mut self.ignored_games {
            changed |= NAME.repair(&mut ignored.name, "");
        }
        for game in &mut self.games {
            changed |= NAME.repair(&mut game.name, &game.id) | (game.events.len() > MAX_ITEMS);
            game.events.truncate(MAX_ITEMS);
            for ev in &mut game.events {
                changed |= NAME.repair(&mut ev.name, &ev.id);
            }
        }
        changed
    }
}

/// Longest name (game, event, exercise) and unit, in characters, and most exercises or events in a game.
pub const MAX_NAME: usize = 300;
pub const MAX_UNIT: usize = 50;
pub const MAX_ITEMS: usize = 100;

/// Control or invisible character (zero width, text direction override): it would break the copied
/// summary or hide part of a name. The zero width joiner stays allowed, emoji sequences use it.
pub fn hidden_char(c: char) -> bool {
    c.is_control()
        || matches!(
            c,
            '\u{AD}' | '\u{200B}' | '\u{200C}' | '\u{200E}' | '\u{200F}' | '\u{FEFF}'
        )
        || matches!(c, '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{2069}')
}

/// Text field limited in length and without hidden characters, with its error codes.
struct TextField {
    max: usize,
    too_long: &'static str,
    hidden: &'static str,
}

const NAME: TextField = TextField {
    max: MAX_NAME,
    too_long: "name_too_long",
    hidden: "name_hidden_char",
};
const UNIT: TextField = TextField {
    max: MAX_UNIT,
    too_long: "unit_too_long",
    hidden: "unit_hidden_char",
};

/// Characters of `owner` shown in an error, enough to recognize it.
const SHOWN: usize = 40;

impl TextField {
    /// `owner` is the name of what carries the text, shown in the error, cut if it is long.
    fn check(&self, text: &str, owner: &str) -> Result<(), AppError> {
        let shown = || match owner.char_indices().nth(SHOWN) {
            Some((end, _)) => format!("{}…", &owner[..end]),
            None => owner.to_string(),
        };
        if text.chars().any(hidden_char) {
            return Err(AppError::new(self.hidden).with("name", shown()));
        }
        if text.chars().count() > self.max {
            return Err(AppError::new(self.too_long)
                .with("name", shown())
                .with("max", self.max));
        }
        Ok(())
    }

    /// Text without its hidden characters, cut at the limit.
    fn clean(&self, text: &str) -> String {
        text.chars()
            .filter(|&c| !hidden_char(c))
            .take(self.max)
            .collect()
    }

    /// Removes the hidden characters and cuts the text at the limit; left empty, it becomes `fallback`.
    fn repair(&self, text: &mut String, fallback: &str) -> bool {
        let mut repaired = self.clean(text);
        if repaired.trim().is_empty() && !text.trim().is_empty() {
            repaired = fallback.chars().take(self.max).collect();
        }
        let changed = repaired != *text;
        *text = repaired;
        changed
    }
}

/// Name from outside the configuration (Steam Store), brought within the name limits.
pub fn clean_name(name: &str) -> String {
    NAME.clean(name)
}
