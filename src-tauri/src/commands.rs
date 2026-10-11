//! Commandes Tauri appelées par l'interface.

use crate::fitness::{
    steam_app_id, Config, DayTotals, Draw, Session, SessionView, SteamNameError, SteamNames,
    SteamResponse,
};
use crate::state::{lock, now, AppState, ConfigView, Shared};
use chrono::Local;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Manager};

const HISTORY_UNREADABLE: &str = "Historique illisible";

#[tauri::command]
pub(crate) fn get_config(state: Shared) -> Result<ConfigView, String> {
    Ok(lock(&state)?.config_view())
}

#[tauri::command]
pub(crate) fn save_config(state: Shared, config: Config) -> Result<ConfigView, String> {
    config.validate()?;
    lock(&state)?.store_config(config)
}

#[tauri::command]
pub(crate) fn draw(state: Shared, game_id: String, event_id: String) -> Result<Draw, String> {
    let mut guard = lock(&state)?;
    let AppState {
        config, session, ..
    } = &mut *guard;
    session.draw(config, &game_id, &event_id, &mut rand::rng())
}

#[tauri::command]
pub(crate) fn complete_exercise(
    state: Shared,
    draw_id: u64,
    exercise_id: String,
) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    let entry = s.session.complete_exercise(draw_id, &exercise_id, now())?;
    if let Err(e) = s.store.append_history(&[entry]) {
        eprintln!("Historique non enregistré : {e}");
    }
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn dismiss(state: Shared, draw_id: u64) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    let entries = s.session.dismiss(draw_id)?;
    if let Err(e) = s.store.remove_history(&entries) {
        eprintln!("Historique non modifié : {e}");
    }
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn halve(
    state: Shared,
    draw_id: u64,
    exercise_id: String,
) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    s.session.halve(draw_id, &exercise_id, now())?;
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn skip_exercise(
    state: Shared,
    draw_id: u64,
    exercise_id: String,
) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    s.session.skip_exercise(draw_id, &exercise_id, now())?;
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn skip_set(state: Shared, draw_id: u64) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    s.session.skip_set(draw_id, now())?;
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn get_session(state: Shared) -> Result<SessionView, String> {
    Ok(lock(&state)?.session.view())
}

#[tauri::command]
pub(crate) fn new_session(state: Shared) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    if s.session.view().sets > 0 {
        if let Err(e) = s.store.end_session(s.session.started_at, now()) {
            eprintln!("Fin de séance non enregistrée : {e}");
        }
    }
    s.session = Session::new(now());
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn cancel_session(state: Shared) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    s.cancel_session()?;
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn get_last_game(state: Shared) -> Result<String, String> {
    lock(&state)?
        .store
        .last_game()
        .map_err(|e| format!("Dernier jeu illisible : {e}"))
}

#[tauri::command]
pub(crate) fn set_last_game(state: Shared, game_id: String) -> Result<(), String> {
    lock(&state)?
        .store
        .set_last_game(&game_id)
        .map_err(|e| format!("Dernier jeu non enregistré : {e}"))
}

#[tauri::command]
pub(crate) fn get_history(state: Shared, days: u32) -> Result<Vec<DayTotals>, String> {
    lock(&state)?
        .store
        .history(days, Local::now().date_naive())
        .map_err(|e| format!("{HISTORY_UNREADABLE} : {e}"))
}

#[tauri::command]
pub(crate) fn get_full_history(state: Shared) -> Result<Vec<DayTotals>, String> {
    lock(&state)?
        .store
        .full_history()
        .map_err(|e| format!("{HISTORY_UNREADABLE} : {e}"))
}

/// AppId du jeu Steam en cours, s'il y en a un. Lu hors du fil principal, comme le nom.
#[tauri::command]
pub(crate) async fn running_steam_app() -> Result<Option<u32>, String> {
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
        system
            .processes()
            .values()
            .find_map(|p| steam_app_id(&text(p.cmd()), &text(p.environ())))
    })
    .await
    .map_err(|e| format!("Processus illisibles : {e}"))
}

/// Délai maximal d'un appel au Steam Store.
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

/// Nom d'un jeu Steam, demandé au Steam Store avec les garde-fous de `SteamNames`.
/// Les demandes passent une à une : la suivante attend la réponse de la précédente.
#[tauri::command]
pub(crate) async fn steam_game_name(app: AppHandle, app_id: u32) -> Result<String, SteamNameError> {
    spawn_blocking(move || {
        let names = app.state::<Mutex<SteamNames>>();
        let mut names = names.lock().map_err(|_| SteamNameError::Unknown)?;
        names.name(app_id, Instant::now(), fetch)
    })
    .await
    .map_err(|_| SteamNameError::Unknown)?
}
