//! Opening of the database at launch, told to the interface: restored from the copy of the previous
//! launch, or the minimal mode when it cannot be opened, where the update is still looked for. The copy
//! is then renewed regularly while the app runs.

use crate::app::{AppError, Language, LanguageSetting, LaunchError, Launched, OpenFailure, Store};
use crate::state::AppState;
use crate::{tray, update};
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, LogicalSize, Manager};

/// Interval between two copies of the database while the app runs.
const COPY_INTERVAL: Duration = Duration::from_secs(60 * 60);

/// Size of the minimal window, just enough for the reason and the update banner.
const WIDTH: f64 = 640.0;
const HEIGHT: f64 = 360.0;

/// Why the database could not be opened, with the language of the system: the saved one is in it.
#[derive(Clone, Serialize)]
pub(crate) struct StartupFailure {
    error: AppError,
    language: Language,
}

/// The damaged database was replaced by the copy of the previous launch.
pub(crate) struct Restored(pub(crate) bool);

/// Opens the database, or enters the minimal mode and gives nothing.
pub(crate) fn open(app: &AppHandle, system_locale: Option<&str>) -> Option<Store> {
    match launch(app, system_locale) {
        Ok(Launched {
            store,
            restored,
            copy_error,
        }) => {
            if let Some(e) = copy_error {
                log::warn!("Database not copied for the next launch: {e}");
            }
            if restored {
                log::error!("Database damaged, replaced by the copy of the previous launch");
            }
            app.manage(Restored(restored));
            Some(store)
        }
        Err(e) => {
            fail(app, &e, system_locale);
            None
        }
    }
}

/// The development build has its own database so it never touches real data.
fn launch(app: &AppHandle, system_locale: Option<&str>) -> Result<Launched, LaunchError> {
    let dir = app.path().app_config_dir().map_err(|e| LaunchError {
        failure: OpenFailure::Denied,
        cause: e.to_string(),
    })?;
    std::fs::create_dir_all(&dir)?;
    let db = if cfg!(debug_assertions) {
        "squatsouls-dev.db"
    } else {
        "squatsouls.db"
    };
    Store::launch(&dir.join(db), system_locale)
}

/// Enters the minimal mode. The database is left as it is. The update setting is in it too: the
/// update is looked for as when the setting is on, its default.
fn fail(app: &AppHandle, e: &LaunchError, system_locale: Option<&str>) {
    log::error!("Database unusable: {e}");
    app.manage(StartupFailure {
        error: AppError::new(e.failure.code()),
        language: LanguageSetting::System.resolve(system_locale),
    });
    if let Some(window) = app.get_webview_window("main") {
        let size = LogicalSize::new(WIDTH, HEIGHT);
        let _ = window.set_min_size(Some(size));
        let _ = window.set_size(size);
        let _ = window.center();
    }
    tray::show_main(app);
    update::apply(app, true);
}

/// Renews the copy of the database at regular intervals, for as long as the app runs.
pub(crate) fn copy_regularly(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(COPY_INTERVAL);
        let state = app.state::<Mutex<AppState>>();
        let copied = match state.lock() {
            Ok(s) => s.store.save_copy(),
            Err(_) => continue,
        };
        if let Err(e) = copied {
            log::warn!("Database not copied: {e}");
        }
    });
}
