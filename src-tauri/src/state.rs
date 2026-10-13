//! Shared application state and configuration loading.

use crate::app::{default_config, AppError, Config, Language, LanguageSetting, Session, Store};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};
use tauri::State;

pub(crate) struct AppState {
    pub(crate) config: Config,
    /// Error met while loading the configuration (the default config is then used).
    pub(crate) config_error: Option<AppError>,
    pub(crate) store: Store,
    pub(crate) session: Session,
    /// System locale, read at launch to resolve the `System` language setting.
    pub(crate) system_locale: Option<String>,
    pub(crate) language: Language,
    /// Texts of the tray menu and notifications, translated by the interface: none until it sends them.
    pub(crate) native_texts: Option<NativeTexts>,
}

pub(crate) type Shared<'a> = State<'a, Mutex<AppState>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfigView {
    config: Config,
    error: Option<AppError>,
}

#[derive(Serialize)]
pub(crate) struct LanguageView {
    setting: LanguageSetting,
    language: Language,
}

/// Translated texts of the tray menu and of the launched Steam game notification.
/// Notification texts are templates where `{name}` stands for the game's name.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NativeTexts {
    pub(crate) show: String,
    pub(crate) quit: String,
    pub(crate) launched_title: String,
    pub(crate) launched_configured: String,
    pub(crate) launched_unknown: String,
}

impl AppState {
    pub(crate) fn new(store: Store, system_locale: Option<String>) -> Self {
        let language = store
            .language()
            .unwrap_or_else(|e| {
                log::error!("Language setting unreadable: {e}");
                LanguageSetting::System
            })
            .resolve(system_locale.as_deref());
        let (config, config_error) = match load_config(&store) {
            Ok(config) => (config, None),
            Err(e) => {
                log::warn!("Saved configuration ignored, default configuration used: {e:?}");
                (
                    default_config(language),
                    Some(AppError::new("config_ignored").with("cause", e)),
                )
            }
        };
        AppState {
            config,
            config_error,
            store,
            session: Session::default(),
            system_locale,
            language,
            native_texts: None,
        }
    }

    pub(crate) fn config_view(&self) -> ConfigView {
        ConfigView {
            config: self.config.clone(),
            error: self.config_error.clone(),
        }
    }

    pub(crate) fn language_view(&self) -> Result<LanguageView, AppError> {
        Ok(LanguageView {
            setting: self
                .store
                .language()
                .map_err(|e| AppError::internal("Language setting unreadable", e))?,
            language: self.language,
        })
    }

    pub(crate) fn store_language(&mut self, setting: LanguageSetting) -> Result<(), AppError> {
        self.store
            .set_language(setting)
            .map_err(|e| AppError::internal("Language not saved", e))?;
        self.language = setting.resolve(self.system_locale.as_deref());
        Ok(())
    }

    pub(crate) fn store_config(&mut self, config: Config) -> Result<ConfigView, AppError> {
        self.store
            .save_config(&config)
            .map_err(|e| AppError::internal("Configuration not saved", e))?;
        // The session's game was removed: the session is cancelled.
        if self.session.game_removed(&config) {
            self.cancel_session()?;
        }
        self.config = config;
        self.config_error = None;
        self.session.apply_config(&self.config, &mut rand::rng());
        self.save_session_summary();
        Ok(self.config_view())
    }

    /// Saves the events triggered and the jokers of the started session, for the history.
    pub(crate) fn save_session_summary(&mut self) {
        let Some(started_at) = self.session.started_at else {
            return;
        };
        let events = self.session.event_counts(&self.config);
        if let Err(e) = self
            .store
            .save_session_summary(started_at, &events, self.session.jokers())
        {
            log::error!("Session summary not saved: {e}");
        }
    }

    /// Removes the current session's sets from the history; the next one waits to be started.
    pub(crate) fn cancel_session(&mut self) -> Result<(), AppError> {
        if let Some(started_at) = self.session.started_at {
            self.store
                .remove_session(started_at)
                .map_err(|e| AppError::internal("History not modified", e))?;
        }
        self.session = Session::default();
        Ok(())
    }
}

fn load_config(store: &Store) -> Result<Config, AppError> {
    let mut config = store
        .config()
        .map_err(|e| AppError::internal("Saved configuration unreadable", e))?;
    if config.repair() {
        log::warn!("Saved configuration brought within the name and count limits");
    }
    config.validate()?;
    Ok(config)
}

pub(crate) fn now() -> i64 {
    Local::now().timestamp()
}

pub(crate) fn lock<'a>(state: &'a Shared) -> Result<MutexGuard<'a, AppState>, AppError> {
    state
        .lock()
        .map_err(|_| AppError::internal("Application state unavailable", "poisoned lock"))
}
