use super::*;

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
fn last_game_is_kept() {
    let s = store();
    assert_eq!(s.last_game().unwrap(), "");
    s.set_last_game("ds3").unwrap();
    s.set_last_game("sekiro").unwrap();
    assert_eq!(s.last_game().unwrap(), "sekiro");
}

#[test]
fn onboarding_seen_is_kept() {
    let s = store();
    assert!(!s.onboarding_seen().unwrap());
    s.set_onboarding_seen().unwrap();
    assert!(s.onboarding_seen().unwrap());
}

#[test]
fn last_intensity_is_kept() {
    let s = store();
    assert_eq!(s.intensity().unwrap(), Intensity::Normal);
    s.set_intensity(Intensity::Hard).unwrap();
    assert_eq!(s.intensity().unwrap(), Intensity::Hard);
}

#[test]
fn last_limit_is_kept() {
    let s = store();
    assert_eq!(s.limit().unwrap(), None);
    let limit = SessionLimit {
        kind: LimitKind::PlayTime,
        value: 90,
    };
    s.set_limit(Some(limit)).unwrap();
    assert_eq!(s.limit().unwrap(), Some(limit));
    s.set_limit(None).unwrap();
    assert_eq!(s.limit().unwrap(), None);
}
