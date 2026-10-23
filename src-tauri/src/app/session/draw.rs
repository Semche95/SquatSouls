//! Draw of the exercises of a set.

use super::{estimate, Draw, DrawItem, Session};
use crate::app::{AppError, Config, Exercise};
use rand::Rng;

impl Session {
    /// Draws a set for the event at `now`. Beyond the event's limit or the session's, it gives none and the trigger
    /// is not counted; a set that would exceed the session's limit keeps only its exercises that fit.
    pub fn draw<R: Rng + ?Sized>(
        &mut self,
        config: &Config,
        game_id: &str,
        event_id: &str,
        now: i64,
        rng: &mut R,
    ) -> Result<Draw, AppError> {
        let game = self.session_game(config, game_id)?;
        let ev = game
            .events
            .iter()
            .find(|e| e.id == event_id)
            .ok_or_else(|| AppError::new("unknown_event").with("id", event_id))?;

        self.game_id = Some(game.id.clone());
        let key = (game.id.clone(), ev.id.clone());
        // Once the limit is reached, the counter no longer moves.
        let capped = ev
            .max_per_session
            .is_some_and(|max| self.triggers.get(&key).copied().unwrap_or(0) >= max);
        let (items, reduced) = if capped || self.limit_reached(now) {
            (Vec::new(), false)
        } else {
            self.asked_items(config, &key, ev.draws as usize, rng)
        };
        let count = self.triggers.entry(key).or_insert(0);
        if !items.is_empty() {
            *count += 1;
        }
        let trigger_count = *count;
        let time = items.iter().map(|it| estimate(config, it)).sum();
        let items_len = items.len() as u32;

        self.next_id += 1;
        let draw = Draw {
            id: self.next_id,
            game_id: game.id.clone(),
            game_name: game.name.clone(),
            event_id: ev.id.clone(),
            event_name: ev.name.clone(),
            items: items.clone(),
            jokers: if items.is_empty() { 0 } else { ev.jokers },
            capped,
            limit_reached: !capped && items.is_empty(),
            trigger_count,
            max_per_session: ev.max_per_session,
            drawn: items,
            spent: 0,
            asked: items_len,
            time,
            reduced: reduced.then_some(items_len),
        };
        if !draw.items.is_empty() {
            self.ask(draw.asked, time);
            self.last = draw.items.iter().map(|it| it.exercise_id.clone()).collect();
            self.pending.push(draw.clone());
        }
        Ok(draw)
    }

    /// Exercises of the event's next set, those it reserved or else drawn, as many as fit within the session's
    /// limit, and whether the limit left out some. Those that do not fit go back to the round, or stay reserved when
    /// none fits.
    fn asked_items<R: Rng + ?Sized>(
        &mut self,
        config: &Config,
        key: &(String, String),
        draws: usize,
        rng: &mut R,
    ) -> (Vec<DrawItem>, bool) {
        let room = self.limit_room();
        let reserved = self.reserved.remove(key);
        let reserve = reserved.is_some();
        let mut asked = reserved.unwrap_or_else(|| {
            self.pick(config, draws.min(room), &[], rng)
                .into_iter()
                .map(|ex| DrawItem::drawn(ex, self.intensity, rng))
                .collect()
        });
        let cut = asked.split_off(room.min(asked.len()));
        let (items, mut rest) = self.fit(config, asked);
        rest.extend(cut);
        let reduced = !rest.is_empty() || (!reserve && draws > room);
        if reserve && items.is_empty() {
            self.reserved.insert(key.clone(), rest);
        } else {
            self.give_back(rest);
        }
        (items, reduced)
    }

    /// Puts exercises asked back in the round, those it does not hold yet: a reserved set dates from a round that
    /// may have ended since.
    fn give_back(&mut self, items: Vec<DrawItem>) {
        for it in items {
            if !self.bag.contains(&it.exercise_id) {
                self.bag.push(it.exercise_id);
            }
        }
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
