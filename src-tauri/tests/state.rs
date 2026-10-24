use chrono::Local;
use rand::rngs::StdRng;
use rand::SeedableRng;
use squat_souls_lib::app::*;
use squat_souls_lib::state::AppState;

/// State on `store` holding the French seed, games included: a new database has none.
fn seeded(mut store: Store) -> AppState {
    store.save_config(&seed_config(Language::Fr)).unwrap();
    AppState::new(store, None)
}

/// State on a new database, with the French seed.
fn state() -> AppState {
    seeded(Store::open_in_memory(Some("fr_FR.UTF-8")).unwrap())
}

/// Configuration of `state` without the game `id`.
fn without_game(state: &AppState, id: &str) -> Config {
    let mut cfg = state.config.clone();
    cfg.games.retain(|g| g.id != id);
    cfg
}

/// End of a session that ended by itself for `cause`, saved or not.
fn ended(saved: bool, cause: EndCause) -> SessionEnd {
    SessionEnd {
        saved,
        cause: Some(cause),
    }
}

/// Starts the session on "ds3" now.
fn start(state: &mut AppState) -> i64 {
    state
        .change_session(|s, cfg| s.start(cfg, "ds3", Local::now().timestamp()))
        .unwrap()
}

/// Marks done at `at` the first exercise of `set`.
fn complete_first(state: &mut AppState, set: &Draw, at: i64) {
    state
        .change_session(|s, _| s.complete_exercise(set.id, &set.items[0].exercise_id, at))
        .unwrap();
}

/// The app quit then launched again on the same database.
fn relaunch(state: AppState) -> AppState {
    AppState::new(state.store, None)
}

/// Sessions of the history over the last two days.
fn sessions(state: &AppState) -> Vec<SessionTotals> {
    let today = Local::now().date_naive();
    let days = state.store.history(2, today).unwrap();
    days.into_iter().flat_map(|d| d.sessions).collect()
}

#[test]
fn removing_the_sessions_game_ends_it_with_the_exercises_done() {
    let mut state = state();
    let mut r = StdRng::seed_from_u64(42);
    let start = Local::now().timestamp() - 100;
    let boss = state
        .change_session(|s, cfg| s.draw(cfg, "ds3", "boss", 0, &mut r))
        .unwrap();
    state
        .change_session(|s, cfg| s.draw(cfg, "ds3", "mort", 0, &mut r))
        .unwrap();
    complete_first(&mut state, &boss, start);

    state.store_config(without_game(&state, "ds3")).unwrap();
    assert_eq!(state.session().started_at, None, "the next session waits");
    assert_eq!(
        state.session().view().ended,
        Some(ended(true, EndCause::GameRemoved))
    );
    state
        .change_session(|s, cfg| s.start(cfg, "ds1", start + 200))
        .unwrap();
    assert_eq!(
        state.session().view().ended,
        None,
        "told until the next session starts"
    );
    let [session] = &sessions(&state)[..] else {
        panic!("one session in the history");
    };
    assert_eq!(session.sets, 1, "only the exercise done");
    assert!(session.duration >= 100, "ended at the removal");
    assert_eq!(
        session.events,
        vec![EventCount {
            event: "Boss vaincu".into(),
            count: 1
        }],
        "the set not started is no longer counted"
    );
    assert_eq!(
        session.jokers,
        Some(Jokers {
            earned: 0,
            spent: 0
        })
    );
}

#[test]
fn removing_the_sessions_game_without_any_exercise_done_cancels_it() {
    let mut state = state();
    start(&mut state);
    state
        .change_session(|s, cfg| s.draw(cfg, "ds3", "mort", 0, &mut StdRng::seed_from_u64(42)))
        .unwrap();

    state.store_config(without_game(&state, "ds3")).unwrap();
    assert_eq!(state.session().started_at, None);
    assert_eq!(
        state.session().view().ended,
        Some(ended(false, EndCause::GameRemoved))
    );
    assert!(sessions(&state).is_empty(), "nothing saved");
}

#[test]
fn switching_every_exercise_off_ends_the_session_with_the_exercises_done() {
    let mut state = state();
    let boss = state
        .change_session(|s, cfg| s.draw(cfg, "ds3", "boss", 0, &mut StdRng::seed_from_u64(42)))
        .unwrap();
    complete_first(&mut state, &boss, Local::now().timestamp() - 100);

    let mut off = state.config.clone();
    off.exercises.iter_mut().for_each(|ex| ex.enabled = false);
    state.store_config(off).unwrap();
    assert_eq!(state.session().started_at, None, "the next session waits");
    assert_eq!(
        state.session().view().ended,
        Some(ended(true, EndCause::NoExercise))
    );
    let [session] = &sessions(&state)[..] else {
        panic!("one session in the history");
    };
    assert_eq!(session.sets, 1, "only the exercise done");
}

#[test]
fn removing_every_exercise_without_any_exercise_done_cancels_the_session() {
    let mut state = state();
    start(&mut state);
    let mut none = state.config.clone();
    none.exercises.clear();
    state.store_config(none).unwrap();
    assert_eq!(state.session().started_at, None);
    assert_eq!(
        state.session().view().ended,
        Some(ended(false, EndCause::NoExercise))
    );
    assert!(sessions(&state).is_empty(), "nothing saved");
}

#[test]
fn removing_another_game_keeps_the_session() {
    let mut state = state();
    let start = start(&mut state);
    state.store_config(without_game(&state, "ds1")).unwrap();
    assert_eq!(state.session().started_at, Some(start));
}

#[test]
fn the_session_resumes_as_it_was_at_the_next_launch() {
    let mut state = state();
    let mut r = StdRng::seed_from_u64(42);
    let start = Local::now().timestamp() - 100;
    state
        .change_session(|s, cfg| {
            s.set_intensity(Intensity::Easy);
            s.start(cfg, "ds3", start)
        })
        .unwrap();
    let boss = state
        .change_session(|s, cfg| s.draw(cfg, "ds3", "boss", 0, &mut r))
        .unwrap();
    complete_first(&mut state, &boss, start + 10);
    state
        .change_session(|s, cfg| s.draw(cfg, "ds3", "mort", 0, &mut r))
        .unwrap();
    let before = state.session().view();

    let state = relaunch(state);
    let after = state.session().view();
    assert_eq!(after.started_at, Some(start));
    assert_eq!(after.game_id.as_deref(), Some("ds3"));
    assert_eq!(after.intensity, Intensity::Easy, "its own intensity");
    assert_eq!(after.pending, before.pending);
    assert_eq!(after.log, before.log);
    assert_eq!(after.triggers, before.triggers);
    assert_eq!(after.jokers, before.jokers);
    assert_eq!(after.ended, None);
    let [session] = &sessions(&state)[..] else {
        panic!("one session in the history");
    };
    assert_eq!(session.sets, 1, "the exercise done is saved once");
}

#[test]
fn an_ended_or_cancelled_session_does_not_resume() {
    let mut state = state();
    let boss = state
        .change_session(|s, cfg| s.draw(cfg, "ds3", "boss", 0, &mut StdRng::seed_from_u64(42)))
        .unwrap();
    for it in &boss.items {
        state
            .change_session(|s, _| {
                s.complete_exercise(boss.id, &it.exercise_id, Local::now().timestamp())
            })
            .unwrap();
    }
    state.end_session(None);
    let mut state = relaunch(state);
    assert_eq!(state.session().started_at, None);
    assert_eq!(
        sessions(&state).len(),
        1,
        "the ended session stays in the history"
    );

    start(&mut state);
    state.cancel_session().unwrap();
    let state = relaunch(state);
    assert_eq!(state.session().started_at, None);
}

#[test]
fn a_session_saved_before_counting_what_it_asked_counts_it_again_at_launch() {
    let path = temp_db("recount");
    let mut state = seeded(Store::open(&path, Some("fr_FR.UTF-8")).unwrap());
    let mut r = StdRng::seed_from_u64(42);
    start(&mut state);
    state
        .change_session(|s, cfg| s.draw(cfg, "ds3", "boss", 0, &mut r))
        .unwrap();
    let asked = state.session().view().asked;
    drop(state);
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute(
        "UPDATE current_session SET asked = 0, asked_time = 0, asked_known = 0",
        [],
    )
    .unwrap();
    drop(conn);
    let state = AppState::new(Store::open(&path, Some("fr_FR.UTF-8")).unwrap(), None);
    assert_eq!(state.session().view().asked, asked);
    let state = relaunch(state);
    assert_eq!(state.session().view().asked, asked, "saved once counted");
    drop(state);
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn an_unreadable_session_in_progress_is_told_at_the_next_launch() {
    let path = temp_db("unreadable");
    let mut state = seeded(Store::open(&path, Some("fr_FR.UTF-8")).unwrap());
    start(&mut state);
    drop(state);
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute("UPDATE current_session SET intensity = 'extreme'", [])
        .unwrap();
    drop(conn);
    let state = AppState::new(Store::open(&path, Some("fr_FR.UTF-8")).unwrap(), None);
    assert_eq!(state.session().started_at, None);
    assert_eq!(
        state.session().view().ended,
        Some(ended(false, EndCause::Unreadable))
    );
    let state = relaunch(state);
    assert_eq!(state.session().view().ended, None, "told only once");
    drop(state);
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn no_update_is_installed_during_a_session() {
    let mut state = state();
    assert_eq!(state.allow_update(), Ok(()), "waiting to be started");
    start(&mut state);
    assert_eq!(
        state.allow_update(),
        Err(AppError::new("update_during_session"))
    );
    let mut state = relaunch(state);
    assert!(state.allow_update().is_err(), "the resumed session too");
    state.cancel_session().unwrap();
    assert_eq!(state.allow_update(), Ok(()));
}

/// Launches again on the database at `path` after `breaking` made its configuration invalid: the default one is
/// used in its place.
fn relaunch_with_invalid_config(state: AppState, path: &std::path::Path) -> AppState {
    drop(state);
    let conn = rusqlite::Connection::open(path).unwrap();
    conn.execute(
        "UPDATE games SET name = (SELECT name FROM games WHERE id = 'ds1') WHERE id = 'ds3'",
        [],
    )
    .unwrap();
    drop(conn);
    let state = AppState::new(Store::open(path, Some("fr_FR.UTF-8")).unwrap(), None);
    assert_eq!(
        state.config,
        default_config(Language::En),
        "default configuration, in the system language"
    );
    state
}

/// Temporary database for the test `name`, deleted beforehand.
fn temp_db(name: &str) -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("squatsouls-state-{name}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

#[test]
fn a_session_resumed_under_the_default_configuration_ends() {
    let path = temp_db("invalid-config");
    let mut state = seeded(Store::open(&path, Some("fr_FR.UTF-8")).unwrap());
    let set = state
        .change_session(|s, cfg| s.draw(cfg, "ds3", "mort", 0, &mut StdRng::seed_from_u64(42)))
        .unwrap();
    complete_first(&mut state, &set, Local::now().timestamp());

    let state = relaunch_with_invalid_config(state, &path);
    assert_eq!(state.session().started_at, None);
    assert_eq!(
        state.session().view().ended,
        Some(ended(true, EndCause::GameRemoved))
    );
    assert_eq!(sessions(&state)[0].sets, 1, "its exercise done stays");
    drop(state);
    std::fs::remove_file(&path).unwrap();
}
