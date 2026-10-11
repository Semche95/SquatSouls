mod fitness;

use chrono::Local;
use fitness::{
    append_history, append_session_end, default_config, read_full_history, read_history,
    remove_session, Config, DayTotals, Draw, Session, SessionView,
};
use serde::Serialize;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tauri::{Manager, State};

struct AppState {
    config: Config,
    /// Erreur rencontrée au chargement de config.json (la config par défaut est alors utilisée).
    config_error: Option<String>,
    config_path: PathBuf,
    history_path: PathBuf,
    /// Fichier qui garde le dernier jeu choisi d'un lancement à l'autre.
    last_game_path: PathBuf,
    session: Session,
}

type Shared<'a> = State<'a, Mutex<AppState>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigView {
    config: Config,
    error: Option<String>,
}

impl AppState {
    fn config_view(&self) -> ConfigView {
        ConfigView {
            config: self.config.clone(),
            error: self.config_error.clone(),
        }
    }

    fn store_config(&mut self, config: Config) -> Result<ConfigView, String> {
        fs::write(&self.config_path, to_json(&config))
            .map_err(|e| format!("Impossible d'écrire {} : {e}", self.config_path.display()))?;
        self.config = config;
        self.config_error = None;
        // Les tirages en attente peuvent viser des exercices qui n'existent plus.
        self.session.clear_pending();
        Ok(self.config_view())
    }
}

fn to_json(config: &Config) -> String {
    serde_json::to_string_pretty(config).unwrap_or_default()
}

fn now() -> i64 {
    Local::now().timestamp()
}

fn lock<'a>(state: &'a Shared) -> Result<MutexGuard<'a, AppState>, String> {
    state
        .lock()
        .map_err(|_| "État de l'application inaccessible.".to_string())
}

fn load_config(path: &Path) -> (Config, Option<String>) {
    match fs::read_to_string(path) {
        Ok(text) => {
            let parsed = serde_json::from_str::<Config>(&text)
                .map_err(|e| format!("JSON invalide : {e}"))
                .and_then(|c| c.validate().map(|_| c));
            match parsed {
                Ok(config) => (config, None),
                Err(e) => (
                    default_config(),
                    Some(format!(
                        "{} ignoré, configuration par défaut utilisée. {e}",
                        path.display()
                    )),
                ),
            }
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let config = default_config();
            let _ = fs::write(path, to_json(&config));
            (config, None)
        }
        Err(e) => (
            default_config(),
            Some(format!("Lecture de {} impossible : {e}", path.display())),
        ),
    }
}

#[tauri::command]
fn get_config(state: Shared) -> Result<ConfigView, String> {
    Ok(lock(&state)?.config_view())
}

#[tauri::command]
fn save_config(state: Shared, config: Config) -> Result<ConfigView, String> {
    config.validate()?;
    lock(&state)?.store_config(config)
}

#[tauri::command]
fn draw(state: Shared, game_id: String, event_id: String) -> Result<Draw, String> {
    let mut guard = lock(&state)?;
    let AppState {
        config, session, ..
    } = &mut *guard;
    session.draw(config, &game_id, &event_id, &mut rand::rng())
}

#[tauri::command]
fn complete(state: Shared, draw_id: u64) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    let entries = s.session.complete(draw_id, now())?;
    if let Err(e) = append_history(&s.history_path, &entries) {
        eprintln!("Historique non enregistré : {e}");
    }
    Ok(s.session.view())
}

#[tauri::command]
fn dismiss(state: Shared, draw_id: u64) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    s.session.dismiss(draw_id)?;
    Ok(s.session.view())
}

#[tauri::command]
fn use_joker(state: Shared, draw_id: u64) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    s.session.use_joker(draw_id, now())?;
    Ok(s.session.view())
}

#[tauri::command]
fn get_session(state: Shared) -> Result<SessionView, String> {
    Ok(lock(&state)?.session.view())
}

#[tauri::command]
fn new_session(state: Shared) -> Result<SessionView, String> {
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
fn cancel_session(state: Shared) -> Result<SessionView, String> {
    let mut s = lock(&state)?;
    remove_session(&s.history_path, s.session.started_at)
        .map_err(|e| format!("Historique non modifié : {e}"))?;
    s.session = Session::new(now());
    Ok(s.session.view())
}

#[tauri::command]
fn get_last_game(state: Shared) -> Result<String, String> {
    let s = lock(&state)?;
    Ok(fs::read_to_string(&s.last_game_path)
        .map(|id| id.trim().to_string())
        .unwrap_or_default())
}

#[tauri::command]
fn set_last_game(state: Shared, game_id: String) -> Result<(), String> {
    let s = lock(&state)?;
    fs::write(&s.last_game_path, game_id).map_err(|e| format!("Dernier jeu non enregistré : {e}"))
}

#[tauri::command]
fn get_history(state: Shared, days: u32) -> Result<Vec<DayTotals>, String> {
    let s = lock(&state)?;
    read_history(&s.history_path, days, Local::now().date_naive())
        .map_err(|e| format!("Historique illisible : {e}"))
}

#[tauri::command]
fn get_full_history(state: Shared) -> Result<Vec<DayTotals>, String> {
    let s = lock(&state)?;
    read_full_history(&s.history_path).map_err(|e| format!("Historique illisible : {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let data_dir = app.path().app_data_dir()?;
            fs::create_dir_all(&config_dir)?;
            fs::create_dir_all(&data_dir)?;

            let config_path = config_dir.join("config.json");
            let (config, config_error) = load_config(&config_path);
            app.manage(Mutex::new(AppState {
                config,
                config_error,
                config_path,
                history_path: data_dir.join("history.jsonl"),
                last_game_path: data_dir.join("last_game"),
                session: Session::new(now()),
            }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            draw,
            complete,
            dismiss,
            use_joker,
            get_session,
            new_session,
            cancel_session,
            get_last_game,
            set_last_game,
            get_history,
            get_full_history
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de Squat Souls");
}
