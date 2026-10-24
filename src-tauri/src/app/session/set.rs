//! A set and its exercises.

use super::{Intensity, HALVE_COST, SKIP_COST};
use crate::app::Exercise;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DrawItem {
    pub exercise_id: String,
    pub name: String,
    pub unit: String,
    pub amount: u32,
    /// Number drawn between the exercise's bounds, before the intensity and the joker.
    #[serde(skip)]
    pub base: u32,
    /// True once halved with a joker.
    pub halved: bool,
    /// True once marked done: it stays displayed until the end of the set.
    pub done: bool,
    /// Jokers to halve it, absent when no longer possible.
    pub halve_cost: Option<u32>,
    /// Jokers to skip what is left of it.
    pub skip_cost: u32,
}

impl DrawItem {
    /// Draws the exercise's number between its bounds, scaled by the intensity.
    pub(super) fn drawn<R: Rng + ?Sized>(ex: &Exercise, intensity: Intensity, rng: &mut R) -> Self {
        DrawItem {
            exercise_id: ex.id.clone(),
            name: ex.name.clone(),
            unit: ex.unit.clone(),
            amount: 0,
            base: rng.random_range(ex.min..=ex.max),
            halved: false,
            done: false,
            halve_cost: None,
            skip_cost: SKIP_COST,
        }
        .at(intensity)
    }

    /// Takes as number the drawn one scaled by `intensity`, halved first if it was.
    pub(super) fn at(self, intensity: Intensity) -> Self {
        if self.halved {
            let amount = intensity.scale(intensity.halve(self.base));
            DrawItem { amount, ..self }
        } else {
            let amount = intensity.scale(self.base);
            self.with_amount(amount)
        }
    }

    fn with_amount(mut self, amount: u32) -> Self {
        self.amount = amount;
        self.halve_cost = (amount >= 2).then_some(HALVE_COST);
        // An exercise that cannot be halved costs a skip as if it had been.
        self.skip_cost = SKIP_COST - self.halve_cost.map_or(HALVE_COST, |_| 0);
        self
    }

    /// Takes the exercise's name and unit and clamps its number between its bounds scaled by the intensity,
    /// if it was not halved.
    pub(super) fn refit(&mut self, ex: &Exercise, intensity: Intensity) {
        self.name = ex.name.clone();
        self.unit = ex.unit.clone();
        if !self.halved {
            self.base = self.base.clamp(ex.min, ex.max);
            *self = self.clone().at(intensity);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Draw {
    /// A UUIDv7, given when the set is drawn.
    pub id: String,
    pub game_id: String,
    pub game_name: String,
    pub event_id: String,
    pub event_name: String,
    pub items: Vec<DrawItem>,
    /// Jokers earned on completion (reward event).
    pub jokers: u32,
    /// True when the event's limit for the session is exceeded.
    pub capped: bool,
    /// True when the session's limit gave no set.
    pub limit_reached: bool,
    pub trigger_count: u32,
    pub max_per_session: Option<u32>,
    /// Exercises as drawn, reserved for the next trigger if the set is dismissed before being started.
    #[serde(skip)]
    pub drawn: Vec<DrawItem>,
    /// Jokers spent on the set.
    #[serde(skip)]
    pub spent: u32,
    /// Exercises it asked when triggered, and their estimated time in seconds, counted against the session's
    /// limit: a saved configuration changing the set never recounts them.
    #[serde(skip)]
    pub asked: u32,
    #[serde(skip)]
    pub time: u32,
    /// Exercises the session's limit left it, fewer than its event draws: a saved configuration never adds more.
    #[serde(skip)]
    pub reduced: Option<u32>,
}

impl Draw {
    /// True once an exercise of the set is done.
    pub fn started(&self) -> bool {
        self.items.iter().any(|it| it.done)
    }

    /// Spends jokers on the set: it no longer earns its own.
    pub(super) fn spend(&mut self, jokers: u32) {
        self.jokers = 0;
        self.spent += jokers;
    }
}
