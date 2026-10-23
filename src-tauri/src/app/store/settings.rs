//! Settings outside the configuration (last game, intensity, session limit, onboarding, language) and setting
//! access.

use super::Store;
use crate::app::{Intensity, LanguageSetting, SessionLimit};
use rusqlite::{Connection, Result};
use std::str::FromStr;

const LAST_GAME: &str = "last_game";
const LANGUAGE: &str = "language";
const INTENSITY: &str = "intensity";
const LIMIT: &str = "session_limit";
const ONBOARDING_SEEN: &str = "onboarding_seen";

impl Store {
    /// Last chosen game, empty if there is none yet.
    pub fn last_game(&self) -> Result<String> {
        Ok(self.setting(LAST_GAME)?.unwrap_or_default())
    }

    pub fn set_last_game(&self, game_id: &str) -> Result<()> {
        set_setting(&self.conn, LAST_GAME, game_id)
    }

    /// Last chosen intensity, normal if missing or unknown.
    pub fn intensity(&self) -> Result<Intensity> {
        self.parsed_setting(INTENSITY, Intensity::Normal)
    }

    pub fn set_intensity(&self, intensity: Intensity) -> Result<()> {
        set_setting(&self.conn, INTENSITY, intensity.as_str())
    }

    /// Last chosen session limit, none if missing or unknown.
    pub fn limit(&self) -> Result<Option<SessionLimit>> {
        Ok(self.setting(LIMIT)?.and_then(|v| v.parse().ok()))
    }

    /// Saves the chosen session limit, an empty value without one.
    pub fn set_limit(&self, limit: Option<SessionLimit>) -> Result<()> {
        let value = limit.map(|l| l.to_string()).unwrap_or_default();
        set_setting(&self.conn, LIMIT, &value)
    }

    /// Whether the onboarding was closed once, false until then.
    pub fn onboarding_seen(&self) -> Result<bool> {
        self.parsed_setting(ONBOARDING_SEEN, false)
    }

    pub fn set_onboarding_seen(&self) -> Result<()> {
        set_setting(&self.conn, ONBOARDING_SEEN, "true")
    }

    /// Language setting, `System` if missing or unknown.
    pub fn language(&self) -> Result<LanguageSetting> {
        self.parsed_setting(LANGUAGE, LanguageSetting::System)
    }

    pub fn set_language(&self, language: LanguageSetting) -> Result<()> {
        set_setting(&self.conn, LANGUAGE, language.as_str())
    }

    /// Setting parsed as `T`, `default` if missing or unreadable.
    pub(super) fn parsed_setting<T: FromStr>(&self, key: &str, default: T) -> Result<T> {
        Ok(self
            .setting(key)?
            .and_then(|v| v.parse().ok())
            .unwrap_or(default))
    }

    fn setting(&self, key: &str) -> Result<Option<String>> {
        match self
            .conn
            .query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
                r.get(0)
            }) {
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            other => other.map(Some),
        }
    }
}

pub(super) fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        [key, value],
    )?;
    Ok(())
}
