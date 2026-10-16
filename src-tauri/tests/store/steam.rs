use super::*;

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
