//! Tirage et séance.

use super::Config;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DrawItem {
    pub exercise_id: String,
    pub name: String,
    pub unit: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Draw {
    pub id: u64,
    pub game_id: String,
    pub game_name: String,
    pub event_id: String,
    pub event_name: String,
    pub items: Vec<DrawItem>,
    /// Jokers gagnés à la validation (événement de récompense).
    pub jokers: u32,
    /// Vrai quand la limite de l'événement pour la séance est dépassée.
    pub capped: bool,
    pub trigger_count: u32,
    pub max_per_session: Option<u32>,
}

/// Une série faite, telle qu'enregistrée dans l'historique.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub at: i64,
    /// Début de la séance (0 pour les entrées d'avant ce champ).
    #[serde(default)]
    pub session: i64,
    pub game: String,
    pub event: String,
    pub exercise: String,
    pub unit: String,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Total {
    pub exercise: String,
    pub unit: String,
    pub amount: u32,
    pub sets: u32,
}

/// Une série sautée grâce à un joker.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Skipped {
    pub at: i64,
    pub game: String,
    pub event: String,
    pub items: Vec<DrawItem>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TriggerCount {
    pub game_id: String,
    pub event_id: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub started_at: i64,
    /// Jeu de la séance, fixé au premier événement.
    pub game_id: Option<String>,
    pub pending: Vec<Draw>,
    pub sets: usize,
    pub totals: Vec<Total>,
    /// Séries faites, la plus récente en premier.
    pub log: Vec<LogEntry>,
    pub triggers: Vec<TriggerCount>,
    pub jokers: u32,
    pub jokers_earned: u32,
    /// Séries sautées, la plus récente en premier.
    pub skipped: Vec<Skipped>,
}

#[derive(Debug, Default)]
pub struct Session {
    pub started_at: i64,
    game_id: Option<String>,
    triggers: HashMap<(String, String), u32>,
    pending: Vec<Draw>,
    done: Vec<LogEntry>,
    jokers: u32,
    jokers_earned: u32,
    skipped: Vec<Skipped>,
    next_id: u64,
}

impl Session {
    pub fn new(now: i64) -> Self {
        Session {
            started_at: now,
            ..Default::default()
        }
    }

    pub fn draw<R: Rng + ?Sized>(
        &mut self,
        config: &Config,
        game_id: &str,
        event_id: &str,
        rng: &mut R,
    ) -> Result<Draw, String> {
        let game = config
            .games
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| format!("Jeu inconnu : {game_id}"))?;
        if self.game_id.as_ref().is_some_and(|id| id != &game.id) {
            return Err("La séance a déjà commencé sur un autre jeu : lance une nouvelle séance pour changer.".into());
        }
        let ev = game
            .events
            .iter()
            .find(|e| e.id == event_id)
            .ok_or_else(|| format!("Événement inconnu : {event_id}"))?;

        self.game_id = Some(game.id.clone());
        let count = self
            .triggers
            .entry((game.id.clone(), ev.id.clone()))
            .or_insert(0);
        // Une fois la limite atteinte, le compteur ne bouge plus.
        let capped = ev.max_per_session.is_some_and(|max| *count >= max);
        if !capped {
            *count += 1;
        }
        let trigger_count = *count;

        let items = if capped {
            Vec::new()
        } else {
            let picks = rand::seq::index::sample(rng, config.exercises.len(), ev.draws);
            picks
                .iter()
                .map(|i| {
                    let ex = &config.exercises[i];
                    DrawItem {
                        exercise_id: ex.id.clone(),
                        name: ex.name.clone(),
                        unit: ex.unit.clone(),
                        amount: rng.random_range(ex.min..=ex.max),
                    }
                })
                .collect()
        };

        self.next_id += 1;
        let draw = Draw {
            id: self.next_id,
            game_id: game.id.clone(),
            game_name: game.name.clone(),
            event_id: ev.id.clone(),
            event_name: ev.name.clone(),
            items,
            jokers: if capped { 0 } else { ev.jokers },
            capped,
            trigger_count,
            max_per_session: ev.max_per_session,
        };
        if !capped {
            self.pending.push(draw.clone());
        }
        Ok(draw)
    }

    /// Valide une série tirée : elle passe dans les séries faites.
    pub fn complete(&mut self, id: u64, now: i64) -> Result<Vec<LogEntry>, String> {
        let draw = self.take_pending(id)?;
        self.jokers += draw.jokers;
        self.jokers_earned += draw.jokers;
        let entries: Vec<LogEntry> = draw
            .items
            .into_iter()
            .map(|it| LogEntry {
                at: now,
                session: self.started_at,
                game: draw.game_name.clone(),
                event: draw.event_name.clone(),
                exercise: it.name,
                unit: it.unit,
                amount: it.amount,
            })
            .collect();
        self.done.extend(entries.iter().cloned());
        Ok(entries)
    }

    /// Annule un déclenchement fait par erreur : rien n'est compté.
    pub fn dismiss(&mut self, id: u64) -> Result<(), String> {
        let draw = self.take_pending(id)?;
        if let Some(count) = self.triggers.get_mut(&(draw.game_id, draw.event_id)) {
            *count = count.saturating_sub(1);
        }
        Ok(())
    }

    /// Dépense un joker pour sauter une série : le déclenchement reste compté.
    pub fn use_joker(&mut self, id: u64, now: i64) -> Result<(), String> {
        if self.jokers == 0 {
            return Err("Aucun joker disponible.".into());
        }
        let draw = self.take_pending(id)?;
        self.jokers -= 1;
        self.skipped.push(Skipped {
            at: now,
            game: draw.game_name,
            event: draw.event_name,
            items: draw.items,
        });
        Ok(())
    }

    /// Retire les tirages en attente (utile quand la configuration change).
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    fn take_pending(&mut self, id: u64) -> Result<Draw, String> {
        let pos = self
            .pending
            .iter()
            .position(|d| d.id == id)
            .ok_or("Cette série n'est plus en attente.")?;
        Ok(self.pending.remove(pos))
    }

    pub fn view(&self) -> SessionView {
        let mut triggers: Vec<TriggerCount> = self
            .triggers
            .iter()
            .filter(|(_, &count)| count > 0)
            .map(|((g, e), &count)| TriggerCount {
                game_id: g.clone(),
                event_id: e.clone(),
                count,
            })
            .collect();
        triggers.sort_by(|a, b| (&a.game_id, &a.event_id).cmp(&(&b.game_id, &b.event_id)));
        SessionView {
            started_at: self.started_at,
            game_id: self.game_id.clone(),
            pending: self.pending.clone(),
            sets: self.done.len(),
            totals: totals(&self.done),
            log: self.done.iter().rev().cloned().collect(),
            triggers,
            jokers: self.jokers,
            jokers_earned: self.jokers_earned,
            skipped: self.skipped.iter().rev().cloned().collect(),
        }
    }
}

/// Cumule les séries par exercice, dans l'ordre de première apparition.
pub fn totals(entries: &[LogEntry]) -> Vec<Total> {
    let mut out: Vec<Total> = Vec::new();
    for e in entries {
        match out
            .iter_mut()
            .find(|t| t.exercise == e.exercise && t.unit == e.unit)
        {
            Some(t) => {
                t.amount += e.amount;
                t.sets += 1;
            }
            None => out.push(Total {
                exercise: e.exercise.clone(),
                unit: e.unit.clone(),
                amount: e.amount,
                sets: 1,
            }),
        }
    }
    out
}
