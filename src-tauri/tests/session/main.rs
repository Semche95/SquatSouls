mod draw;
mod intensity;
mod jokers;
mod progress;
mod refit;
mod summary;

use rand::rngs::StdRng;
use rand::SeedableRng;
use squat_souls_lib::app::*;

fn rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

/// Marks done, one by one, the remaining exercises of a set.
fn complete(s: &mut Session, id: u64, now: i64) -> Result<Vec<LogEntry>, AppError> {
    let draw = s
        .view()
        .pending
        .into_iter()
        .find(|d| d.id == id)
        .ok_or(AppError::new("set_not_pending"))?;
    draw.items
        .iter()
        .filter(|it| !it.done)
        .map(|it| s.complete_exercise(id, &it.exercise_id, now))
        .collect()
}

/// Session where a completed "boss no hit" earned 2 jokers, with a two-exercise boss set pending.
fn session_with_jokers() -> (Session, Draw) {
    let cfg = default_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let nohit = s.draw(&cfg, "ds3", "boss-nohit", &mut r).unwrap();
    assert_eq!(nohit.items.len(), 2, "the reward comes with a set");
    assert_eq!(nohit.jokers, 2);
    assert_eq!(
        s.skip_set(nohit.id).unwrap_err(),
        AppError::new("not_enough_jokers"),
        "no joker at first"
    );
    assert_eq!(complete(&mut s, nohit.id, 10).unwrap().len(), 2);
    assert_eq!(s.view().jokers, 2, "earned on completion");
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    (s, boss)
}

/// Same session, whose boss set is dismissed to leave room for the set `event`.
fn session_with_jokers_on(event: &str) -> (Session, Draw) {
    let (mut s, boss) = session_with_jokers();
    s.dismiss(boss.id).unwrap();
    let cfg = default_config(Language::Fr);
    let draw = s.draw(&cfg, "ds3", event, &mut rng()).unwrap();
    (s, draw)
}

/// Event of the default configuration, to modify.
fn event_mut<'a>(cfg: &'a mut Config, game: &str, event: &str) -> &'a mut GameEvent {
    cfg.games
        .iter_mut()
        .find(|g| g.id == game)
        .and_then(|g| g.events.iter_mut().find(|e| e.id == event))
        .unwrap()
}

fn pending(s: &Session, id: u64) -> Option<Draw> {
    s.view().pending.into_iter().find(|d| d.id == id)
}

#[test]
fn session_waits_to_be_started() {
    let cfg = default_config(Language::Fr);
    let mut s = Session::default();
    assert_eq!(s.view().started_at, None);
    let id = s.draw(&cfg, "ds1", "mort", &mut rng()).unwrap().id;
    let entries = complete(&mut s, id, 5).unwrap();
    assert_eq!(
        s.view().started_at,
        Some(5),
        "started by its first set done"
    );
    assert!(entries.iter().all(|e| e.session == 5));
    assert_eq!(
        s.start(&cfg, "ds1", 9).unwrap(),
        5,
        "a started session keeps its start"
    );
}

#[test]
fn nothing_starts_without_an_included_exercise() {
    let mut cfg = default_config(Language::En);
    cfg.exercises.iter_mut().for_each(|ex| ex.enabled = false);
    let (game, event) = (cfg.games[0].id.clone(), cfg.games[0].events[0].id.clone());
    let refused = AppError::new("start_without_exercise");
    let mut s = Session::default();
    assert_eq!(s.start(&cfg, &game, 5).unwrap_err(), refused);
    assert_eq!(
        s.draw(&cfg, &game, &event, &mut rng()).unwrap_err(),
        refused
    );
    cfg.exercises.clear();
    assert_eq!(
        s.draw(&cfg, &game, &event, &mut rng()).unwrap_err(),
        refused
    );
    let view = s.view();
    assert_eq!(view.started_at, None, "refused, not started");
    assert!(
        view.pending.is_empty() && view.triggers.is_empty(),
        "nothing drawn nor counted"
    );
}

#[test]
fn a_session_starts_on_a_game_with_an_event_then_keeps_it() {
    let mut cfg = default_config(Language::En);
    let mut s = Session::default();
    assert_eq!(
        s.start(&cfg, "nioh", 5).unwrap_err(),
        AppError::new("unknown_game").with("id", "nioh")
    );
    let (game, name) = (cfg.games[0].id.clone(), cfg.games[0].name.clone());
    cfg.games[0].events.clear();
    assert_eq!(
        s.start(&cfg, &game, 5).unwrap_err(),
        AppError::new("start_without_event").with("game", name.as_str())
    );
    assert_eq!(s.view().started_at, None, "refused, not started");
    let cfg = default_config(Language::En);
    assert_eq!(s.start(&cfg, &game, 5).unwrap(), 5);
    assert_eq!(s.view().game_id.as_deref(), Some(game.as_str()));
    let other = &cfg.games[1];
    assert_eq!(
        s.start(&cfg, &other.id, 9).unwrap_err().code,
        "session_other_game"
    );
    let event = &other.events[0].id;
    assert_eq!(
        s.draw(&cfg, &other.id, event, &mut rng()).unwrap_err().code,
        "session_other_game",
        "locked from the start, before any event"
    );
}

#[test]
fn a_session_ends_with_a_set_done_and_none_to_do() {
    let cfg = default_config(Language::Fr);
    let mut s = Session::default();
    assert_eq!(s.check_end().unwrap_err().code, "end_without_set");
    let death = s.draw(&cfg, "ds3", "mort", &mut rng()).unwrap();
    assert_eq!(s.check_end().unwrap_err().code, "end_with_pending_sets");
    complete(&mut s, death.id, 10).unwrap();
    s.check_end().unwrap();
}
