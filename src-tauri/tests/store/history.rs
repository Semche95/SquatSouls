use super::*;
use chrono::{Local, NaiveDate};

/// Id of the session started at `start`: its start written out.
fn session(start: i64) -> String {
    start.to_string()
}

/// Set done at `at` of the session started at `start`.
fn entry(at: i64, start: i64, exercise: &str, amount: u32) -> LogEntry {
    LogEntry {
        at,
        session: session(start),
        game: "Dark Souls".into(),
        event: "Death".into(),
        exercise: exercise.into(),
        unit: "reps".into(),
        amount,
    }
}

/// Appends `entries` to the history, as the app does once their sessions are recorded.
fn append(s: &mut Store, entries: &[LogEntry]) {
    for e in entries {
        s.record_session(&e.session, e.session.parse().unwrap())
            .unwrap();
    }
    s.append_history(entries).unwrap();
}

#[test]
fn history_and_sessions_are_stored() {
    let mut s = store();
    let now = Local::now();
    let today = now.date_naive();
    let t = now.timestamp();
    append(
        &mut s,
        &[
            entry(t, t - 60, "Squats", 12),
            entry(t, t - 60, "Squats", 15),
            entry(t, t - 60, "Push-ups", 10),
        ],
    );
    append(
        &mut s,
        &[entry(t - 86_400 * 2, t - 86_400 * 2, "Push-ups", 8)],
    );
    append(&mut s, &[entry(t, t, "Squats", 5)]);
    s.end_session(&session(t - 60), t + 600).unwrap();

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
    s.save_session_summary(&session(t - 60), &[death(1)], Jokers::default())
        .unwrap();
    s.save_session_summary(&session(t - 60), &[death(2), boss.clone()], jokers)
        .unwrap();
    assert_eq!(
        summed(&s).events,
        vec![death(2), boss],
        "replaced, in order"
    );
    assert_eq!(summed(&s).jokers, Some(jokers));

    s.remove_session(&session(t)).unwrap();
    assert_eq!(
        s.history(1, today).unwrap()[0].sets,
        3,
        "last session deleted"
    );
    s.remove_session(&session(t - 60)).unwrap();
    assert_eq!(s.full_history(None, usize::MAX, None).unwrap().len(), 1);
    assert_eq!(s.active_day_count(None).unwrap(), 1);
    append(&mut s, &[entry(t, t - 60, "Squats", 12)]);
    let again = s.history(1, today).unwrap()[0].sessions[0].clone();
    assert!(
        again.events.is_empty() && again.jokers.is_none(),
        "events and jokers removed with the session"
    );
}

#[test]
fn full_history_is_read_by_pages() {
    let mut s = store();
    let noon = |day: u32| {
        NaiveDate::from_ymd_opt(2026, 3, day)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp()
    };
    for day in [2, 5, 6, 9] {
        append(&mut s, &[entry(noon(day), noon(day), "Squats", 10)]);
    }
    append(&mut s, &[entry(noon(6) + 60, noon(6) + 60, "Squats", 12)]);
    s.end_session(&session(noon(6) + 60), noon(6) + 600)
        .unwrap();
    let dates = |before: Option<u32>, days| {
        s.full_history(
            before.map(|day| NaiveDate::from_ymd_opt(2026, 3, day).unwrap()),
            days,
            None,
        )
        .unwrap()
        .iter()
        .map(|d| (d.date.clone(), d.sets))
        .collect::<Vec<_>>()
    };
    let day = |day: u32, sets| (format!("2026-03-{day:02}"), sets);
    assert_eq!(dates(None, 2), vec![day(9, 1), day(6, 2)], "most recent");
    assert_eq!(
        dates(Some(6), 2),
        vec![day(5, 1), day(2, 1)],
        "before the last one shown"
    );
    assert_eq!(dates(Some(2), 2), vec![], "nothing older");
    let page = s.full_history(Some(NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()), 1, None);
    assert_eq!(
        page.unwrap()[0].sessions[0].duration,
        540,
        "with what is kept of its sessions"
    );
}

#[test]
fn full_history_is_filtered_by_game_and_month() {
    let mut s = store();
    let noon = |month: u32, day: u32| {
        NaiveDate::from_ymd_opt(2026, month, day)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap()
            .timestamp()
    };
    for (start, game) in [
        (noon(1, 15), "Dark Souls"),
        (noon(3, 2), "Sekiro"),
        (noon(3, 2) + 3600, "Dark Souls"),
        (noon(4, 10), "Sekiro"),
    ] {
        let e = LogEntry {
            game: game.into(),
            ..entry(start, start, "Squats", 10)
        };
        append(&mut s, &[e]);
    }
    let mut games = s.history_games().unwrap();
    games.sort();
    assert_eq!(games, ["Dark Souls", "Sekiro"]);
    assert_eq!(
        s.history_months(None).unwrap(),
        ["2026-04", "2026-03", "2026-01"]
    );
    assert_eq!(
        s.history_months(Some("Sekiro")).unwrap(),
        ["2026-04", "2026-03"],
        "only the months the game was played"
    );
    assert_eq!(s.active_day_count(None).unwrap(), 3);
    assert_eq!(s.active_day_count(Some("Sekiro")).unwrap(), 2);
    let days = |before: Option<NaiveDate>, game| {
        s.full_history(before, usize::MAX, game)
            .unwrap()
            .into_iter()
            .map(|d| {
                let games: Vec<_> = d.sessions.into_iter().map(|se| se.game).collect();
                (d.date, games)
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        days(None, Some("Sekiro")),
        [
            ("2026-04-10".to_string(), vec!["Sekiro".to_string()]),
            ("2026-03-02".to_string(), vec!["Sekiro".to_string()]),
        ],
        "without the sessions of the other games"
    );
    assert_eq!(
        days(NaiveDate::from_ymd_opt(2026, 4, 1), Some("Dark Souls")),
        [
            ("2026-03-02".to_string(), vec!["Dark Souls".to_string()]),
            ("2026-01-15".to_string(), vec!["Dark Souls".to_string()]),
        ],
        "from the end of a month"
    );
    assert_eq!(
        days(NaiveDate::from_ymd_opt(2026, 4, 1), None)[0].1,
        ["Dark Souls", "Sekiro"],
        "every game without a filter"
    );
}
