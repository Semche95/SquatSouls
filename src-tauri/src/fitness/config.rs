//! Configuration : exercices, jeux et événements, validation et configuration par défaut.

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
    #[serde(default = "one")]
    pub draws: usize,
    /// Au-delà de ce nombre de déclenchements dans la séance, plus aucune série.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_per_session: Option<u32>,
    /// Récompense : jokers gagnés en validant la série de chaque déclenchement.
    #[serde(default)]
    pub jokers: u32,
}

fn one() -> usize {
    1
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
        for game in &self.games {
            if game.id.trim().is_empty() || game.name.trim().is_empty() {
                return Err("Un jeu a un id ou un nom vide.".into());
            }
            if !game_ids.insert(game.id.as_str()) {
                return Err(format!("Le jeu « {} » est défini deux fois.", game.id));
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
                if ev.draws > self.exercises.len() {
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

pub fn event(id: &str, name: &str, draws: usize, max: Option<u32>) -> GameEvent {
    GameEvent {
        id: id.into(),
        name: name.into(),
        draws,
        max_per_session: max,
        jokers: 0,
    }
}

fn reward(id: &str, name: &str, jokers: u32) -> GameEvent {
    GameEvent {
        jokers,
        ..event(id, name, 2, None)
    }
}

/// Événements communs aux Souls, avec le vocabulaire propre à chaque jeu.
fn souls_events(rest: &str, discovered: &str, level: &str, lost: &str) -> Vec<GameEvent> {
    vec![
        event("mort", "Mort", 1, Some(15)),
        event("repos", rest, 1, None),
        event("decouverte", discovered, 1, None),
        event("niveau", level, 1, None),
        event("perte", lost, 1, None),
        event("boss", "Boss vaincu", 2, None),
        reward("boss-premier", "Boss vaincu au premier essai", 1),
        reward("boss-nohit", "Boss vaincu sans prendre un coup", 2),
    ]
}

pub fn default_config() -> Config {
    let ex = |id: &str, name: &str, unit: &str, min, max| Exercise {
        id: id.into(),
        name: name.into(),
        unit: unit.into(),
        min,
        max,
    };
    let game = |id: &str, name: &str, events| Game {
        id: id.into(),
        name: name.into(),
        events,
    };
    let ds = || {
        souls_events(
            "Repos à un feu connu",
            "Nouveau feu allumé",
            "Montée de niveau",
            "Âmes perdues",
        )
    };
    Config {
        exercises: vec![
            ex("pompes", "Pompes", "rép.", 8, 12),
            ex("tractions", "Tractions", "rép.", 2, 4),
            ex("squats", "Squats", "rép.", 12, 18),
            ex("fentes", "Fentes", "rép. par jambe", 5, 8),
            ex("gainage", "Gainage (planche)", "s", 20, 45),
            ex("superman", "Superman", "rép.", 10, 15),
        ],
        games: vec![
            game(
                "elden-ring",
                "Elden Ring",
                souls_events(
                    "Repos à un site de grâce connu",
                    "Site de grâce découvert",
                    "Montée de niveau",
                    "Runes perdues",
                ),
            ),
            game("ds1", "Dark Souls", ds()),
            game("ds2", "Dark Souls II", ds()),
            game("ds3", "Dark Souls III", ds()),
            game(
                "bloodborne",
                "Bloodborne",
                souls_events(
                    "Repos à une lanterne connue",
                    "Lanterne allumée",
                    "Montée de niveau",
                    "Échos de sang perdus",
                ),
            ),
            game(
                "sekiro",
                "Sekiro",
                souls_events(
                    "Repos à une statue du Sculpteur",
                    "Statue du Sculpteur découverte",
                    "Point de compétence gagné",
                    "Sen et expérience perdus",
                ),
            ),
        ],
    }
}
