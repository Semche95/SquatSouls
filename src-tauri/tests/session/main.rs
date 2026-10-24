#[path = "../catalog_config/mod.rs"]
mod catalog_config;
mod draw;
mod intensity;
mod jokers;
mod limit;
mod progress;
mod refit;
mod summary;

use catalog_config::{catalog_config, event_id, game_id};
use rand::rngs::StdRng;
use rand::SeedableRng;
use squat_souls_lib::app::*;

/// Version of the UUID `id`, none if it is not one.
fn uuid_version(id: &str) -> usize {
    uuid::Uuid::parse_str(id).map_or(0, |u| u.get_version_num())
}

fn rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

/// Marks done, one by one, the remaining exercises of a set.
fn complete(s: &mut Session, id: &str, now: i64) -> Result<Vec<LogEntry>, AppError> {
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
    let cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = rng();
    let nohit = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated without getting hit"),
            0,
            &mut r,
        )
        .unwrap();
    assert_eq!(nohit.items.len(), 2, "the reward comes with a set");
    assert_eq!(nohit.jokers, 2);
    assert_eq!(
        s.skip_set(&nohit.id).unwrap_err(),
        AppError::new("not_enough_jokers"),
        "no joker at first"
    );
    assert_eq!(complete(&mut s, &nohit.id, 10).unwrap().len(), 2);
    assert_eq!(s.view().jokers, 2, "earned on completion");
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    (s, boss)
}

/// Same session, whose boss set is dismissed to leave room for the set of the event named `name` in English.
fn session_with_jokers_on(name: &str) -> (Session, Draw) {
    let (mut s, boss) = session_with_jokers();
    s.dismiss(&boss.id).unwrap();
    let cfg = catalog_config(Language::Fr);
    let draw = s
        .draw(&cfg, &game_id("ds3"), &event_id("ds3", name), 0, &mut rng())
        .unwrap();
    (s, draw)
}

/// Event of the catalog, to modify.
fn event_mut<'a>(cfg: &'a mut Config, game: &str, event: &str) -> &'a mut GameEvent {
    cfg.games
        .iter_mut()
        .find(|g| g.id == game)
        .and_then(|g| g.events.iter_mut().find(|e| e.id == event))
        .unwrap()
}

fn pending(s: &Session, id: &str) -> Option<Draw> {
    s.view().pending.into_iter().find(|d| d.id == id)
}

#[test]
fn session_waits_to_be_started() {
    let cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    assert_eq!(s.view().started_at, None);
    let id = s
        .draw(
            &cfg,
            &game_id("ds1"),
            &event_id("ds1", "Death"),
            0,
            &mut rng(),
        )
        .unwrap()
        .id;
    assert_eq!(uuid_version(&id), 7, "each set has its own id");
    assert!(s.id.is_empty(), "no id before it starts");
    let entries = complete(&mut s, &id, 5).unwrap();
    assert_eq!(
        s.view().started_at,
        Some(5),
        "started by its first set done"
    );
    assert_eq!(uuid_version(&s.id), 7);
    assert!(entries.iter().all(|e| e.session == s.id));
    assert_eq!(
        s.start(&cfg, &game_id("ds1"), 9).unwrap(),
        5,
        "a started session keeps its start"
    );
}

#[test]
fn nothing_starts_without_an_included_exercise() {
    let mut cfg = catalog_config(Language::En);
    cfg.exercises.iter_mut().for_each(|ex| ex.enabled = false);
    let (game, event) = (cfg.games[0].id.clone(), cfg.games[0].events[0].id.clone());
    let refused = AppError::new("start_without_exercise");
    let mut s = Session::default();
    assert_eq!(s.start(&cfg, &game, 5).unwrap_err(), refused);
    assert_eq!(
        s.draw(&cfg, &game, &event, 0, &mut rng()).unwrap_err(),
        refused
    );
    cfg.exercises.clear();
    assert_eq!(
        s.draw(&cfg, &game, &event, 0, &mut rng()).unwrap_err(),
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
    let mut cfg = catalog_config(Language::En);
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
    let cfg = catalog_config(Language::En);
    assert_eq!(s.start(&cfg, &game, 5).unwrap(), 5);
    assert_eq!(s.view().game_id.as_deref(), Some(game.as_str()));
    let other = &cfg.games[1];
    assert_eq!(
        s.start(&cfg, &other.id, 9).unwrap_err().code,
        "session_other_game"
    );
    let event = &other.events[0].id;
    assert_eq!(
        s.draw(&cfg, &other.id, event, 0, &mut rng())
            .unwrap_err()
            .code,
        "session_other_game",
        "locked from the start, before any event"
    );
}

#[test]
fn a_session_ends_with_a_set_done_and_none_to_do() {
    let cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    assert_eq!(s.check_end().unwrap_err().code, "end_without_set");
    let death = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Death"),
            0,
            &mut rng(),
        )
        .unwrap();
    assert_eq!(s.check_end().unwrap_err().code, "end_with_pending_sets");
    complete(&mut s, &death.id, 10).unwrap();
    s.check_end().unwrap();
}
