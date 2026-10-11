use squat_souls_lib::fitness::*;

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

const REGISTRY: &str = r#""Registry"
{
	"HKCU"
	{
		"Software"
		{
			"Valve"
			{
				"Steam"
				{
					"language"		"french"
					"RunningAppID"		"RUNNING"
				}
			}
		}
	}
}
"#;

fn registry(running: &str) -> String {
    REGISTRY.replace("RUNNING", running)
}

#[test]
fn app_id_is_read_from_the_steam_registry() {
    assert_eq!(running_app_id_in_vdf(&registry("1245620")), Some(1245620));
    assert_eq!(running_app_id_in_vdf(&registry("0")), None);
    assert_eq!(
        running_app_id_in_vdf(&REGISTRY.replace("RunningAppID", "Other")),
        None
    );
}

const SEKIRO: &str =
    r#"{"814380":{"success":true,"data":{"type":"game","name":"Sekiro™: Shadows Die Twice"}}}"#;
const START: i64 = 1_700_000_000;

fn forget(_: &SteamNames) {}

fn ok(json: &str) -> impl FnOnce(&str) -> SteamResponse + '_ {
    move |_| Some((200, json.to_string()))
}

fn never(_: &str) -> SteamResponse {
    panic!("appel interdit")
}

#[test]
fn name_is_read_from_the_store_without_trademarks_and_asked_once() {
    let mut names = SteamNames::default();
    let name = names.name(
        814380,
        START,
        |url| {
            assert!(url.contains("appids=814380"));
            Some((200, SEKIRO.into()))
        },
        forget,
    );
    assert_eq!(name.as_deref(), Ok("Sekiro: Shadows Die Twice"));
    let much_later = START + STEAM_RETRY_DELAY * 100;
    let again = names.name(814380, much_later, never, forget);
    assert_eq!(again.as_deref(), Ok("Sekiro: Shadows Die Twice"));
}

#[test]
fn unknown_or_unanswered_app_is_asked_again_after_a_delay() {
    let mut names = SteamNames::default();
    let later = START + STEAM_STORE_INTERVAL;
    let unknown = names.name(1, START, ok(r#"{"1":{"success":false}}"#), forget);
    assert_eq!(unknown, Err(SteamNameError::Unknown));
    assert_eq!(
        names.name(2, later, |_| None, forget),
        Err(SteamNameError::Unknown)
    );
    let soon = START + STEAM_RETRY_DELAY - 1;
    assert_eq!(
        names.name(1, soon, never, forget),
        Err(SteamNameError::Unknown)
    );
    let json = r#"{"1":{"data":{"name":"Retrouvé"}}}"#;
    let found = names.name(1, START + STEAM_RETRY_DELAY, ok(json), forget);
    assert_eq!(found.as_deref(), Ok("Retrouvé"));
}

#[test]
fn store_is_called_at_most_once_per_interval() {
    let mut names = SteamNames::default();
    names.name(814380, START, ok(SEKIRO), forget).unwrap();
    let soon = START + STEAM_STORE_INTERVAL - 1;
    let waited = names.name(1245620, soon, never, forget);
    assert_eq!(waited, Err(SteamNameError::Wait));
    let json = r#"{"1245620":{"data":{"name":"ELDEN RING"}}}"#;
    let name = names.name(1245620, START + STEAM_STORE_INTERVAL, ok(json), forget);
    assert_eq!(name.as_deref(), Ok("ELDEN RING"));
}

#[test]
fn rate_limited_store_is_not_called_for_a_while() {
    for status in [429, 403] {
        let mut names = SteamNames::default();
        let refused = names.name(814380, START, |_| Some((status, String::new())), forget);
        assert_eq!(refused, Err(SteamNameError::Blocked));
        let soon = START + STEAM_BLOCK_DELAY - 1;
        let blocked = names.name(814380, soon, never, forget);
        assert_eq!(blocked, Err(SteamNameError::Blocked));
        let name = names.name(814380, START + STEAM_BLOCK_DELAY, ok(SEKIRO), forget);
        assert!(name.is_ok());
    }
}

#[test]
fn state_is_handed_over_after_each_call() {
    let mut names = SteamNames::default();
    let mut remembered = None;
    let refused = |_: &str| Some((429, String::new()));
    names
        .name(814380, START, refused, |s| remembered = Some(s.clone()))
        .unwrap_err();
    assert_eq!(remembered.as_ref(), Some(&names));
    remembered = None;
    names
        .name(814380, START + 1, never, |s| remembered = Some(s.clone()))
        .unwrap_err();
    assert_eq!(remembered, None);
}
