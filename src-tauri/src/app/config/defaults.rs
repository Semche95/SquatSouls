//! Default values of the configuration and default configuration, from the seeds.

use crate::app::{Config, Language};

pub const DEFAULT_RECOVERY: u32 = 60;

pub(super) fn default_recovery() -> u32 {
    DEFAULT_RECOVERY
}

pub(super) fn default_true() -> bool {
    true
}

pub const DEFAULT_REP_DURATION: u32 = 3;

pub(super) fn default_rep_duration() -> u32 {
    DEFAULT_REP_DURATION
}

pub const DEFAULT_TIMER_PREP: u32 = 3;

pub(super) fn default_timer_prep() -> u32 {
    DEFAULT_TIMER_PREP
}

pub const DEFAULT_STEAM_NOTIFY: bool = true;

pub(super) fn default_steam_notify() -> bool {
    DEFAULT_STEAM_NOTIFY
}

/// Default configuration in `language`, written to a new database and used if the saved
/// configuration is invalid. Both seeds have the same ids, numbers and AppIds: only names and units differ.
pub fn default_config(language: Language) -> Config {
    let seed = match language {
        Language::En => include_str!("../seed.en.json"),
        Language::Fr => include_str!("../seed.fr.json"),
    };
    serde_json::from_str(seed).expect("invalid seed")
}
