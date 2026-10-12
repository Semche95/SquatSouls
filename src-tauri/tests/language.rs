use squat_souls_lib::app::*;

#[test]
fn system_language_follows_a_french_locale() {
    for locale in ["fr", "fr-FR", "fr_CA.UTF-8", "FR-be"] {
        assert_eq!(
            LanguageSetting::System.resolve(Some(locale)),
            Language::Fr,
            "{locale}"
        );
    }
}

#[test]
fn system_language_is_english_otherwise() {
    for locale in [Some("de-DE"), Some("en-US"), Some(""), None] {
        assert_eq!(
            LanguageSetting::System.resolve(locale),
            Language::En,
            "{locale:?}"
        );
    }
}

#[test]
fn explicit_language_ignores_the_locale() {
    assert_eq!(LanguageSetting::En.resolve(Some("fr-FR")), Language::En);
    assert_eq!(LanguageSetting::Fr.resolve(Some("de-DE")), Language::Fr);
    assert_eq!(LanguageSetting::Fr.resolve(None), Language::Fr);
}

#[test]
fn setting_parses_only_known_values() {
    for setting in [
        LanguageSetting::System,
        LanguageSetting::En,
        LanguageSetting::Fr,
    ] {
        assert_eq!(setting.as_str().parse(), Ok(setting));
    }
    assert!("de".parse::<LanguageSetting>().is_err());
    assert!("".parse::<LanguageSetting>().is_err());
}
