//! Steam Store names and the safeguards on its calls.

use super::settings::set_setting;
use super::Store;
use crate::app::{clean_name, SteamName, SteamNames};
use rusqlite::{params, Result};

const STEAM_LAST_CALL: &str = "steam_last_call";
const STEAM_BLOCKED_UNTIL: &str = "steam_blocked_until";

impl Store {
    /// Steam Store answers and the safeguards on its calls.
    pub fn steam_names(&self) -> Result<SteamNames> {
        let names = self
            .conn
            .prepare("SELECT app_id, name, retry_at FROM steam_names")?
            .query_map([], |r| {
                let name = match r.get::<_, Option<String>>(1)? {
                    // Name saved before the Steam Store names were cleaned.
                    Some(name) => SteamName::Known(clean_name(&name)),
                    None => SteamName::RetryAt(r.get::<_, Option<i64>>(2)?.unwrap_or(0)),
                };
                Ok((r.get(0)?, name))
            })?
            .collect::<Result<_>>()?;
        Ok(SteamNames {
            names,
            last_call: self.parsed_setting(STEAM_LAST_CALL, 0)?,
            blocked_until: self.parsed_setting(STEAM_BLOCKED_UNTIL, 0)?,
        })
    }

    /// Short list, fully rewritten after each Steam Store call.
    pub fn save_steam_names(&mut self, steam: &SteamNames) -> Result<()> {
        let tx = self.conn.transaction()?;
        set_setting(&tx, STEAM_LAST_CALL, &steam.last_call.to_string())?;
        set_setting(&tx, STEAM_BLOCKED_UNTIL, &steam.blocked_until.to_string())?;
        tx.execute("DELETE FROM steam_names", [])?;
        {
            let mut stmt =
                tx.prepare("INSERT INTO steam_names (app_id, name, retry_at) VALUES (?1, ?2, ?3)")?;
            for (app_id, name) in &steam.names {
                match name {
                    SteamName::Known(name) => stmt.execute(params![app_id, name, None::<i64>])?,
                    SteamName::RetryAt(at) => stmt.execute(params![app_id, None::<String>, at])?,
                };
            }
        }
        tx.commit()
    }
}
