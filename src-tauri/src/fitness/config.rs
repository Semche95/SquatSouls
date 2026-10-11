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
pub struct Game {
    pub id: String,
    pub name: String,
    pub events: Vec<GameEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
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
                if ev.draws as usize > self.exercises.len() {
                    return Err(format!(
                        "{at} : {} exercices à tirer mais seulement {} définis.",
                        ev.draws,
                        self.exercises.len()
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

/// Configuration par défaut, écrite dans une base neuve et utilisée si la configuration enregistrée est invalide.
pub fn default_config() -> Config {
    serde_json::from_str(include_str!("seed.json")).expect("seed.json invalide")
}
