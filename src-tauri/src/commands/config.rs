//! Configuration: reading and saving.

use crate::app::{AppError, Config};
use crate::state::{lock, ConfigView, Shared};
use crate::tray;
use tauri::AppHandle;

#[tauri::command]
pub(crate) fn get_config(state: Shared) -> Result<ConfigView, AppError> {
    Ok(lock(&state)?.config_view())
}

#[tauri::command]
pub(crate) fn save_config(
    app: AppHandle,
    state: Shared,
    config: Config,
) -> Result<ConfigView, AppError> {
    config.validate()?;
    let mut s = lock(&state)?;
    let view = s.store_config(config)?;
    tray::apply(&app, &s).map_err(|e| {
        AppError::logged(
            "tray_unavailable",
            "Configuration saved, notification area unavailable",
            e,
        )
    })?;
    Ok(view)
}
