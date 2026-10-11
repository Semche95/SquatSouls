use squat_souls_lib::fitness::*;
use std::time::{Duration, Instant};

const NONE: [&str; 0] = [];

#[test]
fn app_id_is_read_from_the_steam_launch_command() {
    let cmd = [
        "/home/me/.steam/ubuntu12_32/reaper",
        "SteamLaunch",
        "AppId=1245620",
        "--",
        "eldenring.exe",
    ];
    assert_eq!(steam_app_id(&cmd, &NONE), Some(1245620));
}

#[test]
fn app_id_is_read_from_the_environment() {
    let environ = ["HOME=/home/me", "SteamAppId=814380"];
    assert_eq!(steam_app_id(&["sekiro.exe"], &environ), Some(814380));
}

#[test]
fn other_processes_have_no_app_id() {
    // Un AppId hors de `SteamLaunch`, ou nul (raccourci hors Steam), ne compte pas.
    assert_eq!(steam_app_id(&["tool", "AppId=1245620"], &NONE), None);
    assert_eq!(steam_app_id(&["game"], &["SteamAppId=0"]), None);
    assert_eq!(steam_app_id(&NONE, &NONE), None);
}

const SEKIRO: &str =
    r#"{"814380":{"success":true,"data":{"type":"game","name":"Sekiro™: Shadows Die Twice"}}}"#;

fn ok(json: &str) -> impl FnOnce(&str) -> SteamResponse + '_ {
    move |_| Some((200, json.to_string()))
}

#[test]
fn name_is_read_from_the_store_without_trademarks_and_asked_once() {
    let mut names = SteamNames::default();
    let now = Instant::now();
    let name = names.name(814380, now, |url| {
        assert!(url.contains("appids=814380"));
        Some((200, SEKIRO.into()))
    });
    assert_eq!(name.as_deref(), Ok("Sekiro: Shadows Die Twice"));
    let again = names.name(814380, now, |_| panic!("déjà demandé"));
    assert_eq!(again.as_deref(), Ok("Sekiro: Shadows Die Twice"));
}

#[test]
fn unknown_or_unanswered_app_is_not_asked_again() {
    let mut names = SteamNames::default();
    let start = Instant::now();
    let later = start + STEAM_STORE_INTERVAL;
    let unknown = names.name(1, start, ok(r#"{"1":{"success":false}}"#));
    assert_eq!(unknown, Err(SteamNameError::Unknown));
    assert_eq!(names.name(2, later, |_| None), Err(SteamNameError::Unknown));
    let never = |_: &str| panic!("déjà demandé");
    assert_eq!(names.name(1, later, never), Err(SteamNameError::Unknown));
    assert_eq!(names.name(2, later, never), Err(SteamNameError::Unknown));
}

#[test]
fn store_is_called_at_most_once_per_interval() {
    let mut names = SteamNames::default();
    let start = Instant::now();
    names.name(814380, start, ok(SEKIRO)).unwrap();
    let soon = start + STEAM_STORE_INTERVAL - Duration::from_secs(1);
    let waited = names.name(1245620, soon, |_| panic!("trop tôt"));
    assert_eq!(waited, Err(SteamNameError::Wait));
    let json = r#"{"1245620":{"data":{"name":"ELDEN RING"}}}"#;
    let name = names.name(1245620, start + STEAM_STORE_INTERVAL, ok(json));
    assert_eq!(name.as_deref(), Ok("ELDEN RING"));
}

#[test]
fn rate_limited_store_is_no_longer_called() {
    for status in [429, 403] {
        let mut names = SteamNames::default();
        let start = Instant::now();
        let refused = names.name(814380, start, |_| Some((status, String::new())));
        assert_eq!(refused, Err(SteamNameError::Blocked));
        let much_later = start + STEAM_STORE_INTERVAL * 100;
        let blocked = names.name(814380, much_later, |_| panic!("bloqué"));
        assert_eq!(blocked, Err(SteamNameError::Blocked));
    }
}
