//! Détection du jeu Steam en cours : AppId lu sur un processus, nom lu dans la réponse du Steam Store.

use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

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

/// Délai minimal entre deux appels au Steam Store, pour ne jamais approcher de sa limite.
pub const STEAM_STORE_INTERVAL: Duration = Duration::from_secs(60);

/// Pourquoi le nom d'un jeu n'est pas connu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SteamNameError {
    /// Le Steam Store ne connaît pas ce jeu, ou n'a pas répondu à la seule demande faite pour lui.
    Unknown,
    /// Un autre appel vient d'être fait : on réessaiera plus tard.
    Wait,
    /// Le Steam Store a limité les requêtes : plus aucun appel jusqu'au prochain lancement.
    Blocked,
}

/// Réponse d'un appel au Steam Store : code HTTP et contenu, ou `None` sans réponse.
pub type SteamResponse = Option<(u16, String)>;

/// Noms des jeux demandés au Steam Store, avec les garde-fous qui évitent d'en être bloqué :
/// un seul appel par jeu, réussi ou non, au plus un appel par `STEAM_STORE_INTERVAL`,
/// et plus aucun après un refus pour excès de requêtes (429 ou 403).
#[derive(Default)]
pub struct SteamNames {
    names: HashMap<u32, Option<String>>,
    last_call: Option<Instant>,
    blocked: bool,
}

impl SteamNames {
    pub fn name(
        &mut self,
        app_id: u32,
        now: Instant,
        fetch: impl FnOnce(&str) -> SteamResponse,
    ) -> Result<String, SteamNameError> {
        if let Some(known) = self.names.get(&app_id) {
            return known.clone().ok_or(SteamNameError::Unknown);
        }
        if self.blocked {
            return Err(SteamNameError::Blocked);
        }
        if self
            .last_call
            .is_some_and(|last| now.duration_since(last) < STEAM_STORE_INTERVAL)
        {
            return Err(SteamNameError::Wait);
        }
        self.last_call = Some(now);
        let name = match fetch(&steam_app_url(app_id)) {
            Some((429 | 403, _)) => {
                self.blocked = true;
                return Err(SteamNameError::Blocked);
            }
            Some((200, json)) => steam_app_name(app_id, &json),
            _ => None,
        };
        self.names.insert(app_id, name.clone());
        name.ok_or(SteamNameError::Unknown)
    }
}
