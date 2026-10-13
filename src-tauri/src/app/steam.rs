//! Detection of the running Steam game: AppId read from a process or recorded by Steam, name read from
//! the Steam Store response.

use super::clean_name;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// AppId of the game launched by Steam, read from a process command line
/// (`SteamLaunch AppId=…`, on Linux) or from its environment (`SteamAppId=…`).
pub fn steam_app_id<S: AsRef<str>>(cmd: &[S], environ: &[S]) -> Option<u32> {
    let launched = cmd
        .iter()
        .skip_while(|arg| arg.as_ref() != "SteamLaunch")
        .find_map(|arg| arg.as_ref().strip_prefix("AppId="));
    let inherited = || {
        environ
            .iter()
            .find_map(|var| var.as_ref().strip_prefix("SteamAppId="))
    };
    launched
        .or_else(inherited)?
        .parse()
        .ok()
        .filter(|&id| id != 0)
}

/// AppId of the running game recorded by Steam in its `registry.vdf` (macOS, Linux): value of the
/// `RunningAppID` key, zero when no game is running.
pub fn running_app_id_in_vdf(vdf: &str) -> Option<u32> {
    vdf.lines().find_map(|line| {
        let mut fields = line.split('"').map(str::trim).filter(|f| !f.is_empty());
        fields
            .next()?
            .eq_ignore_ascii_case("RunningAppID")
            .then(|| fields.next()?.parse().ok())
            .flatten()
            .filter(|&id| id != 0)
    })
}

/// URL of a game's details on the Steam Store API, which requires no key, with its name in
/// `language` (Steam API language name: "french", "english"...).
fn steam_app_url(app_id: u32, language: &str) -> String {
    format!(
        "https://store.steampowered.com/api/appdetails?appids={app_id}&filters=basic&l={language}"
    )
}

/// Game name in the `steam_app_url` response, without the ™ and ® marks. Written by the game's
/// publisher, it is never trusted: brought within the configuration name limits.
fn steam_app_name(app_id: u32, json: &str) -> Option<String> {
    let response: Value = serde_json::from_str(json).ok()?;
    let name = clean_name(
        &response
            .get(app_id.to_string())?
            .get("data")?
            .get("name")?
            .as_str()?
            .replace(['™', '®'], ""),
    );
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// Minimum delay between two Steam Store calls, in seconds, to never get close to its limit.
pub const STEAM_STORE_INTERVAL: i64 = 60;
/// Delay before asking again for a game the Steam Store did not name, or did not answer for.
pub const STEAM_RETRY_DELAY: i64 = 3600;
/// Delay without any call after the Steam Store refused for too many requests.
pub const STEAM_BLOCK_DELAY: i64 = 24 * 3600;

/// Why a game's name is not known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SteamNameError {
    /// The Steam Store does not know this game, or did not answer: it will be asked again after `STEAM_RETRY_DELAY`.
    Unknown,
    /// Another call was just made: try again later.
    Wait,
    /// The Steam Store rate-limited requests: no more calls for `STEAM_BLOCK_DELAY`.
    Blocked,
}

/// Response of a Steam Store call: HTTP status and body, or `None` without a response.
pub type SteamResponse = Option<(u16, String)>;

/// Answer kept for a game: its name, or the time from which to ask again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteamName {
    Known(String),
    RetryAt(i64),
}

/// Game names asked from the Steam Store, with the safeguards that avoid getting blocked by it,
/// kept from one launch to the next: a name obtained is not asked again, a failure is after
/// `STEAM_RETRY_DELAY`, at most one call per `STEAM_STORE_INTERVAL`, and none for
/// `STEAM_BLOCK_DELAY` after a refusal for too many requests (429 or 403). Times in Unix seconds.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SteamNames {
    pub(super) names: HashMap<u32, SteamName>,
    pub(super) last_call: i64,
    pub(super) blocked_until: i64,
}

impl SteamNames {
    /// `remember` receives the state after each call, to keep it from one launch to the next.
    pub fn name(
        &mut self,
        app_id: u32,
        language: &str,
        now: i64,
        fetch: impl FnOnce(&str) -> SteamResponse,
        remember: impl FnOnce(&SteamNames),
    ) -> Result<String, SteamNameError> {
        let retry_at = match self.names.get(&app_id) {
            Some(SteamName::Known(name)) => return Ok(name.clone()),
            Some(&SteamName::RetryAt(at)) => at,
            None => now,
        };
        if now < self.blocked_until {
            return Err(SteamNameError::Blocked);
        }
        if now < retry_at {
            return Err(SteamNameError::Unknown);
        }
        if now - self.last_call < STEAM_STORE_INTERVAL {
            return Err(SteamNameError::Wait);
        }
        self.last_call = now;
        let name = match fetch(&steam_app_url(app_id, language)) {
            Some((429 | 403, _)) => {
                self.blocked_until = now + STEAM_BLOCK_DELAY;
                remember(self);
                return Err(SteamNameError::Blocked);
            }
            Some((200, json)) => steam_app_name(app_id, &json),
            _ => None,
        };
        let entry = name.clone().map_or(
            SteamName::RetryAt(now + STEAM_RETRY_DELAY),
            SteamName::Known,
        );
        self.names.insert(app_id, entry);
        remember(self);
        name.ok_or(SteamNameError::Unknown)
    }
}
