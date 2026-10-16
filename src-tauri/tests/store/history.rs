use super::*;
use chrono::{Local, NaiveDate};

fn entry(at: i64, session: i64, exercise: &str, amount: u32) -> LogEntry {
    LogEntry {
        at,
        session,
        game: "Dark Souls".into(),
        event: "Death".into(),
        exercise: exercise.into(),
        unit: "reps".into(),
        amount,
    }
}

#[test]
fn history_and_sessions_are_stored() {
    let mut s = store();
    let now = Local::now();
    let today = now.date_naive();
    let t = now.timestamp();
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
    assert_eq!(s.full_history(None, usize::MAX).unwrap().len(), 1);
    assert_eq!(s.active_day_count().unwrap(), 1);
    s.append_history(&[entry(t, t - 60, "Squats", 12)]).unwrap();
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
        s.append_history(&[entry(noon(day), noon(day), "Squats", 10)])
            .unwrap();
    }
    s.append_history(&[entry(noon(6) + 60, noon(6) + 60, "Squats", 12)])
        .unwrap();
    s.end_session(noon(6) + 60, noon(6) + 600).unwrap();
    let dates = |before: Option<u32>, days| {
        s.full_history(
            before.map(|day| NaiveDate::from_ymd_opt(2026, 3, day).unwrap()),
            days,
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
    let page = s.full_history(Some(NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()), 1);
    assert_eq!(
        page.unwrap()[0].sessions[0].duration,
        540,
        "with what is kept of its sessions"
    );
}
