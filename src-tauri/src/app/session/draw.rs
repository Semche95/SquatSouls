//! Draw of the exercises of a set.

use super::{Draw, DrawItem, Session};
use crate::app::{AppError, Config, Exercise};
use rand::Rng;

impl Session {
    pub fn draw<R: Rng + ?Sized>(
        &mut self,
        config: &Config,
        game_id: &str,
        event_id: &str,
        rng: &mut R,
    ) -> Result<Draw, AppError> {
        let game = config
            .games
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| AppError::new("unknown_game").with("id", game_id))?;
        if self.game_id.as_ref().is_some_and(|id| id != &game.id) {
            return Err(AppError::new("session_other_game"));
        }
        let ev = game
            .events
            .iter()
            .find(|e| e.id == event_id)
            .ok_or_else(|| AppError::new("unknown_event").with("id", event_id))?;

        self.game_id = Some(game.id.clone());
        let key = (game.id.clone(), ev.id.clone());
        let count = self.triggers.entry(key.clone()).or_insert(0);
        // Once the limit is reached, the counter no longer moves.
        let capped = ev.max_per_session.is_some_and(|max| *count >= max);
        if !capped {
            *count += 1;
        }
        let trigger_count = *count;

        let items = if capped {
            Vec::new()
        } else if let Some(items) = self.reserved.remove(&key) {
            items
        } else {
            self.pick(config, ev.draws as usize, &[], rng)
                .into_iter()
                .map(|ex| DrawItem::drawn(ex, self.intensity, rng))
                .collect()
        };

        self.next_id += 1;
        let draw = Draw {
            id: self.next_id,
            game_id: game.id.clone(),
            game_name: game.name.clone(),
            event_id: ev.id.clone(),
            event_name: ev.name.clone(),
            items: items.clone(),
            jokers: if capped { 0 } else { ev.jokers },
            capped,
            trigger_count,
            max_per_session: ev.max_per_session,
            drawn: items,
            forfeited: false,
        };
        if !capped {
            self.last = draw.items.iter().map(|it| it.exercise_id.clone()).collect();
            self.pending.push(draw.clone());
        }
        Ok(draw)
    }

    /// Draws `n` distinct exercises, outside `exclude`, from the current round, starting a new round when it runs out.
    /// An exercise of the last set does not come out again right away, unless it is the only one left to draw.
    /// Stops early when there is no exercise left to draw.
    pub(super) fn pick<'a, R: Rng + ?Sized>(
        &mut self,
        config: &'a Config,
        n: usize,
        exclude: &[String],
        rng: &mut R,
    ) -> Vec<&'a Exercise> {
        let mut picks: Vec<&Exercise> = Vec::new();
        let mut refilled = false;
        while picks.len() < n {
            let candidates: Vec<usize> = (0..self.bag.len())
                .filter(|&i| {
                    !exclude.contains(&self.bag[i]) && picks.iter().all(|ex| ex.id != self.bag[i])
                })
                .collect();
            if candidates.is_empty() {
                if refilled {
                    break;
                }
                self.bag = config.active_exercises().map(|ex| ex.id.clone()).collect();
                refilled = true;
                continue;
            }
            let fresh: Vec<usize> = candidates
                .iter()
                .copied()
                .filter(|&i| !self.last.contains(&self.bag[i]))
                .collect();
            let pool = if fresh.is_empty() {
                &candidates
            } else {
                &fresh
            };
            let id = self.bag.swap_remove(pool[rng.random_range(0..pool.len())]);
            picks.extend(config.active_exercises().find(|ex| ex.id == id));
        }
        picks
    }
}
