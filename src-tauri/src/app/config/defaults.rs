//! Default values of the configuration fields.

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
