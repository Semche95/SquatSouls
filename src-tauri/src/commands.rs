//! Tauri commands called by the interface.

use crate::app::{
    steam_app_id, AppError, Config, DayTotals, Draw, LanguageSetting, Session, SessionView,
    SteamNameError, SteamNames, SteamResponse,
};
use crate::state::{lock, now, AppState, ConfigView, LanguageView, NativeTexts, Shared};
use crate::tray;
use chrono::Local;
use std::sync::Mutex;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Manager};

const HISTORY_UNREADABLE: &str = "History unreadable";

#[tauri::command]
pub(crate) fn get_config(state: Shared) -> Result<ConfigView, AppError> {
    Ok(lock(&state)?.config_view())
}

#[tauri::command]
pub(crate) fn save_config(
    app: AppHandle,
    state: Shared,
    config: Config,
) -> Result<ConfigView, AppError> {
    config.validate()?;
    let mut s = lock(&state)?;
    let view = s.store_config(config)?;
    tray::apply(&app, &s).map_err(|e| {
        AppError::logged(
            "tray_unavailable",
            "Configuration saved, notification area unavailable",
            e,
        )
    })?;
    Ok(view)
}

#[tauri::command]
pub(crate) fn get_language(state: Shared) -> Result<LanguageView, AppError> {
    lock(&state)?.language_view()
}

#[tauri::command]
pub(crate) fn set_language(state: Shared, setting: String) -> Result<LanguageView, AppError> {
    let setting = setting
        .parse::<LanguageSetting>()
        .map_err(|_| AppError::new("invalid_language").with("value", setting.as_str()))?;
    let mut s = lock(&state)?;
    s.store_language(setting)?;
    s.language_view()
}

/// Texts of the tray menu and notifications, sent by the interface at launch and on each language change.
#[tauri::command]
pub(crate) fn set_native_texts(
    app: AppHandle,
    state: Shared,
    texts: NativeTexts,
) -> Result<(), AppError> {
    let mut s = lock(&state)?;
    s.native_texts = Some(texts);
    tray::update_texts(&app, &s)
        .map_err(|e| AppError::logged("tray_unavailable", "Notification area unavailable", e))
}

#[tauri::command]
pub(crate) fn draw(state: Shared, game_id: String, event_id: String) -> Result<Draw, AppError> {
    let mut guard = lock(&state)?;
    let AppState {
        config, session, ..
    } = &mut *guard;
    let draw = session.draw(config, &game_id, &event_id, &mut rand::rng())?;
    session.start(now());
    guard.save_session_summary();
    Ok(draw)
}

#[tauri::command]
pub(crate) fn complete_exercise(
    state: Shared,
    draw_id: u64,
    exercise_id: String,
) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    let entry = s.session.complete_exercise(draw_id, &exercise_id, now())?;
    if let Err(e) = s.store.append_history(&[entry]) {
        log::error!("History not saved: {e}");
    }
    s.save_session_summary();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn dismiss(state: Shared, draw_id: u64) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    let entries = s.session.dismiss(draw_id)?;
    if let Err(e) = s.store.remove_history(&entries) {
        log::error!("History not modified: {e}");
    }
    s.save_session_summary();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn halve(
    state: Shared,
    draw_id: u64,
    exercise_id: String,
) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session.halve(draw_id, &exercise_id)?;
    s.save_session_summary();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn skip_exercise(
    state: Shared,
    draw_id: u64,
    exercise_id: String,
) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session.skip_exercise(draw_id, &exercise_id)?;
    s.save_session_summary();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn skip_set(state: Shared, draw_id: u64) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session.skip_set(draw_id)?;
    s.save_session_summary();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn get_session(state: Shared) -> Result<SessionView, AppError> {
    Ok(lock(&state)?.session.view())
}

#[tauri::command]
pub(crate) fn start_session(state: Shared) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session.start(now());
    Ok(s.session.view())
}

/// Ends the session; the next one waits to be started.
#[tauri::command]
pub(crate) fn end_session(state: Shared) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    if let Some(started_at) = s.session.started_at.filter(|_| s.session.view().sets > 0) {
        if let Err(e) = s.store.end_session(started_at, now()) {
            log::error!("Session end not saved: {e}");
        }
    }
    s.session = Session::default();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn cancel_session(state: Shared) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.cancel_session()?;
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn get_last_game(state: Shared) -> Result<String, AppError> {
    lock(&state)?
        .store
        .last_game()
        .map_err(|e| AppError::internal("Last game unreadable", e))
}

#[tauri::command]
pub(crate) fn set_last_game(state: Shared, game_id: String) -> Result<(), AppError> {
    lock(&state)?
        .store
        .set_last_game(&game_id)
        .map_err(|e| AppError::internal("Last game not saved", e))
}

#[tauri::command]
pub(crate) fn get_history(state: Shared, days: u32) -> Result<Vec<DayTotals>, AppError> {
    lock(&state)?
        .store
        .history(days, Local::now().date_naive())
        .map_err(|e| AppError::internal(HISTORY_UNREADABLE, e))
}

#[tauri::command]
pub(crate) fn get_full_history(state: Shared) -> Result<Vec<DayTotals>, AppError> {
    lock(&state)?
        .store
        .full_history()
        .map_err(|e| AppError::internal(HISTORY_UNREADABLE, e))
}

#[tauri::command]
pub(crate) fn count_active_days(state: Shared) -> Result<usize, AppError> {
    lock(&state)?
        .store
        .active_day_count()
        .map_err(|e| AppError::internal(HISTORY_UNREADABLE, e))
}

/// AppId of the running Steam game, if any. Read off the main thread, like the name.
#[tauri::command]
pub(crate) async fn running_steam_app() -> Result<Option<u32>, AppError> {
    spawn_blocking(|| {
        let mut system = System::new();
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cmd(UpdateKind::Always)
                .with_environ(UpdateKind::Always),
        );
        let text = |args: &[std::ffi::OsString]| -> Vec<String> {
            args.iter()
                .map(|a| a.to_string_lossy().into_owned())
                .collect()
        };
        let processes = system.processes().values();
        let steam_running = || {
            processes.clone().any(|p| {
                p.name()
                    .to_string_lossy()
                    .to_lowercase()
                    .starts_with("steam")
            })
        };
        // Steam does not reset its record if it crashes: it only counts while Steam runs.
        processes
            .clone()
            .find_map(|p| steam_app_id(&text(p.cmd()), &text(p.environ())))
            .or_else(|| steam_running().then(noted_steam_app).flatten())
    })
    .await
    .map_err(|e| AppError::internal("Processes unreadable", e))
}

/// AppId of the running game recorded by Steam, for processes whose environment is unreadable
/// (Windows, macOS): in the registry on Windows.
#[cfg(windows)]
fn noted_steam_app() -> Option<u32> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let id: u32 = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Valve\Steam")
        .ok()?
        .get_value("RunningAppID")
        .ok()?;
    (id != 0).then_some(id)
}

/// AppId of the running game recorded by Steam: in its `registry.vdf` on macOS and Linux.
#[cfg(not(windows))]
fn noted_steam_app() -> Option<u32> {
    let home = std::path::PathBuf::from(std::env::var_os("HOME")?);
    let vdf = if cfg!(target_os = "macos") {
        home.join("Library/Application Support/Steam/registry.vdf")
    } else {
        home.join(".steam/registry.vdf")
    };
    crate::app::running_app_id_in_vdf(&std::fs::read_to_string(vdf).ok()?)
}

/// Maximum duration of a Steam Store call.
const STEAM_STORE_TIMEOUT: Duration = Duration::from_secs(10);

fn fetch(url: &str) -> SteamResponse {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(STEAM_STORE_TIMEOUT))
        .http_status_as_error(false)
        .build()
        .into();
    let mut response = agent.get(url).call().ok()?;
    let status = response.status().as_u16();
    Some((
        status,
        response.body_mut().read_to_string().unwrap_or_default(),
    ))
}

/// Name of a Steam game in `language`, asked from the Steam Store with the safeguards of `SteamNames`.
/// Requests go one at a time: the next one waits for the previous one's answer.
#[tauri::command]
pub(crate) async fn steam_game_name(
    app: AppHandle,
    app_id: u32,
    language: String,
) -> Result<String, SteamNameError> {
    spawn_blocking(move || {
        let names = app.state::<Mutex<SteamNames>>();
        let mut names = names.lock().map_err(|_| SteamNameError::Unknown)?;
        // If saving fails, the state stays kept until the app closes.
        let remember = |steam: &SteamNames| {
            if let Ok(mut state) = lock(&app.state::<Mutex<AppState>>()) {
                let _ = state.store.save_steam_names(steam);
            }
        };
        names.name(app_id, &language, now(), fetch, remember)
    })
    .await
    .map_err(|_| SteamNameError::Unknown)?
}

/// Notification of a launched Steam game, with the texts sent by the interface (none before them):
/// `{name}` is replaced by the game's name, its body tells whether the game is already configured.
#[tauri::command]
pub(crate) async fn notify(
    app: AppHandle,
    state: Shared<'_>,
    name: String,
    configured: bool,
) -> Result<(), AppError> {
    let Some(texts) = lock(&state)?.native_texts.clone() else {
        return Ok(());
    };
    let body = if configured {
        texts.launched_configured
    } else {
        texts.launched_unknown
    };
    let fill = |template: &str| template.replace("{name}", &name);
    show_notification(app, fill(&texts.launched_title), fill(&body))
        .await
        .map_err(|e| AppError::internal("Notification not sent", e))
}

/// System notification. On Linux, the plugin would sign it with the executable name and a system
/// icon missing outside an installation: here it carries the app name and its icon.
async fn show_notification(app: AppHandle, title: String, body: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let icon = app
            .path()
            .app_cache_dir()
            .map_err(|e| e.to_string())?
            .join("icon.png");
        spawn_blocking(move || {
            std::fs::create_dir_all(icon.parent().expect("cache directory"))?;
            std::fs::write(&icon, include_bytes!("../icons/128x128.png"))?;
            notify_rust::Notification::new()
                .appname(&app.package_info().name)
                .summary(&title)
                .body(&body)
                .icon(&icon.to_string_lossy())
                .show()
                .map(drop)
                .map_err(std::io::Error::other)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "linux"))]
    {
        use tauri_plugin_notification::NotificationExt;
        app.notification()
            .builder()
            .title(title)
            .body(body)
            .show()
            .map_err(|e| e.to_string())
    }
}
