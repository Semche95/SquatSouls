use chrono::Local;
use squat_souls_lib::fitness::*;

fn store() -> Store {
    Store::open_in_memory().unwrap()
}

#[test]
fn new_store_holds_default_config() {
    assert_eq!(store().config().unwrap(), default_config());
}

#[test]
fn config_changes_are_saved() {
    let mut s = store();
    let mut cfg = default_config();
    cfg.exercises.swap(0, 1);
    cfg.exercises[0].max = 30;
    cfg.exercises.pop();
    cfg.games.remove(1);
    cfg.games[0].name = "Elden Ring Nightreign".into();
    cfg.games[0].events.remove(0);
    cfg.games[0].events[0].max_per_session = Some(3);
    cfg.games.push(Game {
        id: "lies-of-p".into(),
        name: "Lies of P".into(),
        steam_app_id: Some(1627720),
        events: vec![event("mort", "Mort", 2, None)],
    });
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
}

#[test]
fn pauses_are_saved() {
    let mut s = store();
    let mut cfg = default_config();
    assert_eq!(cfg.pause, DEFAULT_PAUSE);
    cfg.pause = 90;
    cfg.exercises[0].pause = Some(120);
    cfg.exercises[1].pause = Some(0);
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
    cfg.exercises[0].pause = None;
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
}

/// Chemin d'une base temporaire propre au test `name`, effacée au préalable.
fn temp_db(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("squatsouls-{name}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn set_user_version(path: &std::path::Path, version: i64) {
    rusqlite::Connection::open(path)
        .unwrap()
        .pragma_update(None, "user_version", version)
        .unwrap();
}

/// Ramène une base au schéma d'avant la détection Steam, numéroté `version` (0 : avant toute numérotation).
fn drop_steam_schema(path: &std::path::Path, version: i64) {
    rusqlite::Connection::open(path)
        .unwrap()
        .execute_batch(&format!(
            "ALTER TABLE games DROP COLUMN steam_app_id; DROP TABLE ignored_games;
             PRAGMA user_version = {version};"
        ))
        .unwrap();
}

#[test]
fn database_without_schema_version_keeps_its_data() {
    let path = temp_db("unversioned");
    let mut cfg = Store::open(&path).unwrap().config().unwrap();
    cfg.games.truncate(1);
    cfg.games[0].steam_app_id = None;
    Store::open(&path).unwrap().save_config(&cfg).unwrap();
    drop_steam_schema(&path, 0);
    let reopened = Store::open(&path).unwrap().config().unwrap();
    let version: i64 = rusqlite::Connection::open(&path)
        .unwrap()
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    std::fs::remove_file(&path).unwrap();
    assert_eq!(reopened, cfg);
    assert!(version > 0);
}

#[test]
fn database_before_steam_detection_receives_its_schema() {
    let path = temp_db("before-steam");
    let mut cfg = Store::open(&path).unwrap().config().unwrap();
    drop_steam_schema(&path, 1);
    let mut s = Store::open(&path).unwrap();
    let reopened = s.config().unwrap();
    cfg.games.iter_mut().for_each(|g| g.steam_app_id = None);
    cfg.steam_detection = true;
    cfg.ignored_games.push(IgnoredGame {
        app_id: 1627720,
        name: "Lies of P".into(),
    });
    s.save_config(&cfg).unwrap();
    let saved = s.config().unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(reopened.games.iter().all(|g| g.steam_app_id.is_none()));
    assert_eq!(saved, cfg);
}

#[test]
fn steam_settings_are_saved() {
    let mut s = store();
    let mut cfg = default_config();
    assert!(!cfg.steam_detection);
    cfg.steam_detection = true;
    cfg.games[0].steam_app_id = None;
    cfg.ignored_games = vec![
        IgnoredGame {
            app_id: 1627720,
            name: "Lies of P".into(),
        },
        IgnoredGame {
            app_id: 367520,
            name: "Hollow Knight".into(),
        },
    ];
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
    cfg.ignored_games.remove(0);
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
}

#[test]
fn newer_database_is_refused() {
    let path = temp_db("newer");
    Store::open(&path).unwrap();
    set_user_version(&path, i64::from(i32::MAX));
    let opened = Store::open(&path);
    std::fs::remove_file(&path).unwrap();
    assert!(opened.is_err());
}

#[test]
fn removed_game_takes_its_events_along() {
    let mut s = store();
    let mut cfg = default_config();
    let removed = cfg.games.remove(0);
    s.save_config(&cfg).unwrap();
    // Recréer un jeu du même id repart d'une liste d'événements vide.
    cfg.games.push(Game {
        events: vec![event("mort", "Mort", 1, None)],
        ..removed
    });
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
}

#[test]
fn history_and_sessions_are_stored() {
    let mut s = store();
    let now = Local::now();
    let today = now.date_naive();
    let t = now.timestamp();
    let entry = |at: i64, session: i64, ex: &str, amount| LogEntry {
        at,
        session,
        game: "Dark Souls".into(),
        event: "Mort".into(),
        exercise: ex.into(),
        unit: "rép.".into(),
        amount,
    };
    s.append_history(&[
        entry(t, t - 60, "Squats", 12),
        entry(t, t - 60, "Squats", 15),
        entry(t, t - 60, "Pompes", 10),
    ])
    .unwrap();
    s.append_history(&[entry(t - 86_400 * 2, t - 86_400 * 2, "Pompes", 8)])
        .unwrap();
    s.append_history(&[entry(t, t, "Squats", 5)]).unwrap();
    s.end_session(t - 60, t + 600).unwrap();

    let days = s.history(7, today).unwrap();
    assert_eq!(days[0].sets, 4);
    assert_eq!(days[2].sets, 1);
    let first = days[0].sessions.iter().find(|x| x.sets == 3).unwrap();
    assert_eq!(first.duration, 660, "jusqu'à la fin enregistrée");
    assert_eq!(
        first.totals[0],
        Total {
            exercise: "Squats".into(),
            unit: "rép.".into(),
            amount: 27,
            sets: 2
        }
    );

    s.remove_history(&[entry(t, t - 60, "Squats", 15)]).unwrap();
    assert_eq!(
        s.history(1, today).unwrap()[0].sets,
        3,
        "série annulée retirée"
    );
    s.append_history(&[entry(t, t - 60, "Squats", 15)]).unwrap();

    s.remove_session(t).unwrap();
    assert_eq!(
        s.history(1, today).unwrap()[0].sets,
        3,
        "dernière séance effacée"
    );
    s.remove_session(t - 60).unwrap();
    assert_eq!(s.full_history().unwrap().len(), 1);
}

#[test]
fn last_game_is_kept() {
    let s = store();
    assert_eq!(s.last_game().unwrap(), "");
    s.set_last_game("ds3").unwrap();
    s.set_last_game("sekiro").unwrap();
    assert_eq!(s.last_game().unwrap(), "sekiro");
}
