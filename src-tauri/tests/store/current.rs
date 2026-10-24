use super::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

/// Session on "ds3" started at `start`: a reward set done earning 2 jokers, a boss set started with its first
/// exercise halved then done, and a death set dismissed before being started, kept in reserve; with the
/// exercises done.
fn played(start: i64) -> (Session, Vec<LogEntry>) {
    let cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    let mut r = StdRng::seed_from_u64(42);
    s.start(&cfg, &game_id("ds3"), start).unwrap();
    s.set_intensity(Intensity::Hard);
    let mut done = Vec::new();
    let nohit = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated without getting hit"),
            0,
            &mut r,
        )
        .unwrap();
    for it in &nohit.items {
        done.push(
            s.complete_exercise(&nohit.id, &it.exercise_id, start + 10)
                .unwrap(),
        );
    }
    let boss = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            0,
            &mut r,
        )
        .unwrap();
    s.halve(&boss.id, &boss.items[0].exercise_id).unwrap();
    done.push(
        s.complete_exercise(&boss.id, &boss.items[0].exercise_id, start + 20)
            .unwrap(),
    );
    let death = s
        .draw(&cfg, &game_id("ds3"), &event_id("ds3", "Death"), 0, &mut r)
        .unwrap();
    s.dismiss(&death.id).unwrap();
    (s, done)
}

/// Saves the session as the app does, which records it, with its exercises done in the history.
fn save(store: &mut Store, s: &Session, done: &[LogEntry], seen: i64) {
    store.save_current_session(s, seen).unwrap();
    store.append_history(done).unwrap();
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
    // Round, last set and reserve carry on as if the app had never closed; each set receives its own id.
    let cfg = catalog_config(Language::Fr);
    for name in ["Death", "Boss defeated", "Death"] {
        let (game, event) = (game_id("ds3"), event_id("ds3", name));
        let mut r = StdRng::seed_from_u64(7);
        let (resumed, live) = (
            read.draw(&cfg, &game, &event, 0, &mut r).unwrap(),
            s.draw(&cfg, &game, &event, 0, &mut StdRng::seed_from_u64(7))
                .unwrap(),
        );
        assert_eq!(
            Draw {
                id: live.id.clone(),
                ..resumed
            },
            live
        );
    }
}

#[test]
fn the_limit_of_the_session_in_progress_is_read_back() {
    let mut store = store();
    let mut cfg = catalog_config(Language::Fr);
    // 40 s each, so that 3 exercises fit in 2 minutes: the second set is reduced, and the limit reached.
    cfg.recovery = 10;
    for ex in &mut cfg.exercises {
        (ex.min, ex.max, ex.recovery, ex.timed) = (10, 10, None, None);
    }
    let limit = SessionLimit {
        kind: LimitKind::TrainingTime,
        value: 2,
    };
    let mut s = Session::default();
    let mut r = StdRng::seed_from_u64(42);
    s.set_limit(Some(limit)).unwrap();
    s.start(&cfg, &game_id("ds3"), 1000).unwrap();
    s.draw(
        &cfg,
        &game_id("ds3"),
        &event_id("ds3", "Boss defeated"),
        1000,
        &mut r,
    )
    .unwrap();
    assert_eq!(
        s.draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            1000,
            &mut r
        )
        .unwrap()
        .items
        .len(),
        1
    );
    save(&mut store, &s, &[], 1500);

    let (mut read, _) = store.current_session().unwrap().unwrap();
    assert_eq!(format!("{:?}", read.view()), format!("{:?}", s.view()));
    assert_eq!(read.view().limit, Some(limit));
    read.apply_config(&cfg, &mut r);
    s.apply_config(&cfg, &mut r);
    assert_eq!(format!("{:?}", read.view()), format!("{:?}", s.view()));
}

/// Saves the session in a database as one saved before the exercises asked were counted, and reads it back.
fn saved_before_counting(name: &str, s: &Session, done: &[LogEntry]) -> Session {
    let path = temp_db(name);
    save(&mut Store::open(&path, LOCALE).unwrap(), s, done, 1500);
    {
        let conn = connect(&path);
        conn.execute(
            "UPDATE current_session SET asked = 0, asked_time = 0, asked_known = 0",
            [],
        )
        .unwrap();
        conn.execute("UPDATE current_sets SET time = 0", [])
            .unwrap();
    }
    let read = Store::open(&path, LOCALE)
        .unwrap()
        .current_session()
        .unwrap()
        .unwrap()
        .0;
    std::fs::remove_file(&path).unwrap();
    read
}

#[test]
fn a_set_saved_before_keeping_what_it_asked_frees_its_exercises_drawn() {
    let cfg = catalog_config(Language::Fr);
    let mut s = Session::default();
    s.start(&cfg, &game_id("ds3"), 1000).unwrap();
    s.draw(
        &cfg,
        &game_id("ds3"),
        &event_id("ds3", "Boss defeated"),
        1000,
        &mut StdRng::seed_from_u64(42),
    )
    .unwrap();
    let path = temp_db("set-asked");
    save(&mut Store::open(&path, LOCALE).unwrap(), &s, &[], 1500);
    connect(&path)
        .execute("UPDATE current_sets SET asked = NULL", [])
        .unwrap();
    let mut read = Store::open(&path, LOCALE)
        .unwrap()
        .current_session()
        .unwrap()
        .unwrap()
        .0;
    std::fs::remove_file(&path).unwrap();
    assert_eq!(read.view().asked, 2);
    let id = read.view().pending[0].id.clone();
    read.dismiss(&id).unwrap();
    assert_eq!(read.view().asked, 0);
}

#[test]
fn a_session_saved_before_counting_what_it_asked_counts_it_again() {
    let cfg = catalog_config(Language::Fr);
    let (s, done) = played(1000);
    let mut read = saved_before_counting("recount", &s, &done);
    assert_eq!(read.view().asked, 0);
    read.recount_asked(&cfg);
    assert_eq!(read.view().asked, s.view().asked);
    assert_eq!(read.view().asked_time, s.view().asked_time);
    // Its sets to do free their time again once dismissed.
    let id = read.view().pending[0].id.clone();
    let mut live = s;
    read.dismiss(&id).unwrap();
    live.dismiss(&id).unwrap();
    assert_eq!(read.view().asked_time, live.view().asked_time);
    // Counted once only.
    read.recount_asked(&cfg);
    assert_eq!(read.view().asked_time, live.view().asked_time);
}

#[test]
fn a_stopped_set_saved_before_counting_still_counts_its_exercises() {
    let cfg = catalog_config(Language::Fr);
    let (mut s, mut done) = played(1000);
    let boss = s.view().pending[0].clone();
    let rest = boss.items.iter().find(|it| !it.done).unwrap();
    done.push(
        s.complete_exercise(&boss.id, &rest.exercise_id, 1030)
            .unwrap(),
    );
    let stopped = s
        .draw(
            &cfg,
            &game_id("ds3"),
            &event_id("ds3", "Boss defeated"),
            1040,
            &mut StdRng::seed_from_u64(1),
        )
        .unwrap();
    done.push(
        s.complete_exercise(&stopped.id, &stopped.items[0].exercise_id, 1050)
            .unwrap(),
    );
    s.dismiss(&stopped.id).unwrap();
    let mut read = saved_before_counting("recount-stopped", &s, &done);
    read.recount_asked(&cfg);
    assert_eq!(
        read.view().asked,
        s.view().asked,
        "the exercise not done counts"
    );
    assert!(read.view().asked_time > 0);
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
        .complete_exercise(&boss.id, &next.exercise_id, 5030)
        .unwrap();
    save(&mut store, &s, &[entry], 5030);
    assert_eq!(today(&store), 90, "then from the reopening");

    store.end_session(&s.id, 5100).unwrap();
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
