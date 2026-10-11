use chrono::{Duration, Local, NaiveDate};
use squat_souls_lib::fitness::*;
use std::collections::HashMap;

fn entry(at: i64, session: i64, game: &str, amount: u32) -> LogEntry {
    LogEntry {
        at,
        session,
        game: game.into(),
        event: "Mort".into(),
        exercise: "Squats".into(),
        unit: "rép.".into(),
        amount,
    }
}

#[test]
fn sessions_grouped_by_session_and_game() {
    let groups = sessions(
        &[
            entry(100, 90, "Dark Souls", 10),
            entry(300, 250, "Sekiro", 8),
            entry(120, 90, "Dark Souls", 12),
        ],
        &HashMap::from([(90, 400)]),
    );
    assert_eq!(groups.len(), 2);
    assert_eq!((groups[0].game.as_str(), groups[0].sets), ("Sekiro", 1));
    assert_eq!(
        (groups[1].game.as_str(), groups[1].start, groups[1].sets),
        ("Dark Souls", 100, 2)
    );
    assert_eq!(groups[1].totals[0].amount, 22);
    assert_eq!(groups[0].duration, 50, "depuis le début de la séance");
    assert_eq!(groups[1].duration, 310, "jusqu'à la fin de la séance");
}

#[test]
fn days_window_and_active_days() {
    let now = Local::now();
    let today = now.date_naive();
    let t = now.timestamp();
    let entries = [
        entry(t - 86_400 * 30, 1, "Dark Souls", 99),
        entry(t - 86_400 * 2, 2, "Dark Souls", 8),
        entry(t, 3, "Dark Souls", 12),
        entry(t, 3, "Dark Souls", 15),
    ];
    let ends = HashMap::new();

    let days = last_days(&entries, &ends, 7, today);
    assert_eq!(days.len(), 7);
    assert_eq!(
        days.iter().map(|d| d.sets).collect::<Vec<_>>(),
        vec![2, 0, 1, 0, 0, 0, 0],
        "l'entrée de 30 jours est hors fenêtre"
    );

    let full = active_days(&entries, &ends);
    assert_eq!(
        full.iter().map(|d| d.sets).collect::<Vec<_>>(),
        vec![2, 1, 1]
    );
}

#[test]
fn window_starts_at_local_midnight() {
    let today = NaiveDate::from_ymd_opt(2026, 3, 10).unwrap();
    let start = window_start(3, today);
    let midnight = (today - Duration::days(2))
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_local_timezone(Local)
        .unwrap()
        .timestamp();
    assert_eq!(start, midnight);
    assert_eq!(window_start(0, today), window_start(1, today));
}
