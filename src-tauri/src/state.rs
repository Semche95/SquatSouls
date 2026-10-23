//! Shared application state: configuration loading, and the session, saved at each change and resumed at launch.

use crate::app::{
    default_config, AppError, Config, EndCause, Language, LanguageSetting, Session, SessionEnd,
    Store,
};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};
use tauri::{AppHandle, Manager, State};

pub struct AppState {
    pub config: Config,
    /// Error met while loading the configuration (the default config is then used).
    pub(crate) config_error: Option<AppError>,
    pub store: Store,
    /// Changed only through `change_session` and the configuration, which save it.
    session: Session,
    /// System locale, read at launch to resolve the `System` language setting.
    pub(crate) system_locale: Option<String>,
    pub(crate) language: Language,
    /// Texts of the tray menu and notifications, translated by the interface: none until it sends them.
    pub(crate) native_texts: Option<NativeTexts>,
}

pub(crate) type Shared<'a> = State<'a, Mutex<AppState>>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigView {
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
    pub fn new(store: Store, system_locale: Option<String>) -> Self {
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
        let mut state = AppState {
            config,
            config_error,
            store,
            session: Session::default(),
            system_locale,
            language,
            native_texts: None,
        };
        state.resume_session();
        state
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// Refuses to install an update during a session: a failed update could keep the app, and the session, from
    /// opening again.
    pub fn allow_update(&self) -> Result<(), AppError> {
        match self.session.started_at {
            Some(_) => Err(AppError::new("update_during_session")),
            None => Ok(()),
        }
    }

    /// Resumes the session in progress when the app closed, without the time it stayed closed, counting again what
    /// it asked if it was saved before that was counted. Under the default
    /// configuration, used in place of an unreadable one, it adjusts to it as to a saved configuration.
    fn resume_session(&mut self) {
        match self.store.current_session() {
            Ok(Some((mut session, closed))) => {
                session.resume(closed, now());
                session.recount_asked(&self.config);
                self.session = session;
                if self.config_error.is_some() {
                    let config = self.config.clone();
                    if let Err(e) = self.adjust_session(config) {
                        log::error!("Session not adjusted to the configuration: {e:?}");
                    }
                } else {
                    self.save_session();
                }
            }
            Ok(None) => {}
            Err(e) => {
                log::error!("Session in progress unreadable: {e}");
                if let Err(e) = self.store.end_unreadable_session() {
                    log::error!("Unreadable session not ended: {e}");
                }
                self.session = Session::after(SessionEnd {
                    saved: false,
                    cause: Some(EndCause::Unreadable),
                });
            }
        }
    }

    /// Changes the session, then saves it, so that nothing of it is lost if the app closes.
    pub fn change_session<T>(
        &mut self,
        change: impl FnOnce(&mut Session, &Config) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let changed = change(&mut self.session, &self.config);
        self.save_session();
        changed
    }

    /// Saves at once the started session as it is now, its sets done not saved yet, the events and jokers of its
    /// summary, and its duration.
    fn save_session(&mut self) {
        let Some(started_at) = self.session.started_at else {
            return;
        };
        let events = self.session.event_counts(&self.config);
        let jokers = self.session.jokers();
        let session = &self.session;
        let saved = self.store.atomically(|store| {
            store.append_history(&session.done[session.recorded..])?;
            store.save_session_summary(started_at, &events, jokers)?;
            store.save_current_session(session, now())
        });
        match saved {
            Ok(()) => self.session.recorded = self.session.done.len(),
            Err(e) => log::error!("Session not saved: {e}"),
        }
    }

    /// Records that the app is still open during a session, so that a crash loses little of its duration.
    pub(crate) fn mark_seen(&self) {
        if self.session.started_at.is_some() {
            if let Err(e) = self.store.mark_seen(now()) {
                log::warn!("Session time not saved: {e}");
            }
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

    pub fn store_config(&mut self, config: Config) -> Result<ConfigView, AppError> {
        self.store
            .save_config(&config)
            .map_err(|e| AppError::internal("Configuration not saved", e))?;
        self.adjust_session(config)?;
        self.config_error = None;
        Ok(self.config_view())
    }

    /// Applies `config` to the session, which ends there if it can no longer be played on it.
    fn adjust_session(&mut self, config: Config) -> Result<(), AppError> {
        if let Some(cause) = self.session.end_cause(&config) {
            self.end_unplayable_session(cause)?;
        }
        self.config = config;
        self.session.apply_config(&self.config, &mut rand::rng());
        self.save_session();
        Ok(())
    }

    /// The session's game is removed or no exercise is included: the session ends there, keeping only the exercises
    /// done, with the summary of the configuration it was played with; without any, it is cancelled. Either way,
    /// the next session tells why.
    fn end_unplayable_session(&mut self, cause: EndCause) -> Result<(), AppError> {
        if self.session.dismiss_all() {
            self.save_session();
            self.end_session(Some(cause));
        } else {
            self.cancel_session()?;
            self.session = Session::after(SessionEnd {
                saved: false,
                cause: Some(cause),
            });
        }
        Ok(())
    }

    /// Saves the end of the started session, by itself for `cause` or else with **End the session**; the next one
    /// waits to be started.
    pub fn end_session(&mut self, cause: Option<EndCause>) {
        if let Some(started_at) = self.session.started_at {
            let ended = self.store.atomically(|store| {
                store.end_session(started_at, now())?;
                store.clear_current_session()
            });
            if let Err(e) = ended {
                log::error!("Session end not saved: {e}");
            }
        }
        self.session = Session::after(SessionEnd { saved: true, cause });
    }

    /// Removes the current session's sets from the history; the next one waits to be started.
    pub fn cancel_session(&mut self) -> Result<(), AppError> {
        if let Some(started_at) = self.session.started_at {
            self.store
                .atomically(|store| {
                    store.remove_session(started_at)?;
                    store.clear_current_session()
                })
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

/// True while a session is in progress; without state (minimal window at launch), never.
pub(crate) fn in_session(app: &AppHandle) -> bool {
    app.try_state::<Mutex<AppState>>()
        .is_some_and(|state| state.lock().is_ok_and(|s| s.session.started_at.is_some()))
}

pub(crate) fn lock<'a>(state: &'a Shared) -> Result<MutexGuard<'a, AppState>, AppError> {
    state
        .lock()
        .map_err(|_| AppError::internal("Application state unavailable", "poisoned lock"))
}
