//! Automatic update: where it stands and its installation.

use crate::app::AppError;
use crate::update::{self, PendingUpdate, UpdateView};
use tauri::{AppHandle, State};

/// Update found, being downloaded or waiting to be installed, if any.
#[tauri::command]
pub(crate) fn get_update(pending: State<'_, PendingUpdate>) -> Option<UpdateView> {
    pending.view()
}

/// Installs the downloaded update right away and restarts the app.
#[tauri::command]
pub(crate) fn restart_to_update(app: AppHandle) -> Result<(), AppError> {
    update::install_and_restart(&app)
}
