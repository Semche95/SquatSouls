//! Configuration: reading and saving.

use crate::app::{AppError, Config};
use crate::state::{lock, ConfigView, Shared};
use crate::{tray, update};
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
    let update = (config.update_check, config.update_auto);
    let update_changed = update != (s.config.update_check, s.config.update_auto);
    let view = s.store_config(config)?;
    if update_changed {
        update::apply(&app, update.0, update.1);
    }
    tray::apply(&app, &s).map_err(|e| {
        AppError::logged(
            "tray_unavailable",
            "Configuration saved, notification area unavailable",
            e,
        )
    })?;
    Ok(view)
}
