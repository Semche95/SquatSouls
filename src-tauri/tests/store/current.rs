use super::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

/// Session on "ds3" started at `start`: a reward set done earning 2 jokers, a boss set started with its first
/// exercise halved then done, and a death set dismissed before being started, kept in reserve; with the
/// exercises done.
fn played(start: i64) -> (Session, Vec<LogEntry>) {
    let cfg = default_config(Language::Fr);
    let mut s = Session::default();
    let mut r = StdRng::seed_from_u64(42);
    s.start(&cfg, "ds3", start).unwrap();
    s.set_intensity(Intensity::Hard);
    let mut done = Vec::new();
    let nohit = s.draw(&cfg, "ds3", "boss-nohit", &mut r).unwrap();
    for it in &nohit.items {
        done.push(
            s.complete_exercise(nohit.id, &it.exercise_id, start + 10)
                .unwrap(),
        );
    }
    let boss = s.draw(&cfg, "ds3", "boss", &mut r).unwrap();
    s.halve(boss.id, &boss.items[0].exercise_id).unwrap();
    done.push(
        s.complete_exercise(boss.id, &boss.items[0].exercise_id, start + 20)
            .unwrap(),
    );
    let death = s.draw(&cfg, "ds3", "mort", &mut r).unwrap();
    s.dismiss(death.id).unwrap();
    (s, done)
}

/// Saves the session as the app does, with its exercises done in the history.
fn save(store: &mut Store, s: &Session, done: &[LogEntry], seen: i64) {
    store.append_history(done).unwrap();
    store.save_current_session(s, seen).unwrap();
}

#[test]
fn the_session_in_progress_is_read_back_as_it_was_saved() {
    let mut store = store();
    let (mut s, done) = played(1000);
    save(&mut store, &s, &done, 1500);

    let (mut read, seen) = store
        .current_session()
        .unwrap()
        .expect("a session in progress");
    assert_eq!(seen, 1500);
    assert_eq!(format!("{:?}", read.view()), format!("{:?}", s.view()));
    // Round, last set, reserve and set ids carry on as if the app had never closed.
    let cfg = default_config(Language::Fr);
    for event in ["mort", "boss", "mort"] {
        let mut r = StdRng::seed_from_u64(7);
        assert_eq!(
            read.draw(&cfg, "ds3", event, &mut r).unwrap(),
            s.draw(&cfg, "ds3", event, &mut StdRng::seed_from_u64(7))
                .unwrap()
        );
    }
}

#[test]
fn no_session_in_progress_without_one_started() {
    let mut store = store();
    store
        .save_current_session(&Session::default(), 1000)
        .unwrap();
    assert!(store.current_session().unwrap().is_none());
}

#[test]
fn an_ended_or_cancelled_session_is_no_longer_in_progress() {
    let mut store = store();
    let (s, done) = played(1000);
    save(&mut store, &s, &done, 1500);
    store.clear_current_session().unwrap();
    assert!(store.current_session().unwrap().is_none());
}

#[test]
fn the_app_is_marked_open_during_the_session() {
    let mut store = store();
    let (s, done) = played(1000);
    save(&mut store, &s, &done, 1500);
    store.mark_seen(1800).unwrap();
    assert_eq!(store.current_session().unwrap().unwrap().1, 1800);
}

#[test]
fn the_duration_leaves_out_the_time_the_app_was_closed() {
    let mut store = store();
    let (mut s, done) = played(1000);
    save(&mut store, &s, &done, 1060);
    let today = |store: &Store| {
        let days = store.full_history(None, 1).unwrap();
        days[0].sessions[0].duration
    };
    assert_eq!(today(&store), 20, "until the last set while in progress");

    let (mut read, closed) = store.current_session().unwrap().unwrap();
    read.resume(closed, 5000);
    store.save_current_session(&read, 5000).unwrap();
    assert_eq!(today(&store), 60, "until the app closed");

    s = read;
    let boss = s.view().pending.remove(0);
    let next = boss.items.iter().find(|it| !it.done).unwrap();
    let entry = s
        .complete_exercise(boss.id, &next.exercise_id, 5030)
        .unwrap();
    save(&mut store, &s, &[entry], 5030);
    assert_eq!(today(&store), 90, "then from the reopening");

    store.end_session(1000, 5100).unwrap();
    assert_eq!(today(&store), 160, "until its end");
}

#[test]
fn an_atomic_change_keeps_nothing_on_failure() {
    let mut store = store();
    let (s, done) = played(1000);
    let failed: rusqlite::Result<()> = store.atomically(|store| {
        save(store, &s, &done, 1500);
        Err(rusqlite::Error::InvalidQuery)
    });
    assert!(failed.is_err());
    assert!(store.current_session().unwrap().is_none());
    assert!(store.full_history(None, 1).unwrap().is_empty());
}

/// Store holding the session `played(1000)`, the app last seen open at 1500, its row then broken by `breaking`.
fn unreadable(name: &str, breaking: &str) -> (Store, std::path::PathBuf) {
    let path = temp_db(name);
    let mut store = Store::open(&path, LOCALE).unwrap();
    let (s, done) = played(1000);
    save(&mut store, &s, &done, 1500);
    drop(store);
    let conn = connect(&path);
    conn.execute(breaking, []).unwrap();
    drop(conn);
    (Store::open(&path, LOCALE).unwrap(), path)
}

#[test]
fn an_unreadable_session_ends_when_the_app_was_last_seen_open() {
    let (mut store, path) = unreadable(
        "current-unreadable-ended",
        "UPDATE current_session SET intensity = 'extreme'",
    );
    assert!(store.current_session().is_err());
    store.end_unreadable_session().unwrap();
    assert!(store.current_session().unwrap().is_none(), "forgotten");
    let session = &store.full_history(None, 1).unwrap()[0].sessions[0];
    assert_eq!(session.sets, 3, "its exercises done stay");
    assert_eq!(session.duration, 500, "until the app was last seen open");
    drop(store);
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn an_unreadable_session_without_its_times_is_only_forgotten() {
    let (mut store, path) = unreadable(
        "current-unreadable-forgotten",
        "UPDATE current_session SET seen = 'never'",
    );
    store.end_unreadable_session().unwrap();
    assert!(store.current_session().unwrap().is_none());
    let session = &store.full_history(None, 1).unwrap()[0].sessions[0];
    assert_eq!(session.duration, 20, "until its last set");
    drop(store);
    std::fs::remove_file(&path).unwrap();
}
