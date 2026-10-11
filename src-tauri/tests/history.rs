use chrono::Local;
use squat_souls_lib::fitness::*;
use std::collections::HashMap;
use std::io::Write;

#[test]
fn history_roundtrip_by_day() {
    let dir = std::env::temp_dir().join(format!("squat-souls-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("history.jsonl");
    let _ = std::fs::remove_file(&path);

    let now = Local::now();
    let today = now.date_naive();
    let entry = |at: i64, ex: &str, amount| LogEntry {
        at,
        session: 1,
        game: "Dark Souls".into(),
        event: "Mort".into(),
        exercise: ex.into(),
        unit: "rép.".into(),
        amount,
    };
    let t = now.timestamp();
    append_history(
        &path,
        &[
            entry(t, "Squats", 12),
            entry(t, "Squats", 15),
            entry(t, "Pompes", 10),
        ],
    )
    .unwrap();
    append_history(&path, &[entry(t - 86_400 * 2, "Pompes", 8)]).unwrap();
    append_history(&path, &[entry(t - 86_400 * 30, "Pompes", 99)]).unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap()
        .write_all(b"ligne abimee\n")
        .unwrap();

    let days = read_history(&path, 7, today).unwrap();
    assert_eq!(days.len(), 7);
    assert_eq!(days[0].sets, 3);
    assert_eq!(days[0].sessions.len(), 1);
    assert_eq!(days[0].sessions[0].game, "Dark Souls");
    assert_eq!(
        days[0].sessions[0].totals[0],
        Total {
            exercise: "Squats".into(),
            unit: "rép.".into(),
            amount: 27,
            sets: 2
        }
    );
    assert_eq!(days[2].sets, 1);
    assert_eq!(
        days.iter().map(|d| d.sets).sum::<usize>(),
        4,
        "l'entrée de 30 jours est hors fenêtre"
    );

    let full = read_full_history(&path).unwrap();
    assert_eq!(
        full.iter().map(|d| d.sets).collect::<Vec<_>>(),
        vec![3, 1, 1],
        "tous les jours actifs, du plus récent"
    );

    append_history(
        &path,
        &[LogEntry {
            session: 2,
            ..entry(t, "Squats", 5)
        }],
    )
    .unwrap();
    append_session_end(&path, 1, t + 600).unwrap();
    append_session_end(&path, 2, t + 60).unwrap();
    assert_eq!(read_history(&path, 1, today).unwrap()[0].sets, 4);
    remove_session(&path, 2).unwrap();
    assert!(!std::fs::read_to_string(&path)
        .unwrap()
        .contains(r#""session":2,"ended""#));
    assert_eq!(
        read_history(&path, 1, today).unwrap()[0].sets,
        3,
        "séance 2 effacée"
    );
    assert!(std::fs::read_to_string(&path)
        .unwrap()
        .contains("ligne abimee"));
    remove_session(&path, 1).unwrap();
    assert!(read_full_history(&path).unwrap().is_empty());

    let missing = read_history(&dir.join("absent.jsonl"), 3, today).unwrap();
    assert!(missing.iter().all(|d| d.sets == 0));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn sessions_grouped_by_session_and_game() {
    let entry = |at: i64, session: i64, game: &str, amount| LogEntry {
        at,
        session,
        game: game.into(),
        event: "Mort".into(),
        exercise: "Squats".into(),
        unit: "rép.".into(),
        amount,
    };
    let old: LogEntry = serde_json::from_str(
        r#"{"at":50,"game":"Elden Ring","event":"Mort","exercise":"Squats","unit":"rép.","amount":5}"#,
    )
    .unwrap();
    assert_eq!(old.session, 0);

    let groups = sessions(
        &[
            entry(100, 90, "Dark Souls", 10),
            entry(300, 250, "Sekiro", 8),
            entry(120, 90, "Dark Souls", 12),
            old,
        ],
        &HashMap::from([(90, 400)]),
    );
    assert_eq!(groups.len(), 3);
    assert_eq!((groups[0].game.as_str(), groups[0].sets), ("Sekiro", 1));
    assert_eq!(
        (groups[1].game.as_str(), groups[1].start, groups[1].sets),
        ("Dark Souls", 100, 2)
    );
    assert_eq!(groups[1].totals[0].amount, 22);
    assert_eq!(groups[0].duration, 50, "depuis le début de la séance");
    assert_eq!(groups[1].duration, 310, "jusqu'à la fin de la séance");
    assert_eq!(groups[2].game, "Elden Ring");
    assert_eq!(
        groups[2].duration, 0,
        "sans séance, depuis la première série"
    );
}
