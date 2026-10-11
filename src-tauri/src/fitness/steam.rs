//! Détection du jeu Steam en cours : AppId lu sur un processus ou noté par Steam, nom lu dans la
//! réponse du Steam Store.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// AppId du jeu lancé par Steam, lu dans la ligne de commande d'un processus
/// (`SteamLaunch AppId=…`, sous Linux) ou dans son environnement (`SteamAppId=…`).
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

/// AppId du jeu en cours noté par Steam dans son `registry.vdf` (macOS, Linux) : valeur de la
/// clé `RunningAppID`, à zéro sans jeu lancé.
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

/// Adresse de la fiche d'un jeu sur l'API du Steam Store, qui ne demande pas de clé.
fn steam_app_url(app_id: u32) -> String {
    format!("https://store.steampowered.com/api/appdetails?appids={app_id}&filters=basic&l=french")
}

/// Nom du jeu dans la réponse de `steam_app_url`, sans les marques ™ et ®.
fn steam_app_name(app_id: u32, json: &str) -> Option<String> {
    let response: Value = serde_json::from_str(json).ok()?;
    let name = response
        .get(app_id.to_string())?
        .get("data")?
        .get("name")?
        .as_str()?
        .replace(['™', '®'], "");
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// Délai minimal entre deux appels au Steam Store, en secondes, pour ne jamais approcher de sa limite.
pub const STEAM_STORE_INTERVAL: i64 = 60;
/// Délai avant de redemander un jeu que le Steam Store n'a pas nommé, ou pas répondu.
pub const STEAM_RETRY_DELAY: i64 = 3600;
/// Délai sans aucun appel après un refus du Steam Store pour excès de requêtes.
pub const STEAM_BLOCK_DELAY: i64 = 24 * 3600;

/// Pourquoi le nom d'un jeu n'est pas connu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SteamNameError {
    /// Le Steam Store ne connaît pas ce jeu, ou n'a pas répondu : il sera redemandé après `STEAM_RETRY_DELAY`.
    Unknown,
    /// Un autre appel vient d'être fait : on réessaiera plus tard.
    Wait,
    /// Le Steam Store a limité les requêtes : plus aucun appel pendant `STEAM_BLOCK_DELAY`.
    Blocked,
}

/// Réponse d'un appel au Steam Store : code HTTP et contenu, ou `None` sans réponse.
pub type SteamResponse = Option<(u16, String)>;

/// Réponse gardée pour un jeu : son nom, ou l'heure à partir de laquelle le redemander.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteamName {
    Known(String),
    RetryAt(i64),
}

/// Noms des jeux demandés au Steam Store, avec les garde-fous qui évitent d'en être bloqué,
/// gardés d'un lancement à l'autre : un nom obtenu n'est plus redemandé, un échec l'est après
/// `STEAM_RETRY_DELAY`, au plus un appel par `STEAM_STORE_INTERVAL`, et plus aucun pendant
/// `STEAM_BLOCK_DELAY` après un refus pour excès de requêtes (429 ou 403). Heures en secondes Unix.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SteamNames {
    pub(super) names: HashMap<u32, SteamName>,
    pub(super) last_call: i64,
    pub(super) blocked_until: i64,
}

impl SteamNames {
    /// `remember` reçoit l'état après chaque appel, pour le garder d'un lancement à l'autre.
    pub fn name(
        &mut self,
        app_id: u32,
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
        let name = match fetch(&steam_app_url(app_id)) {
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
