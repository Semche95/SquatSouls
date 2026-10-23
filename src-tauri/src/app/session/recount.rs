//! Exercises asked of a session saved before they were counted, counted again at launch from what it kept.

use super::{estimate, estimate_of, Session};
use crate::app::Config;
use std::collections::HashMap;

impl Session {
    /// Counts again the exercises asked and their estimated time, if the session was saved before they were
    /// counted. The sets to do count as drawn. The sets already finished count the exercises their event draws: those
    /// done at their number, the others, skipped or stopped, at the middle of their range on average.
    pub fn recount_asked(&mut self, config: &Config) {
        if !std::mem::take(&mut self.asked_unknown) {
            return;
        }
        let mut pending_triggers: HashMap<(String, String), u32> = HashMap::new();
        let (mut asked, mut time, mut done_pending, mut done_pending_time) = (0, 0u32, 0, 0u32);
        for draw in &mut self.pending {
            draw.asked = draw.drawn.len() as u32;
            draw.time = draw.drawn.iter().map(|it| estimate(config, it)).sum();
            asked += draw.drawn.len();
            time = time.saturating_add(draw.time);
            for it in draw.items.iter().filter(|it| it.done) {
                done_pending += 1;
                done_pending_time = done_pending_time.saturating_add(estimate(config, it));
            }
            *pending_triggers
                .entry((draw.game_id.clone(), draw.event_id.clone()))
                .or_default() += 1;
        }

        // Exercises of the finished sets, as many as their event draws when it still exists.
        let active = config.active_exercises().count() as u32;
        let finished: u32 = self
            .triggers
            .iter()
            .map(|(key, &count)| {
                let sets = count.saturating_sub(pending_triggers.get(key).copied().unwrap_or(0));
                let draws = config
                    .games
                    .iter()
                    .find(|g| g.id == key.0)
                    .and_then(|g| g.events.iter().find(|e| e.id == key.1))
                    .map_or(0, |ev| ev.draws.min(active));
                sets.saturating_mul(draws)
            })
            .sum();
        let done = self.done.len().saturating_sub(done_pending);
        let done_time = self
            .done
            .iter()
            .map(|entry| {
                let ex = config.exercises.iter().find(|ex| ex.name == entry.exercise);
                estimate_of(config, ex, entry.amount)
            })
            .fold(0u32, u32::saturating_add)
            .saturating_sub(done_pending_time);
        let missing = (finished as usize).saturating_sub(done);

        self.asked = (asked + done + missing) as u32;
        self.asked_time = time
            .saturating_add(done_time)
            .saturating_add((missing as u32).saturating_mul(self.average_estimate(config)));
    }

    /// Estimated time of an included exercise at the middle of its range, scaled by the intensity, on average.
    fn average_estimate(&self, config: &Config) -> u32 {
        let times: Vec<u32> = config
            .active_exercises()
            .map(|ex| {
                estimate_of(
                    config,
                    Some(ex),
                    self.intensity.scale((ex.min + ex.max) / 2),
                )
            })
            .collect();
        match times.len() {
            0 => 0,
            n => times.iter().sum::<u32>() / n as u32,
        }
    }
}
