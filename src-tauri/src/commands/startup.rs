//! Opening of the database at launch: why it failed, whether it was restored, and launching again.

use crate::startup::{Restored, StartupFailure};
use tauri::{AppHandle, Manager};

/// The failure, in the minimal mode only.
#[tauri::command]
pub(crate) fn get_startup_failure(app: AppHandle) -> Option<StartupFailure> {
    app.try_state::<StartupFailure>().map(|f| f.inner().clone())
}

/// Whether the damaged database was replaced by the copy of the previous launch.
#[tauri::command]
pub(crate) fn was_restored(app: AppHandle) -> bool {
    app.try_state::<Restored>().is_some_and(|r| r.0)
}

/// Launches the app again, once the cause of the failure is fixed.
#[tauri::command]
pub(crate) fn relaunch(app: AppHandle) {
    app.restart()
}
