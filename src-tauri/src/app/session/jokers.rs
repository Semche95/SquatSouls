//! Jokers: their costs and how they are spent.

use super::{Draw, Session};
use crate::app::AppError;
use serde::Serialize;

/// Jokers to halve an exercise.
pub const HALVE_COST: u32 = 1;
/// Jokers to skip a whole exercise.
pub const SKIP_COST: u32 = 2;

#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq)]
pub struct Jokers {
    pub earned: u32,
    pub spent: u32,
}

impl Session {
    /// Halves an exercise (rounded as the intensity does): the other half is not done.
    pub fn halve(&mut self, id: u64, exercise_id: &str) -> Result<(), AppError> {
        let (jokers, intensity) = (self.jokers, self.intensity);
        let (draw, pos) = self.pending_item(id, exercise_id)?;
        let item = &mut draw.items[pos];
        let cost = item
            .halve_cost
            .ok_or(AppError::new("exercise_not_reducible"))?;
        if jokers < cost {
            return Err(AppError::new(NOT_ENOUGH_JOKERS));
        }
        item.amount = intensity.halve(item.amount);
        item.halved = true;
        item.halve_cost = None;
        item.skip_cost -= cost;
        draw.spend(cost);
        self.spend(cost);
        Ok(())
    }

    /// Skips an exercise: the rest of the set remains to do, unless nothing is left.
    pub fn skip_exercise(&mut self, id: u64, exercise_id: &str) -> Result<(), AppError> {
        let jokers = self.jokers;
        let (draw, pos) = self.pending_item(id, exercise_id)?;
        if jokers < draw.items[pos].skip_cost {
            return Err(AppError::new(NOT_ENOUGH_JOKERS));
        }
        let cost = draw.items.remove(pos).skip_cost;
        draw.spend(cost);
        self.spend(cost);
        self.finish_if_done(id)?;
        Ok(())
    }

    /// Skips what is left of a set: the trigger stays counted.
    pub fn skip_set(&mut self, id: u64) -> Result<(), AppError> {
        let draw = &self.pending[self.current_pos(id)?];
        let cost = skip_set_cost(draw);
        if self.jokers < cost {
            return Err(AppError::new(NOT_ENOUGH_JOKERS));
        }
        self.take_pending(id)?;
        self.spend(cost);
        Ok(())
    }

    /// Adds the jokers earned by a completed set.
    pub(super) fn earn(&mut self, jokers: u32) {
        self.jokers = self.jokers.saturating_add(jokers);
        self.jokers_earned = self.jokers_earned.saturating_add(jokers);
    }

    fn spend(&mut self, jokers: u32) {
        self.jokers -= jokers;
        self.jokers_spent += jokers;
    }

    /// A set dropped before being started: the jokers spent on it are lost without ever counting.
    pub(super) fn drop_unstarted(&mut self, draw: &Draw) {
        if !draw.started() {
            self.jokers_spent -= draw.spent;
        }
    }

    /// Jokers earned and spent as the summary counts them: those spent on a set to do not started yet are left
    /// out, as when the session stops there.
    pub fn jokers(&self) -> Jokers {
        let unstarted: u32 = self
            .pending
            .iter()
            .filter(|d| !d.started())
            .map(|d| d.spent)
            .sum();
        Jokers {
            earned: self.jokers_earned,
            spent: self.jokers_spent - unstarted,
        }
    }
}

const NOT_ENOUGH_JOKERS: &str = "not_enough_jokers";

/// Jokers to skip everything left of a set.
pub fn skip_set_cost(draw: &Draw) -> u32 {
    draw.items
        .iter()
        .filter(|it| !it.done)
        .map(|it| it.skip_cost)
        .sum()
}
