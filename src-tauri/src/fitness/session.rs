//! Tirage et séance.

use super::Config;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Jokers pour réduire un exercice de moitié.
pub const HALVE_COST: u32 = 1;
/// Jokers pour sauter un exercice entier.
pub const SKIP_COST: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DrawItem {
    pub exercise_id: String,
    pub name: String,
    pub unit: String,
    pub amount: u32,
    /// Vrai une fois réduit de moitié avec un joker.
    pub halved: bool,
    /// Vrai une fois noté fait : il reste affiché jusqu'à la fin de la série.
    pub done: bool,
    /// Jokers pour le réduire de moitié, absent si ce n'est plus possible.
    pub halve_cost: Option<u32>,
    /// Jokers pour sauter ce qu'il en reste.
    pub skip_cost: u32,
}

impl DrawItem {
    fn new(exercise_id: String, name: String, unit: String, amount: u32) -> Self {
        DrawItem {
            exercise_id,
            name,
            unit,
            amount,
            halved: false,
            done: false,
            halve_cost: (amount >= 2).then_some(HALVE_COST),
            skip_cost: SKIP_COST,
        }
    }
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
    /// Exercices déjà faits de la série, retirés si elle est annulée.
    #[serde(skip)]
    pub logged: Vec<LogEntry>,
}

impl Draw {
    /// Dépense des jokers sur la série : elle ne rapporte plus les siens.
    fn forfeit(&mut self, items: Vec<DrawItem>, jokers: u32, halved: bool, now: i64) -> Skipped {
        self.jokers = 0;
        Skipped {
            at: now,
            game: self.game_name.clone(),
            event: self.event_name.clone(),
            items,
            jokers,
            halved,
        }
    }
}

/// Une série faite, telle qu'enregistrée dans l'historique.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub at: i64,
    /// Début de la séance.
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

/// Ce qu'un joker a retiré d'une série : un ou plusieurs exercices sautés, ou la moitié d'un exercice.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Skipped {
    pub at: i64,
    pub game: String,
    pub event: String,
    /// Exercices avec la quantité retirée.
    pub items: Vec<DrawItem>,
    pub jokers: u32,
    pub halved: bool,
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
            let picks = rand::seq::index::sample(rng, config.exercises.len(), ev.draws as usize);
            picks
                .iter()
                .map(|i| {
                    let ex = &config.exercises[i];
                    DrawItem::new(
                        ex.id.clone(),
                        ex.name.clone(),
                        ex.unit.clone(),
                        rng.random_range(ex.min..=ex.max),
                    )
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
            logged: Vec::new(),
        };
        if !capped {
            self.pending.push(draw.clone());
        }
        Ok(draw)
    }

    /// Note un exercice fait : il passe dans les séries faites. La série finie rapporte ses jokers.
    pub fn complete_exercise(
        &mut self,
        id: u64,
        exercise_id: &str,
        now: i64,
    ) -> Result<LogEntry, String> {
        let session = self.started_at;
        let (draw, pos) = self.pending_item(id, exercise_id)?;
        let item = &mut draw.items[pos];
        item.done = true;
        let entry = LogEntry {
            at: now,
            session,
            game: draw.game_name.clone(),
            event: draw.event_name.clone(),
            exercise: item.name.clone(),
            unit: item.unit.clone(),
            amount: item.amount,
        };
        draw.logged.push(entry.clone());
        self.done.push(entry.clone());
        self.finish_if_done(id)?;
        Ok(entry)
    }

    /// Retire la série quand il ne lui reste rien à faire, avec les jokers qu'elle rapporte encore.
    fn finish_if_done(&mut self, id: u64) -> Result<(), String> {
        if self.pending_draw(id)?.items.iter().all(|it| it.done) {
            let draw = self.take_pending(id)?;
            self.jokers += draw.jokers;
            self.jokers_earned += draw.jokers;
        }
        Ok(())
    }

    /// Annule un déclenchement : il n'est plus compté, ni les exercices déjà faits de la série, renvoyés pour les retirer de l'historique.
    pub fn dismiss(&mut self, id: u64) -> Result<Vec<LogEntry>, String> {
        let draw = self.take_pending(id)?;
        if let Some(count) = self.triggers.get_mut(&(draw.game_id, draw.event_id)) {
            *count = count.saturating_sub(1);
        }
        for entry in &draw.logged {
            if let Some(pos) = self.done.iter().position(|e| e == entry) {
                self.done.remove(pos);
            }
        }
        Ok(draw.logged)
    }

    /// Réduit un exercice de moitié (arrondi au supérieur) : l'autre moitié compte comme sautée.
    pub fn halve(&mut self, id: u64, exercise_id: &str, now: i64) -> Result<(), String> {
        let jokers = self.jokers;
        let (draw, pos) = self.pending_item(id, exercise_id)?;
        let item = &mut draw.items[pos];
        let cost = item
            .halve_cost
            .ok_or("Cet exercice ne peut plus être réduit.")?;
        if jokers < cost {
            return Err(NOT_ENOUGH_JOKERS.into());
        }
        let removed = item.amount / 2;
        item.amount -= removed;
        item.halved = true;
        item.halve_cost = None;
        item.skip_cost -= cost;
        let removed = DrawItem {
            amount: removed,
            ..item.clone()
        };
        let skipped = draw.forfeit(vec![removed], cost, true, now);
        self.spend(skipped);
        Ok(())
    }

    /// Saute un exercice : le reste de la série demeure à faire, sauf s'il n'y a plus rien.
    pub fn skip_exercise(&mut self, id: u64, exercise_id: &str, now: i64) -> Result<(), String> {
        let jokers = self.jokers;
        let (draw, pos) = self.pending_item(id, exercise_id)?;
        if jokers < draw.items[pos].skip_cost {
            return Err(NOT_ENOUGH_JOKERS.into());
        }
        let item = draw.items.remove(pos);
        let cost = item.skip_cost;
        let skipped = draw.forfeit(vec![item], cost, false, now);
        self.spend(skipped);
        self.finish_if_done(id)?;
        Ok(())
    }

    /// Saute ce qui reste d'une série : le déclenchement reste compté.
    pub fn skip_set(&mut self, id: u64, now: i64) -> Result<(), String> {
        let draw = self.pending_draw(id)?;
        let cost = skip_set_cost(draw);
        if self.jokers < cost {
            return Err(NOT_ENOUGH_JOKERS.into());
        }
        let mut draw = self.take_pending(id)?;
        let items = std::mem::take(&mut draw.items)
            .into_iter()
            .filter(|it| !it.done)
            .collect();
        let skipped = draw.forfeit(items, cost, false, now);
        self.spend(skipped);
        Ok(())
    }

    fn spend(&mut self, skipped: Skipped) {
        self.jokers -= skipped.jokers;
        self.skipped.push(skipped);
    }

    /// Retire les tirages en attente (utile quand la configuration change).
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Vrai si la séance a commencé sur un jeu absent de `config`.
    pub fn game_removed(&self, config: &Config) -> bool {
        self.game_id
            .as_ref()
            .is_some_and(|id| config.games.iter().all(|g| &g.id != id))
    }

    fn pending_pos(&self, id: u64) -> Result<usize, String> {
        self.pending
            .iter()
            .position(|d| d.id == id)
            .ok_or_else(|| "Cette série n'est plus en attente.".into())
    }

    fn pending_draw(&self, id: u64) -> Result<&Draw, String> {
        Ok(&self.pending[self.pending_pos(id)?])
    }

    fn take_pending(&mut self, id: u64) -> Result<Draw, String> {
        let pos = self.pending_pos(id)?;
        Ok(self.pending.remove(pos))
    }

    /// Série en attente et position de l'exercice, encore à faire, dans celle-ci.
    fn pending_item(&mut self, id: u64, exercise_id: &str) -> Result<(&mut Draw, usize), String> {
        let pos = self.pending_pos(id)?;
        let draw = &mut self.pending[pos];
        let item = draw
            .items
            .iter()
            .position(|it| it.exercise_id == exercise_id)
            .ok_or("Cet exercice n'est plus dans la série.")?;
        if draw.items[item].done {
            return Err("Cet exercice est déjà fait.".into());
        }
        Ok((draw, item))
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

const NOT_ENOUGH_JOKERS: &str = "Pas assez de jokers.";

/// Jokers pour sauter tout ce qui reste d'une série.
pub fn skip_set_cost(draw: &Draw) -> u32 {
    draw.items
        .iter()
        .filter(|it| !it.done)
        .map(|it| it.skip_cost)
        .sum()
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
