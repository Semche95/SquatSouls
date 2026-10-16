//! Session: intensity, draw, sets, progress, jokers, adjustment to the configuration and summary.

mod draw;
mod intensity;
mod jokers;
mod progress;
mod refit;
mod set;
mod summary;

pub use intensity::*;
pub use jokers::*;
pub use set::*;
pub use summary::*;

use super::AppError;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Session {
    /// Absent until the session is started, by its button or its first event.
    pub started_at: Option<i64>,
    game_id: Option<String>,
    intensity: Intensity,
    triggers: HashMap<(String, String), u32>,
    pending: Vec<Draw>,
    done: Vec<LogEntry>,
    jokers: u32,
    jokers_earned: u32,
    jokers_spent: u32,
    /// Exercises not drawn yet in the current round: each comes out once before another comes back.
    bag: Vec<String>,
    /// Exercises of the last drawn set, which the next round does not pick again right away.
    last: Vec<String>,
    /// Exercises of dismissed sets, by game and event: the next trigger gives them back.
    reserved: HashMap<(String, String), Vec<DrawItem>>,
    next_id: u64,
}

impl Session {
    /// Starts the session at `now`, unless it already is; returns its start.
    pub fn start(&mut self, now: i64) -> i64 {
        *self.started_at.get_or_insert(now)
    }

    /// Sets the intensity of the next draws and, as new bounds do, applies it right away to the sets to do
    /// and to the reserved sets.
    pub fn set_intensity(&mut self, intensity: Intensity) {
        self.intensity = intensity;
        let pending = self.pending.iter_mut().flat_map(|draw| {
            draw.drawn
                .iter_mut()
                .chain(draw.items.iter_mut().filter(|it| !it.done))
        });
        for it in pending.chain(self.reserved.values_mut().flatten()) {
            *it = it.clone().at(intensity);
        }
    }

    /// Checks that the session can end: at least one set done and none left to do.
    pub fn check_end(&self) -> Result<(), AppError> {
        if !self.pending.is_empty() {
            return Err(AppError::new("end_with_pending_sets"));
        }
        if self.done.is_empty() {
            return Err(AppError::new("end_without_set"));
        }
        Ok(())
    }

    fn pending_pos(&self, id: u64) -> Result<usize, AppError> {
        self.pending
            .iter()
            .position(|d| d.id == id)
            .ok_or(AppError::new("set_not_pending"))
    }

    /// Position of the set, which must be the oldest one: sets are done in the order they were drawn.
    fn current_pos(&self, id: u64) -> Result<usize, AppError> {
        match self.pending_pos(id)? {
            0 => Ok(0),
            _ => Err(AppError::new("set_not_current")),
        }
    }

    fn pending_draw(&self, id: u64) -> Result<&Draw, AppError> {
        Ok(&self.pending[self.pending_pos(id)?])
    }

    fn take_pending(&mut self, id: u64) -> Result<Draw, AppError> {
        let pos = self.pending_pos(id)?;
        Ok(self.pending.remove(pos))
    }

    /// Pending set and position in it of the exercise, still to do.
    fn pending_item(&mut self, id: u64, exercise_id: &str) -> Result<(&mut Draw, usize), AppError> {
        let pos = self.current_pos(id)?;
        let draw = &mut self.pending[pos];
        let item = draw
            .items
            .iter()
            .position(|it| it.exercise_id == exercise_id)
            .ok_or(AppError::new("exercise_not_in_set"))?;
        if draw.items[item].done {
            return Err(AppError::new("exercise_already_done"));
        }
        Ok((draw, item))
    }
}
