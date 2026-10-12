//! Draw and session.

use super::{AppError, Config, Exercise, Game, GameEvent};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// Jokers to halve an exercise.
pub const HALVE_COST: u32 = 1;
/// Jokers to skip a whole exercise.
pub const SKIP_COST: u32 = 2;

/// Intensity of the session: scales the drawn numbers without touching the configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Intensity {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Intensity {
    pub fn as_str(self) -> &'static str {
        match self {
            Intensity::Easy => "easy",
            Intensity::Normal => "normal",
            Intensity::Hard => "hard",
        }
    }

    /// Scales a number between an exercise's bounds: half for easy, one and a half for hard, rounded up.
    pub fn scale(self, amount: u32) -> u32 {
        let (num, den) = match self {
            Intensity::Easy => (1, 2),
            Intensity::Normal => (1, 1),
            Intensity::Hard => (3, 2),
        };
        (amount * num).div_ceil(den)
    }
}

impl FromStr for Intensity {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        match value {
            "easy" => Ok(Intensity::Easy),
            "normal" => Ok(Intensity::Normal),
            "hard" => Ok(Intensity::Hard),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DrawItem {
    pub exercise_id: String,
    pub name: String,
    pub unit: String,
    pub amount: u32,
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
    fn drawn<R: Rng + ?Sized>(ex: &Exercise, intensity: Intensity, rng: &mut R) -> Self {
        DrawItem {
            exercise_id: ex.id.clone(),
            name: ex.name.clone(),
            unit: ex.unit.clone(),
            amount: 0,
            halved: false,
            done: false,
            halve_cost: None,
            skip_cost: SKIP_COST,
        }
        .with_amount(intensity.scale(rng.random_range(ex.min..=ex.max)))
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
    fn refit(&mut self, ex: &Exercise, intensity: Intensity) {
        self.name = ex.name.clone();
        self.unit = ex.unit.clone();
        if !self.halved {
            let amount = self
                .amount
                .clamp(intensity.scale(ex.min), intensity.scale(ex.max));
            *self = self.clone().with_amount(amount);
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
    /// Jokers earned on completion (reward event).
    pub jokers: u32,
    /// True when the event's limit for the session is exceeded.
    pub capped: bool,
    pub trigger_count: u32,
    pub max_per_session: Option<u32>,
    /// Exercises of the set already done, removed if it is dismissed.
    #[serde(skip)]
    pub logged: Vec<LogEntry>,
    /// Exercises as drawn, reserved for the next trigger if the set is dismissed.
    #[serde(skip)]
    pub drawn: Vec<DrawItem>,
    /// True once jokers have been spent on the set.
    #[serde(skip)]
    pub forfeited: bool,
}

impl Draw {
    /// Spends jokers on the set: it no longer earns its own.
    fn forfeit(&mut self) {
        self.jokers = 0;
        self.forfeited = true;
    }
}

/// A set done, as saved in the history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub at: i64,
    /// Start of the session.
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

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TriggerCount {
    pub game_id: String,
    pub event_id: String,
    pub count: u32,
}

/// Times an event was triggered in a session, by its name.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EventCount {
    pub event: String,
    pub count: u32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq)]
pub struct Jokers {
    pub earned: u32,
    pub spent: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    /// Absent until the session is started.
    pub started_at: Option<i64>,
    /// Game of the session, set on the first event.
    pub game_id: Option<String>,
    /// Intensity of the session, locked with its game.
    pub intensity: Intensity,
    pub pending: Vec<Draw>,
    pub sets: usize,
    pub totals: Vec<Total>,
    /// Sets done, most recent first.
    pub log: Vec<LogEntry>,
    pub triggers: Vec<TriggerCount>,
    pub jokers: u32,
    pub jokers_earned: u32,
    pub jokers_spent: u32,
}

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

    /// Sets the intensity of the next draws, refused once the game is locked on another one.
    pub fn set_intensity(&mut self, intensity: Intensity) -> Result<(), AppError> {
        if self.game_id.is_some() && self.intensity != intensity {
            return Err(AppError::new("session_other_intensity"));
        }
        self.intensity = intensity;
        Ok(())
    }

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
            logged: Vec::new(),
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
    fn pick<'a, R: Rng + ?Sized>(
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

    /// Marks an exercise done, in the order of the set: it moves to the sets done. The finished set earns its jokers.
    pub fn complete_exercise(
        &mut self,
        id: u64,
        exercise_id: &str,
        now: i64,
    ) -> Result<LogEntry, AppError> {
        let session = self.start(now);
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
        draw.logged.push(entry.clone());
        self.done.push(entry.clone());
        self.finish_if_done(id)?;
        Ok(entry)
    }

    /// Removes the set when nothing is left to do, with the jokers it still earns.
    fn finish_if_done(&mut self, id: u64) -> Result<(), AppError> {
        if self.pending_draw(id)?.items.iter().all(|it| it.done) {
            let draw = self.take_pending(id)?;
            self.jokers += draw.jokers;
            self.jokers_earned += draw.jokers;
        }
        Ok(())
    }

    /// Dismisses a trigger: it is no longer counted, nor are the set's exercises already done, returned to remove them from the history.
    /// Its exercises as drawn are reserved for the next trigger of the same event.
    pub fn dismiss(&mut self, id: u64) -> Result<Vec<LogEntry>, AppError> {
        let draw = self.take_pending(id)?;
        let key = (draw.game_id, draw.event_id);
        if let Some(count) = self.triggers.get_mut(&key) {
            *count = count.saturating_sub(1);
        }
        self.reserved.insert(key, draw.drawn);
        for entry in &draw.logged {
            if let Some(pos) = self.done.iter().position(|e| e == entry) {
                self.done.remove(pos);
            }
        }
        Ok(draw.logged)
    }

    /// Halves an exercise (rounded up): the other half is not done.
    pub fn halve(&mut self, id: u64, exercise_id: &str) -> Result<(), AppError> {
        let jokers = self.jokers;
        let (draw, pos) = self.pending_item(id, exercise_id)?;
        let item = &mut draw.items[pos];
        let cost = item
            .halve_cost
            .ok_or(AppError::new("exercise_not_reducible"))?;
        if jokers < cost {
            return Err(AppError::new(NOT_ENOUGH_JOKERS));
        }
        item.amount -= item.amount / 2;
        item.halved = true;
        item.halve_cost = None;
        item.skip_cost -= cost;
        draw.forfeit();
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
        draw.forfeit();
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

    fn spend(&mut self, jokers: u32) {
        self.jokers -= jokers;
        self.jokers_spent += jokers;
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

    /// Events triggered in the session's game, in the order of its events.
    pub fn event_counts(&self, config: &Config) -> Vec<EventCount> {
        let Some(game) = config
            .games
            .iter()
            .find(|g| Some(&g.id) == self.game_id.as_ref())
        else {
            return Vec::new();
        };
        game.events
            .iter()
            .filter_map(|ev| {
                let count = *self.triggers.get(&(game.id.clone(), ev.id.clone()))?;
                (count > 0).then(|| EventCount {
                    event: ev.name.clone(),
                    count,
                })
            })
            .collect()
    }

    pub fn jokers(&self) -> Jokers {
        Jokers {
            earned: self.jokers_earned,
            spent: self.jokers_spent,
        }
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
            intensity: self.intensity,
            pending: self.pending.clone(),
            sets: self.done.len(),
            totals: totals(&self.done),
            log: self.done.iter().rev().cloned().collect(),
            triggers,
            jokers: self.jokers,
            jokers_earned: self.jokers_earned,
            jokers_spent: self.jokers_spent,
        }
    }
}

const NOT_ENOUGH_JOKERS: &str = "not_enough_jokers";

fn find_event<'a>(
    config: &'a Config,
    game_id: &str,
    event_id: &str,
) -> Option<(&'a Game, &'a GameEvent)> {
    let game = config.games.iter().find(|g| g.id == game_id)?;
    Some((game, game.events.iter().find(|e| e.id == event_id)?))
}

/// Jokers to skip everything left of a set.
pub fn skip_set_cost(draw: &Draw) -> u32 {
    draw.items
        .iter()
        .filter(|it| !it.done)
        .map(|it| it.skip_cost)
        .sum()
}

/// Sums sets by exercise, in order of first appearance.
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
