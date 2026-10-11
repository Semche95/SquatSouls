//! État partagé de l'application et chargement de la configuration.

use crate::fitness::{default_config, remove_session, Config, Session};
use chrono::Local;
use serde::Serialize;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use tauri::State;

pub(crate) struct AppState {
    pub(crate) config: Config,
    /// Erreur rencontrée au chargement de config.json (la config par défaut est alors utilisée).
    pub(crate) config_error: Option<String>,
    pub(crate) config_path: PathBuf,
    pub(crate) history_path: PathBuf,
    /// Fichier qui garde le dernier jeu choisi d'un lancement à l'autre.
    pub(crate) last_game_path: PathBuf,
    pub(crate) session: Session,
}

pub(crate) type Shared<'a> = State<'a, Mutex<AppState>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfigView {
    config: Config,
    error: Option<String>,
}

impl AppState {
    pub(crate) fn new(config_dir: &Path, data_dir: &Path) -> Self {
        let config_path = config_dir.join("config.json");
        let (config, config_error) = load_config(&config_path);
        AppState {
            config,
            config_error,
            config_path,
            history_path: data_dir.join("history.jsonl"),
            last_game_path: data_dir.join("last_game"),
            session: Session::new(now()),
        }
    }

    pub(crate) fn config_view(&self) -> ConfigView {
        ConfigView {
            config: self.config.clone(),
            error: self.config_error.clone(),
        }
    }

    pub(crate) fn store_config(&mut self, config: Config) -> Result<ConfigView, String> {
        fs::write(&self.config_path, to_json(&config))
            .map_err(|e| format!("Impossible d'écrire {} : {e}", self.config_path.display()))?;
        self.config = config;
        self.config_error = None;
        // Les tirages en attente peuvent viser des exercices qui n'existent plus.
        self.session.clear_pending();
        Ok(self.config_view())
    }

    /// Retire de l'historique les séries de la séance en cours et en commence une nouvelle.
    pub(crate) fn cancel_session(&mut self) -> Result<(), String> {
        remove_session(&self.history_path, self.session.started_at)
            .map_err(|e| format!("Historique non modifié : {e}"))?;
        self.session = Session::new(now());
        Ok(())
    }
}

fn to_json(config: &Config) -> String {
    serde_json::to_string_pretty(config).unwrap_or_default()
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

pub(crate) fn now() -> i64 {
    Local::now().timestamp()
}

pub(crate) fn lock<'a>(state: &'a Shared) -> Result<MutexGuard<'a, AppState>, String> {
    state
        .lock()
        .map_err(|_| "État de l'application inaccessible.".to_string())
}
