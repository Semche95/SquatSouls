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
    cfg.exercises[1].enabled = false;
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

fn connect(path: &std::path::Path) -> rusqlite::Connection {
    rusqlite::Connection::open(path).unwrap()
}

/// Versions et dates des migrations inscrites à l'historique de la base.
fn applied(path: &std::path::Path) -> Vec<(i64, String)> {
    connect(path)
        .prepare("SELECT version, applied_on FROM refinery_schema_history ORDER BY version")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn applied_migrations_are_dated() {
    let path = temp_db("dated");
    Store::open(&path).unwrap();
    Store::open(&path).unwrap();
    let applied = applied(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        applied.iter().map(|a| a.0).collect::<Vec<_>>(),
        [1, 2, 3, 4, 5],
        "une migration n'est appliquée qu'une fois"
    );
    assert!(
        applied
            .iter()
            .all(|a| chrono::DateTime::parse_from_rfc3339(&a.1).is_ok()),
        "date et heure"
    );
}

/// Base migrée avant refinery : ses `version` premières migrations jouées, notées dans `user_version`.
fn legacy_db(name: &str, version: usize) -> std::path::PathBuf {
    let path = temp_db(name);
    let conn = connect(&path);
    let migrations = [
        include_str!("../migrations/V1__schema.sql"),
        include_str!("../migrations/V2__steam_detection.sql"),
        include_str!("../migrations/V3__steam_names.sql"),
    ];
    for sql in &migrations[..version] {
        conn.execute_batch(sql).unwrap();
    }
    conn.pragma_update(None, "user_version", version as i64)
        .unwrap();
    conn.execute(
        "INSERT INTO sets (at, session, game, event, exercise, unit, amount)
         VALUES (1000, 1000, 'Sekiro', 'Mort', 'Squats', 'rép.', 12)",
        [],
    )
    .unwrap();
    path
}

#[test]
fn database_migrated_before_refinery_keeps_its_data() {
    for (name, version) in [("legacy-current", 3), ("legacy-behind", 2)] {
        let path = legacy_db(name, version);
        let history = Store::open(&path).unwrap().full_history().unwrap();
        Store::open(&path).unwrap().steam_names().unwrap();
        let applied = applied(&path);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(history.len(), 1, "{name}");
        assert_eq!(applied.len(), 5, "{name}");
    }
}

#[test]
fn steam_names_receive_retry_at_even_from_draft_schema() {
    for (name, draft) in [("names-sound", false), ("names-draft", true)] {
        let path = legacy_db(name, 3);
        let conn = connect(&path);
        if draft {
            conn.execute_batch(
                "DROP TABLE steam_names;
                 CREATE TABLE steam_names (app_id INTEGER PRIMARY KEY, name TEXT);",
            )
            .unwrap();
        }
        conn.execute_batch(if draft {
            "INSERT INTO steam_names (app_id, name) VALUES (814380, 'Sekiro'), (1, NULL)"
        } else {
            "INSERT INTO steam_names (app_id, name, retry_at) VALUES (814380, 'Sekiro', NULL), (1, NULL, 5000)"
        })
        .unwrap();
        let mut names = Store::open(&path).unwrap().steam_names().unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            names.name(814380, 1000, |_| None, |_| ()).unwrap(),
            "Sekiro",
            "{name}"
        );
        if !draft {
            assert_eq!(
                names.name(1, 2000, |_| panic!("redemandé avant sa date"), |_| ()),
                Err(SteamNameError::Unknown),
                "date de nouvel essai gardée"
            );
        }
    }
}

#[test]
fn steam_and_tray_settings_are_saved() {
    let mut s = store();
    let mut cfg = default_config();
    assert!(!cfg.steam_detection && cfg.steam_notify);
    assert!(!cfg.tray && !cfg.close_to_tray && !cfg.start_hidden);
    cfg.steam_detection = true;
    cfg.steam_notify = false;
    cfg.tray = true;
    cfg.close_to_tray = true;
    cfg.start_hidden = true;
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
fn steam_names_are_kept() {
    let mut s = store();
    assert_eq!(s.steam_names().unwrap(), SteamNames::default());
    let mut names = SteamNames::default();
    let json = r#"{"814380":{"data":{"name":"Sekiro"}}}"#;
    names
        .name(814380, 1000, |_| Some((200, json.into())), |_| ())
        .unwrap();
    names.name(1, 2000, |_| None, |_| ()).unwrap_err();
    names
        .name(2, 3000, |_| Some((429, String::new())), |_| ())
        .unwrap_err();
    s.save_steam_names(&names).unwrap();
    assert_eq!(s.steam_names().unwrap(), names);
}

#[test]
fn newer_database_is_refused() {
    let path = temp_db("newer");
    Store::open(&path).unwrap();
    connect(&path)
        .execute(
            "INSERT INTO refinery_schema_history (version, name, applied_on, checksum)
             VALUES (999, 'future', '2100-01-01T00:00:00Z', '0')",
            [],
        )
        .unwrap();
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
