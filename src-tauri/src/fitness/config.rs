//! Configuration : exercices, jeux et événements, validation et configuration par défaut (`seed.json`).

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    /// Unité affichée après le nombre tiré : "rép.", "s", "rép. par jambe"...
    pub unit: String,
    /// Intervalle du nombre tiré, bornes incluses.
    pub min: u32,
    pub max: u32,
    /// Pause propre à l'exercice, en secondes ; à défaut, la pause globale.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pause: Option<u32>,
    /// Un exercice décoché reste défini mais n'est plus tiré.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    pub id: String,
    pub name: String,
    /// Nombre d'exercices différents tirés à chaque déclenchement.
    pub draws: u32,
    /// Au-delà de ce nombre de déclenchements dans la séance, plus aucune série.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_per_session: Option<u32>,
    /// Récompense : jokers gagnés en validant la série de chaque déclenchement.
    #[serde(default)]
    pub jokers: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub name: String,
    /// AppId Steam qui fait reconnaître le jeu quand il tourne.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steam_app_id: Option<u32>,
    pub events: Vec<GameEvent>,
}

/// Jeu Steam détecté que l'utilisateur ne veut plus se voir proposer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IgnoredGame {
    pub app_id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Pause après chaque série validée, en secondes, pour les exercices sans pause propre.
    #[serde(default = "default_pause")]
    pub pause: u32,
    /// Détection du jeu Steam en cours, désactivée par défaut.
    #[serde(default)]
    pub steam_detection: bool,
    /// Notification système à chaque jeu Steam lancé, quand la détection est active.
    #[serde(default = "default_steam_notify")]
    pub steam_notify: bool,
    /// Icône dans la zone de notification, désactivée par défaut.
    #[serde(default)]
    pub tray: bool,
    /// Avec l'icône, fermer la fenêtre la masque au lieu de quitter.
    #[serde(default)]
    pub close_to_tray: bool,
    /// Avec l'icône, l'appli démarre fenêtre masquée.
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
            return Err("Aucun exercice défini.".into());
        }
        let mut exercise_ids = HashSet::new();
        for ex in &self.exercises {
            if ex.id.trim().is_empty() || ex.name.trim().is_empty() {
                return Err("Un exercice a un id ou un nom vide.".into());
            }
            if !exercise_ids.insert(ex.id.as_str()) {
                return Err(format!("L'exercice « {} » est défini deux fois.", ex.id));
            }
            if ex.max == 0 || ex.min > ex.max {
                return Err(format!(
                    "Intervalle invalide pour « {} » ({} à {}).",
                    ex.name, ex.min, ex.max
                ));
            }
        }

        let active = self.active_exercises().count();
        if active == 0 {
            return Err("Aucun exercice coché pour le tirage.".into());
        }

        if self.games.is_empty() {
            return Err("Aucun jeu défini.".into());
        }
        let mut game_ids = HashSet::new();
        let mut game_names = HashSet::new();
        for game in &self.games {
            if game.id.trim().is_empty() || game.name.trim().is_empty() {
                return Err("Un jeu a un id ou un nom vide.".into());
            }
            if !game_ids.insert(game.id.as_str()) {
                return Err(format!("Le jeu « {} » est défini deux fois.", game.id));
            }
            if !game_names.insert(game.name.trim().to_lowercase()) {
                return Err(format!("Deux jeux s'appellent « {} ».", game.name.trim()));
            }
            if game.events.is_empty() {
                return Err(format!("Le jeu « {} » n'a aucun événement.", game.name));
            }
            let mut event_ids = HashSet::new();
            for ev in &game.events {
                let at = format!("{} › {}", game.name, ev.name);
                if ev.id.trim().is_empty() || ev.name.trim().is_empty() {
                    return Err(format!("{} : id ou nom d'événement vide.", game.name));
                }
                if !event_ids.insert(ev.id.as_str()) {
                    return Err(format!(
                        "{} : l'événement « {} » est défini deux fois.",
                        game.name, ev.id
                    ));
                }
                if ev.draws == 0 {
                    return Err(format!("{at} : il faut tirer au moins 1 exercice."));
                }
                if ev.draws as usize > active {
                    return Err(format!(
                        "{at} : {} exercices à tirer mais seulement {active} cochés.",
                        ev.draws
                    ));
                }
                if ev.max_per_session == Some(0) {
                    return Err(format!(
                        "{at} : la limite par séance doit valoir au moins 1 (ou être vide)."
                    ));
                }
            }
        }
        Ok(())
    }

    /// Exercices cochés, seuls à pouvoir être tirés.
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

/// Configuration par défaut, écrite dans une base neuve et utilisée si la configuration enregistrée est invalide.
pub fn default_config() -> Config {
    serde_json::from_str(include_str!("seed.json")).expect("seed.json invalide")
}
