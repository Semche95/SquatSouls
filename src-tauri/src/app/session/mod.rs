//! Session: intensity, draw, sets, progress, jokers, adjustment to the configuration and summary. Saved by the
//! store at each change, it resumes at launch without the time the app was closed.

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

use super::{AppError, Config, Game};
use serde::Serialize;
use std::collections::HashMap;

/// Fields are visible to the store, which saves the session in progress and reads it back at launch.
#[derive(Debug, Default)]
pub struct Session {
    /// Absent until the session is started, by its button or its first event.
    pub started_at: Option<i64>,
    /// Game of the session, set at the latest when it starts and locked until the next session.
    pub(crate) game_id: Option<String>,
    pub(crate) intensity: Intensity,
    pub(crate) triggers: HashMap<(String, String), u32>,
    pub(crate) pending: Vec<Draw>,
    pub(crate) done: Vec<LogEntry>,
    /// Sets done already written to the history.
    pub(crate) recorded: usize,
    pub(crate) jokers: u32,
    pub(crate) jokers_earned: u32,
    pub(crate) jokers_spent: u32,
    /// Exercises not drawn yet in the current round: each comes out once before another comes back.
    pub(crate) bag: Vec<String>,
    /// Exercises of the last drawn set, which the next round does not pick again right away.
    pub(crate) last: Vec<String>,
    /// Exercises of dismissed sets, by game and event: the next trigger gives them back.
    pub(crate) reserved: HashMap<(String, String), Vec<DrawItem>>,
    pub(crate) next_id: u64,
    /// Seconds the session lasted in the periods the app was open before the current one.
    pub(crate) counted: i64,
    /// Start of the current period the app is open in the session: its start, then each reopening.
    pub(crate) opened_at: i64,
    /// How the previous session ended, kept until this one starts.
    pub(crate) ended: Option<SessionEnd>,
}

/// Why a session ended by itself: on saving a configuration it can no longer be played on, or at launch when the
/// session in progress could not be read back.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EndCause {
    GameRemoved,
    NoExercise,
    Unreadable,
}

/// How a session ended; a cancelled one leaves none.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEnd {
    /// False when it ended by itself without any exercise done: nothing was saved, as on a cancellation.
    pub saved: bool,
    /// Why it ended by itself, none when ended with **End the session**.
    pub cause: Option<EndCause>,
}

impl Session {
    /// Next session, waiting to be started after the one that ended as `end`.
    pub fn after(end: SessionEnd) -> Self {
        Session {
            ended: Some(end),
            ..Session::default()
        }
    }

    /// Starts the session on the game at `now`, unless it already is; returns its start. Without an event or an
    /// included exercise, nothing could be played.
    pub fn start(&mut self, config: &Config, game_id: &str, now: i64) -> Result<i64, AppError> {
        let game = self.session_game(config, game_id)?;
        if game.events.is_empty() {
            return Err(AppError::new("start_without_event").with("game", game.name.as_str()));
        }
        self.game_id = Some(game.id.clone());
        self.ended = None;
        Ok(self.started(now))
    }

    /// Start of the session, started at `now` unless it already is.
    fn started(&mut self, now: i64) -> i64 {
        *self.started_at.get_or_insert_with(|| {
            self.opened_at = now;
            now
        })
    }

    /// Resumes the session read back at launch: the period the app was open before ended at `closed`, the time it
    /// was closed does not count, and a new period starts at `now`.
    pub fn resume(&mut self, closed: i64, now: i64) {
        self.counted += (closed - self.opened_at).max(0);
        self.opened_at = now;
    }

    /// Game of `config` the session can be played on: the session's own once it has one, as long as an exercise
    /// is included. Checked before anything changes, so that a refused draw leaves the session as it was.
    fn session_game<'a>(&self, config: &'a Config, game_id: &str) -> Result<&'a Game, AppError> {
        let game = config
            .games
            .iter()
            .find(|g| g.id == game_id)
            .ok_or_else(|| AppError::new("unknown_game").with("id", game_id))?;
        if self.game_id.as_ref().is_some_and(|id| id != &game.id) {
            return Err(AppError::new("session_other_game"));
        }
        if config.active_exercises().next().is_none() {
            return Err(AppError::new("start_without_exercise"));
        }
        Ok(game)
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
