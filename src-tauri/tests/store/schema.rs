use super::*;

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
    assert!(
        cfg.update_check,
        "on in a database saved before the setting"
    );
    assert_eq!(cfg.recovery, 90);
    assert_eq!(cfg.exercises[0].recovery, Some(45));
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
        [1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
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
        include_str!("../../migrations/V1__schema.sql"),
        include_str!("../../migrations/V2__steam_detection.sql"),
        include_str!("../../migrations/V3__steam_names.sql"),
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
        let history = Store::open(&path, LOCALE)
            .unwrap()
            .full_history(None, usize::MAX)
            .unwrap();
        Store::open(&path, LOCALE).unwrap().steam_names().unwrap();
        let applied = applied(&path);
        std::fs::remove_file(&path).unwrap();
        assert_eq!(history.len(), 1, "{name}");
        assert_eq!(applied.len(), 10, "{name}");
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
        drop(conn);
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
