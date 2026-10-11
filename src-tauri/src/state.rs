//! Shared application state and configuration loading.

use crate::fitness::{default_config, Config, Session, Store};
use chrono::Local;
use serde::Serialize;
use std::sync::{Mutex, MutexGuard};
use tauri::State;

pub(crate) struct AppState {
    pub(crate) config: Config,
    /// Error met while loading the configuration (the default config is then used).
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
                    "Saved configuration ignored, default configuration used. {e}"
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
            .map_err(|e| format!("Configuration not saved: {e}"))?;
        // The session's game was removed: the session is cancelled.
        if self.session.game_removed(&config) {
            self.cancel_session()?;
        }
        self.config = config;
        self.config_error = None;
        self.session.apply_config(&self.config, &mut rand::rng());
        Ok(self.config_view())
    }

    /// Removes the current session's sets from the history and starts a new one.
    pub(crate) fn cancel_session(&mut self) -> Result<(), String> {
        self.store
            .remove_session(self.session.started_at)
            .map_err(|e| format!("History not modified: {e}"))?;
        self.session = Session::new(now());
        Ok(())
    }
}

fn load_config(store: &Store) -> Result<Config, String> {
    let config = store.config().map_err(|e| format!("Unreadable: {e}"))?;
    config.validate()?;
    Ok(config)
}

pub(crate) fn now() -> i64 {
    Local::now().timestamp()
}

pub(crate) fn lock<'a>(state: &'a Shared) -> Result<MutexGuard<'a, AppState>, String> {
    state
        .lock()
        .map_err(|_| "Application state unavailable.".to_string())
}
