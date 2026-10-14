use chrono::Local;
use squat_souls_lib::app::*;

/// System locale of the tests: new databases receive the French default configuration.
const LOCALE: Option<&str> = Some("fr_FR.UTF-8");

fn store() -> Store {
    Store::open_in_memory(LOCALE).unwrap()
}

/// Steam Store language of the names asked in the tests.
const LANGUAGE: &str = "english";

#[test]
fn new_store_holds_default_config() {
    assert_eq!(store().config().unwrap(), default_config(Language::Fr));
}

#[test]
fn new_store_default_config_follows_the_system_language() {
    let english = Store::open_in_memory(Some("de-DE")).unwrap();
    assert_eq!(english.config().unwrap(), default_config(Language::En));
    let english = Store::open_in_memory(None).unwrap();
    assert_eq!(english.config().unwrap(), default_config(Language::En));
}

#[test]
fn existing_store_keeps_its_config_whatever_the_language() {
    let path = temp_db("language");
    Store::open(&path, LOCALE).unwrap();
    let reopened = Store::open(&path, Some("en-US")).unwrap();
    assert_eq!(reopened.config().unwrap(), default_config(Language::Fr));
}

#[test]
fn language_setting_is_saved() {
    let s = store();
    assert_eq!(s.language().unwrap(), LanguageSetting::System, "missing");
    s.set_language(LanguageSetting::En).unwrap();
    assert_eq!(s.language().unwrap(), LanguageSetting::En);
    s.set_language(LanguageSetting::Fr).unwrap();
    assert_eq!(s.language().unwrap(), LanguageSetting::Fr);
}

#[test]
fn unknown_language_setting_falls_back_to_the_system() {
    let path = temp_db("unknown-language");
    Store::open(&path, LOCALE).unwrap();
    connect(&path)
        .execute(
            "INSERT INTO settings (key, value) VALUES ('language', 'klingon')",
            [],
        )
        .unwrap();
    let s = Store::open(&path, LOCALE).unwrap();
    assert_eq!(s.language().unwrap(), LanguageSetting::System);
}

#[test]
fn config_changes_are_saved() {
    let mut s = store();
    let mut cfg = default_config(Language::Fr);
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
        events: vec![event("death", "Death", 2, None)],
    });
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
}

#[test]
fn recoveries_are_saved() {
    let mut s = store();
    let mut cfg = default_config(Language::Fr);
    assert_eq!(cfg.recovery, DEFAULT_RECOVERY);
    cfg.recovery = 90;
    cfg.exercises[0].recovery = Some(120);
    cfg.exercises[1].recovery = Some(0);
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
    cfg.exercises[0].recovery = None;
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
}

#[test]
fn timed_exercises_sound_and_timer_settings_are_saved() {
    let mut s = store();
    let mut cfg = s.config().unwrap();
    assert!(cfg.sound && cfg.timer_auto_done, "on in a new database");
    assert_eq!(cfg.timer_prep, DEFAULT_TIMER_PREP);
    cfg.sound = false;
    cfg.timer_auto_done = false;
    cfg.timer_prep = 0;
    cfg.exercises[0].timed = Some(TimeUnit::Minutes);
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
    cfg.exercises[0].timed = None;
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
}

#[test]
fn exercises_counted_in_time_become_timed_once_migrated() {
    let path = legacy_db("timed", 3);
    connect(&path)
        .execute_batch(
            "INSERT INTO games (id, position, name) VALUES ('sekiro', 0, 'Sekiro');
             INSERT INTO events (game, id, position, name, draws, jokers) VALUES ('sekiro', 'death', 0, 'Death', 1, 0);
             INSERT INTO exercises (id, position, name, unit, min, max) VALUES
             ('plank', 0, 'Plank', ' S ', 20, 45), ('wall-sit', 1, 'Wall sit', 'secondes', 20, 45),
             ('run', 2, 'Run', 'min', 1, 2), ('squat', 3, 'Squats', 'reps', 10, 15), ('hold', 4, 'Hold', 'seconds held', 5, 10);",
        )
        .unwrap();
    let cfg = Store::open(&path, LOCALE).unwrap().config().unwrap();
    std::fs::remove_file(&path).unwrap();
    let timed: Vec<_> = cfg.exercises.iter().map(|ex| ex.timed).collect();
    assert_eq!(
        timed,
        [
            Some(TimeUnit::Seconds),
            Some(TimeUnit::Seconds),
            Some(TimeUnit::Minutes),
            None,
            None
        ]
    );
}

#[test]
fn renamed_settings_keep_their_value_once_migrated() {
    let path = legacy_db("renamed", 3);
    connect(&path)
        .execute_batch(
            "INSERT INTO games (id, position, name) VALUES ('sekiro', 0, 'Sekiro');
             INSERT INTO exercises (id, position, name, unit, min, max, pause) VALUES ('squat', 0, 'Squats', 'reps', 10, 15, 45);
             INSERT INTO settings (key, value) VALUES ('timer_sound', 'false'), ('pause', '90');",
        )
        .unwrap();
    let cfg = Store::open(&path, LOCALE).unwrap().config().unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(!cfg.sound);
    assert_eq!(cfg.recovery, 90);
    assert_eq!(cfg.exercises[0].recovery, Some(45));
}

/// Path of a temporary database specific to test `name`, deleted beforehand.
fn temp_db(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("squatsouls-{name}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn connect(path: &std::path::Path) -> rusqlite::Connection {
    rusqlite::Connection::open(path).unwrap()
}

/// Versions and dates of the migrations recorded in the database history.
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
    Store::open(&path, LOCALE).unwrap();
    Store::open(&path, LOCALE).unwrap();
    let applied = applied(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        applied.iter().map(|a| a.0).collect::<Vec<_>>(),
        [1, 2, 3, 4, 5, 6, 7, 8, 9],
        "a migration is applied only once"
    );
    assert!(
        applied
            .iter()
            .all(|a| chrono::DateTime::parse_from_rfc3339(&a.1).is_ok()),
        "date and time"
    );
}

/// Database migrated before refinery: its first `version` migrations applied, recorded in `user_version`.
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
         VALUES (1000, 1000, 'Sekiro', 'Death', 'Squats', 'reps', 12)",
        [],
    )
    .unwrap();
    path
}

#[test]
fn database_migrated_before_refinery_keeps_its_data() {
    for (name, version) in [("legacy-current", 3), ("legacy-behind", 2)] {
        let path = legacy_db(name, version);
        let history = Store::open(&path, LOCALE).unwrap().full_history().unwrap();
        Store::open(&path, LOCALE).unwrap().steam_names().unwrap();
        let applied = applied(&path);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(history.len(), 1, "{name}");
        assert_eq!(applied.len(), 9, "{name}");
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
        let mut names = Store::open(&path, LOCALE).unwrap().steam_names().unwrap();
        std::fs::remove_file(&path).unwrap();
        assert_eq!(
            names
                .name(814380, LANGUAGE, 1000, |_| None, |_| ())
                .unwrap(),
            "Sekiro",
            "{name}"
        );
        if !draft {
            assert_eq!(
                names.name(
                    1,
                    LANGUAGE,
                    2000,
                    |_| panic!("asked again before its date"),
                    |_| ()
                ),
                Err(SteamNameError::Unknown),
                "retry date kept"
            );
        }
    }
}

#[test]
fn steam_tray_and_intensity_settings_are_saved() {
    let mut s = store();
    let mut cfg = default_config(Language::Fr);
    assert!(!cfg.steam_detection && cfg.steam_notify);
    assert!(!cfg.tray && !cfg.close_to_tray && !cfg.start_hidden);
    assert!(!cfg.remember_intensity);
    cfg.remember_intensity = true;
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
        .name(814380, LANGUAGE, 1000, |_| Some((200, json.into())), |_| ())
        .unwrap();
    names.name(1, LANGUAGE, 2000, |_| None, |_| ()).unwrap_err();
    names
        .name(2, LANGUAGE, 3000, |_| Some((429, String::new())), |_| ())
        .unwrap_err();
    s.save_steam_names(&names).unwrap();
    assert_eq!(s.steam_names().unwrap(), names);
}

#[test]
fn database_migrated_before_comment_translation_still_opens() {
    let path = temp_db("translated");
    Store::open(&path, LOCALE).unwrap();
    let current = applied_checksums(&path);
    for (version, old) in PRE_TRANSLATION_CHECKSUMS {
        connect(&path)
            .execute(
                "UPDATE refinery_schema_history SET checksum = ?1 WHERE version = ?2",
                (old.to_string(), version),
            )
            .unwrap();
    }
    let opened = Store::open(&path, LOCALE).map(drop);
    let updated = applied_checksums(&path);
    std::fs::remove_file(&path).unwrap();
    opened.unwrap();
    assert_eq!(updated, current);
}

/// Checksums of the migrations recorded in the database history.
fn applied_checksums(path: &std::path::Path) -> Vec<String> {
    connect(path)
        .prepare("SELECT checksum FROM refinery_schema_history ORDER BY version")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn newer_database_is_refused() {
    let path = temp_db("newer");
    Store::open(&path, LOCALE).unwrap();
    connect(&path)
        .execute(
            "INSERT INTO refinery_schema_history (version, name, applied_on, checksum)
             VALUES (999, 'future', '2100-01-01T00:00:00Z', '0')",
            [],
        )
        .unwrap();
    let opened = Store::open(&path, LOCALE);
    std::fs::remove_file(&path).unwrap();
    assert!(opened.is_err());
}

#[test]
fn removed_game_takes_its_events_along() {
    let mut s = store();
    let mut cfg = default_config(Language::Fr);
    let removed = cfg.games.remove(0);
    s.save_config(&cfg).unwrap();
    // Recreating a game with the same id starts from an empty event list.
    cfg.games.push(Game {
        events: vec![event("death", "Death", 1, None)],
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
        event: "Death".into(),
        exercise: ex.into(),
        unit: "reps".into(),
        amount,
    };
    s.append_history(&[
        entry(t, t - 60, "Squats", 12),
        entry(t, t - 60, "Squats", 15),
        entry(t, t - 60, "Push-ups", 10),
    ])
    .unwrap();
    s.append_history(&[entry(t - 86_400 * 2, t - 86_400 * 2, "Push-ups", 8)])
        .unwrap();
    s.append_history(&[entry(t, t, "Squats", 5)]).unwrap();
    s.end_session(t - 60, t + 600).unwrap();

    let days = s.history(7, today).unwrap();
    assert_eq!(days[0].sets, 4);
    assert_eq!(days[2].sets, 1);
    let first = days[0].sessions.iter().find(|x| x.sets == 3).unwrap();
    assert_eq!(first.duration, 660, "until the saved end");
    assert_eq!(
        first.totals[0],
        Total {
            exercise: "Squats".into(),
            unit: "reps".into(),
            amount: 27,
            sets: 2
        }
    );

    s.remove_history(&[entry(t, t - 60, "Squats", 15)]).unwrap();
    assert_eq!(
        s.history(1, today).unwrap()[0].sets,
        3,
        "dismissed set removed"
    );
    s.append_history(&[entry(t, t - 60, "Squats", 15)]).unwrap();

    let summed = |s: &Store| {
        s.history(1, today).unwrap()[0]
            .sessions
            .iter()
            .find(|x| x.sets == 3)
            .cloned()
            .unwrap()
    };
    assert!(
        summed(&s).events.is_empty() && summed(&s).jokers.is_none(),
        "nothing recorded for this session"
    );
    let death = |count| EventCount {
        event: "Death".into(),
        count,
    };
    let boss = EventCount {
        event: "Boss".into(),
        count: 1,
    };
    let jokers = Jokers {
        earned: 3,
        spent: 2,
    };
    s.save_session_summary(t - 60, &[death(1)], Jokers::default())
        .unwrap();
    s.save_session_summary(t - 60, &[death(2), boss.clone()], jokers)
        .unwrap();
    assert_eq!(
        summed(&s).events,
        vec![death(2), boss],
        "replaced, in order"
    );
    assert_eq!(summed(&s).jokers, Some(jokers));

    s.remove_session(t).unwrap();
    assert_eq!(
        s.history(1, today).unwrap()[0].sets,
        3,
        "last session deleted"
    );
    s.remove_session(t - 60).unwrap();
    assert_eq!(s.full_history().unwrap().len(), 1);
    assert_eq!(s.active_day_count().unwrap(), 1);
    s.append_history(&[entry(t, t - 60, "Squats", 12)]).unwrap();
    let again = s.history(1, today).unwrap()[0].sessions[0].clone();
    assert!(
        again.events.is_empty() && again.jokers.is_none(),
        "events and jokers removed with the session"
    );
}

#[test]
fn last_game_is_kept() {
    let s = store();
    assert_eq!(s.last_game().unwrap(), "");
    s.set_last_game("ds3").unwrap();
    s.set_last_game("sekiro").unwrap();
    assert_eq!(s.last_game().unwrap(), "sekiro");
}

#[test]
fn last_intensity_is_kept() {
    let s = store();
    assert_eq!(s.intensity().unwrap(), Intensity::Normal);
    s.set_intensity(Intensity::Hard).unwrap();
    assert_eq!(s.intensity().unwrap(), Intensity::Hard);
}
