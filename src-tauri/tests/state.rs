use chrono::Local;
use rand::rngs::StdRng;
use rand::SeedableRng;
use squat_souls_lib::app::*;
use squat_souls_lib::state::AppState;

/// State on a new database, with the French default configuration.
fn state() -> AppState {
    AppState::new(Store::open_in_memory(Some("fr_FR.UTF-8")).unwrap(), None)
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

/// Sessions of the history over the last two days.
fn sessions(state: &AppState) -> Vec<SessionTotals> {
    let today = Local::now().date_naive();
    let days = state.store.history(2, today).unwrap();
    days.into_iter().flat_map(|d| d.sessions).collect()
}

#[test]
fn removing_the_sessions_game_ends_it_with_the_exercises_done() {
    let mut state = state();
    let cfg = state.config.clone();
    let mut r = StdRng::seed_from_u64(42);
    let start = Local::now().timestamp() - 100;
    let boss = state.session.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    state.session.draw(&cfg, "ds3", "mort", &mut r).unwrap();
    let entry = state
        .session
        .complete_exercise(boss.id, &boss.items[0].exercise_id, start)
        .unwrap();
    state.store.append_history(&[entry]).unwrap();
    state.save_session_summary();

    state.store_config(without_game(&state, "ds3")).unwrap();
    assert_eq!(state.session.started_at, None, "the next session waits");
    assert_eq!(
        state.session.view().ended,
        Some(ended(true, EndCause::GameRemoved))
    );
    let cfg = state.config.clone();
    state.session.start(&cfg, "ds1", start + 200).unwrap();
    assert_eq!(
        state.session.view().ended,
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
    let cfg = state.config.clone();
    state
        .session
        .start(&cfg, "ds3", Local::now().timestamp())
        .unwrap();
    state
        .session
        .draw(&cfg, "ds3", "mort", &mut StdRng::seed_from_u64(42))
        .unwrap();
    state.save_session_summary();

    state.store_config(without_game(&state, "ds3")).unwrap();
    assert_eq!(state.session.started_at, None);
    assert_eq!(
        state.session.view().ended,
        Some(ended(false, EndCause::GameRemoved))
    );
    assert!(sessions(&state).is_empty(), "nothing saved");
}

#[test]
fn removing_another_game_keeps_the_session() {
    let mut state = state();
    let cfg = state.config.clone();
    let start = state
        .session
        .start(&cfg, "ds3", Local::now().timestamp())
        .unwrap();
    state.store_config(without_game(&state, "ds1")).unwrap();
    assert_eq!(state.session.started_at, Some(start));
}
