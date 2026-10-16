//! Downloaded update: its version and its installation.

use crate::app::AppError;
use crate::update::{self, PendingUpdate};
use tauri::{AppHandle, State};

/// Version of the downloaded update waiting to be installed, if any.
#[tauri::command]
pub(crate) fn get_update(pending: State<'_, PendingUpdate>) -> Option<String> {
    pending.version()
}

/// Installs the downloaded update right away and restarts the app.
#[tauri::command]
pub(crate) fn restart_to_update(app: AppHandle) -> Result<(), AppError> {
    update::install_and_restart(&app)
}
