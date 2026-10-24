//! Progress of a set: exercises done and set dismissed.

use super::{Draw, LogEntry, Session};
use crate::app::AppError;

impl Session {
    /// Marks an exercise done, in the order of the set: it moves to the sets done. The finished set earns its jokers.
    pub fn complete_exercise(
        &mut self,
        id: &str,
        exercise_id: &str,
        now: i64,
    ) -> Result<LogEntry, AppError> {
        // The set's draw already gave the session its game.
        self.started(now);
        let session = self.id.clone();
        let (draw, pos) = self.pending_item(id, exercise_id)?;
        if draw.items[..pos].iter().any(|it| !it.done) {
            return Err(AppError::new("exercise_not_next"));
        }
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
        self.done.push(entry.clone());
        self.finish_if_done(id)?;
        Ok(entry)
    }

    /// Removes the set when nothing is left to do, with the jokers it still earns.
    pub(super) fn finish_if_done(&mut self, id: &str) -> Result<(), AppError> {
        if self.pending_draw(id)?.items.iter().all(|it| it.done) {
            let draw = self.take_pending(id)?;
            self.earn(draw.jokers);
        }
        Ok(())
    }

    /// Dismisses a set. Not started, it is a misclick: the trigger is no longer counted, its exercises as drawn
    /// are reserved for the next trigger of the same event, and the jokers spent on it never count. Started, it
    /// stops there: its exercises done stay counted, as does the trigger, without the jokers of the set.
    pub fn dismiss(&mut self, id: &str) -> Result<(), AppError> {
        let draw = self.take_pending(id)?;
        self.dismissed(draw);
        Ok(())
    }

    /// Dismisses every set to do, so that the session keeps only what was done; true if an exercise was done,
    /// the session then having something to end with.
    pub fn dismiss_all(&mut self) -> bool {
        for draw in std::mem::take(&mut self.pending) {
            self.dismissed(draw);
        }
        !self.done.is_empty()
    }

    fn dismissed(&mut self, draw: Draw) {
        if draw.started() {
            return;
        }
        self.drop_unstarted(&draw);
        self.unask(draw.asked, draw.time);
        let key = (draw.game_id, draw.event_id);
        if let Some(count) = self.triggers.get_mut(&key) {
            *count = count.saturating_sub(1);
        }
        self.reserved.insert(key, draw.drawn);
    }
}
