use super::*;

#[test]
fn config_changes_are_saved() {
    let mut s = store();
    let mut cfg = seed_config(Language::Fr);
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
    let mut cfg = seed_config(Language::Fr);
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
fn timed_exercises_on_each_side_sound_and_timer_settings_are_saved() {
    let mut s = store();
    let mut cfg = s.config().unwrap();
    assert!(cfg.sound && cfg.timer_auto_done, "on in a new database");
    assert_eq!(cfg.timer_prep, DEFAULT_TIMER_PREP);
    cfg.sound = false;
    cfg.timer_auto_done = false;
    cfg.timer_prep = 0;
    cfg.exercises[0].timed = Some(TimeUnit::Minutes);
    cfg.exercises[0].each_side = true;
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
    cfg.exercises[0].timed = None;
    cfg.exercises[0].each_side = false;
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
}

#[test]
fn the_duration_of_a_repetition_is_saved() {
    let mut s = store();
    let mut cfg = s.config().unwrap();
    assert_eq!(cfg.rep_duration, DEFAULT_REP_DURATION, "in a new database");
    cfg.rep_duration = 5;
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap().rep_duration, 5);
}

#[test]
fn update_settings_are_on_by_default_and_saved() {
    let mut s = store();
    let mut cfg = s.config().unwrap();
    assert!(cfg.update_check && cfg.update_auto, "on in a new database");
    cfg.update_check = false;
    cfg.update_auto = false;
    s.save_config(&cfg).unwrap();
    assert_eq!(s.config().unwrap(), cfg);
}

#[test]
fn steam_tray_intensity_and_limit_settings_are_saved() {
    let mut s = store();
    let mut cfg = seed_config(Language::Fr);
    assert!(!cfg.steam_detection && cfg.steam_notify);
    assert!(!cfg.tray && !cfg.close_to_tray && !cfg.start_hidden);
    assert!(!cfg.remember_intensity && !cfg.remember_limit);
    cfg.remember_intensity = true;
    cfg.remember_limit = true;
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
fn removed_game_takes_its_events_along() {
    let mut s = store();
    let mut cfg = seed_config(Language::Fr);
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
