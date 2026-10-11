//! Commandes Tauri appelées par l'interface.

use crate::fitness::{
    append_history, append_session_end, read_full_history, read_history, Config, DayTotals, Draw,
    Session, SessionView,
};
use crate::state::{lock, now, AppState, ConfigView, Shared};
use chrono::Local;
use std::fs;

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
pub(crate) fn complete(state: Shared, draw_id: u64) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    let entries = s.session.complete(draw_id, now())?;
    if let Err(e) = append_history(&s.history_path, &entries) {
        eprintln!("Historique non enregistré : {e}");
    }
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn dismiss(state: Shared, draw_id: u64) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    s.session.dismiss(draw_id)?;
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn use_joker(state: Shared, draw_id: u64) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    s.session.use_joker(draw_id, now())?;
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
        if let Err(e) = append_session_end(&s.history_path, s.session.started_at, now()) {
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
    let s = lock(&state)?;
    Ok(fs::read_to_string(&s.last_game_path)
        .map(|id| id.trim().to_string())
        .unwrap_or_default())
}

#[tauri::command]
pub(crate) fn set_last_game(state: Shared, game_id: String) -> Result<(), String> {
    let s = lock(&state)?;
    fs::write(&s.last_game_path, game_id).map_err(|e| format!("Dernier jeu non enregistré : {e}"))
}

#[tauri::command]
pub(crate) fn get_history(state: Shared, days: u32) -> Result<Vec<DayTotals>, String> {
    let s = lock(&state)?;
    read_history(&s.history_path, days, Local::now().date_naive())
        .map_err(|e| format!("{HISTORY_UNREADABLE} : {e}"))
}

#[tauri::command]
pub(crate) fn get_full_history(state: Shared) -> Result<Vec<DayTotals>, String> {
    let s = lock(&state)?;
    read_full_history(&s.history_path).map_err(|e| format!("{HISTORY_UNREADABLE} : {e}"))
}
