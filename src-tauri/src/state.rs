//! État partagé de l'application et chargement de la configuration.

use crate::fitness::{default_config, Config, Session, Store};
use chrono::Local;
use serde::Serialize;
use std::sync::{Mutex, MutexGuard};
use tauri::State;

pub(crate) struct AppState {
    pub(crate) config: Config,
    /// Erreur rencontrée au chargement de la configuration (la config par défaut est alors utilisée).
    pub(crate) config_error: Option<String>,
    pub(crate) store: Store,
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
    pub(crate) fn new(store: Store) -> Self {
        let (config, config_error) = match load_config(&store) {
            Ok(config) => (config, None),
            Err(e) => (
                default_config(),
                Some(format!(
                    "Configuration enregistrée ignorée, configuration par défaut utilisée. {e}"
                )),
            ),
        };
        AppState {
            config,
            config_error,
            store,
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
        self.store
            .save_config(&config)
            .map_err(|e| format!("Configuration non enregistrée : {e}"))?;
        self.config = config;
        self.config_error = None;
        // Les tirages en attente peuvent viser des exercices qui n'existent plus.
        self.session.clear_pending();
        Ok(self.config_view())
    }

    /// Retire de l'historique les séries de la séance en cours et en commence une nouvelle.
    pub(crate) fn cancel_session(&mut self) -> Result<(), String> {
        self.store
            .remove_session(self.session.started_at)
            .map_err(|e| format!("Historique non modifié : {e}"))?;
        self.session = Session::new(now());
        Ok(())
    }
}

fn load_config(store: &Store) -> Result<Config, String> {
    let config = store
        .config()
        .map_err(|e| format!("Lecture impossible : {e}"))?;
    config.validate()?;
    Ok(config)
}

pub(crate) fn now() -> i64 {
    Local::now().timestamp()
}

pub(crate) fn lock<'a>(state: &'a Shared) -> Result<MutexGuard<'a, AppState>, String> {
    state
        .lock()
        .map_err(|_| "État de l'application inaccessible.".to_string())
}
