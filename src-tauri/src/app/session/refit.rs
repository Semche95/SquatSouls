//! Adjustment of the session to a new configuration.

use super::{DrawItem, Session};
use crate::app::{Config, Game, GameEvent};
use rand::Rng;

impl Session {
    /// Adapts the sets to do and the reserved sets to a new configuration, and restarts the round.
    /// The set of a removed event is dropped; a set finished by the adjustment earns its jokers.
    pub fn apply_config<R: Rng + ?Sized>(&mut self, config: &Config, rng: &mut R) {
        self.bag.clear();
        for mut draw in std::mem::take(&mut self.pending) {
            let Some((game, ev)) = find_event(config, &draw.game_id, &draw.event_id) else {
                continue;
            };
            draw.game_name = game.name.clone();
            draw.event_name = ev.name.clone();
            draw.max_per_session = ev.max_per_session;
            if !draw.forfeited {
                draw.jokers = ev.jokers;
            }
            self.refit(
                config,
                ev.draws,
                &mut draw.drawn,
                Some(&mut draw.items),
                rng,
            );
            if draw.items.iter().all(|it| it.done) {
                self.jokers += draw.jokers;
                self.jokers_earned += draw.jokers;
            } else {
                self.pending.push(draw);
            }
        }
        for (key, mut drawn) in std::mem::take(&mut self.reserved) {
            if let Some((_, ev)) = find_event(config, &key.0, &key.1) {
                self.refit(config, ev.draws, &mut drawn, None, rng);
                self.reserved.insert(key, drawn);
            }
        }
    }

    /// Adjusts a set as drawn (`drawn`) and, for a set to do, its remaining exercises (`items`):
    /// a removed or disabled exercise is replaced, a number is clamped between the bounds and the set gains or loses
    /// exercises to count `draws` of them. Exercises already done do not change.
    fn refit<R: Rng + ?Sized>(
        &mut self,
        config: &Config,
        draws: u32,
        drawn: &mut Vec<DrawItem>,
        mut items: Option<&mut Vec<DrawItem>>,
        rng: &mut R,
    ) {
        let exercise = |id: &str| config.active_exercises().find(|ex| ex.id == id);
        let ids = |list: &[DrawItem]| -> Vec<String> {
            list.iter().map(|it| it.exercise_id.clone()).collect()
        };
        let in_items = |items: &Option<&mut Vec<DrawItem>>, id: &str| {
            items
                .as_ref()
                .and_then(|items| items.iter().find(|it| it.exercise_id == id))
                .map(|it| it.done)
        };

        // Removed or disabled exercises: replaced by others, except those already done.
        for i in 0..drawn.len() {
            let id = drawn[i].exercise_id.clone();
            if exercise(&id).is_some() {
                continue;
            }
            let mut exclude = ids(drawn);
            if let Some(items) = &items {
                exclude.extend(ids(items));
            }
            let Some(ex) = self.pick(config, 1, &exclude, rng).pop() else {
                continue;
            };
            let new = DrawItem::drawn(ex, self.intensity, rng);
            if let Some(items) = items.as_mut() {
                if let Some(it) = items.iter_mut().find(|it| it.exercise_id == id && !it.done) {
                    *it = new.clone();
                }
            }
            drawn[i] = new;
        }
        drawn.retain(|it| exercise(&it.exercise_id).is_some());

        for it in drawn.iter_mut() {
            it.refit(
                exercise(&it.exercise_id).expect("retained exercise"),
                self.intensity,
            );
        }
        if let Some(items) = items.as_mut() {
            for it in items.iter_mut().filter(|it| !it.done) {
                if let Some(ex) = exercise(&it.exercise_id) {
                    it.refit(ex, self.intensity);
                }
            }
        }

        // Fewer exercises: remove first those already skipped, then those left to do, then those done.
        let draws = draws as usize;
        for rank in [None, Some(false), Some(true)] {
            while drawn.len() > draws {
                let Some(pos) = drawn
                    .iter()
                    .rposition(|it| in_items(&items, &it.exercise_id) == rank)
                else {
                    break;
                };
                let removed = drawn.remove(pos);
                if let Some(items) = items.as_mut() {
                    if rank == Some(false) {
                        items.retain(|it| it.exercise_id != removed.exercise_id);
                    }
                }
            }
        }

        // More exercises: draw the missing ones.
        if drawn.len() < draws {
            let mut exclude = ids(drawn);
            if let Some(items) = &items {
                exclude.extend(ids(items));
            }
            for ex in self.pick(config, draws - drawn.len(), &exclude, rng) {
                let new = DrawItem::drawn(ex, self.intensity, rng);
                if let Some(items) = items.as_mut() {
                    items.push(new.clone());
                }
                drawn.push(new);
            }
        }
    }

    /// True if the session started on a game missing from `config`.
    pub fn game_removed(&self, config: &Config) -> bool {
        self.game_id
            .as_ref()
            .is_some_and(|id| config.games.iter().all(|g| &g.id != id))
    }
}

fn find_event<'a>(
    config: &'a Config,
    game_id: &str,
    event_id: &str,
) -> Option<(&'a Game, &'a GameEvent)> {
    let game = config.games.iter().find(|g| g.id == game_id)?;
    Some((game, game.events.iter().find(|e| e.id == event_id)?))
}
