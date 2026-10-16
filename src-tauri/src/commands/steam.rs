//! Steam: running game and game names from the Steam Store.

use crate::app::{steam_app_id, AppError, SteamNameError, SteamNames, SteamResponse};
use crate::state::{lock, now, AppState};
use std::sync::Mutex;
use std::time::Duration;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Manager};

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
