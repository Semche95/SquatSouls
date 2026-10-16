//! Native texts, sent by the interface, and quitting from the notification area.

use crate::app::AppError;
use crate::state::{lock, NativeTexts, Shared};
use crate::tray;
use tauri::AppHandle;

/// Texts of the tray menu and notifications, sent by the interface at launch and on each language change.
#[tauri::command]
pub(crate) fn set_native_texts(
    app: AppHandle,
    state: Shared,
    texts: NativeTexts,
) -> Result<(), AppError> {
    let mut s = lock(&state)?;
    s.native_texts = Some(texts);
    tray::update_texts(&app, &s)
        .map_err(|e| AppError::logged("tray_unavailable", "Notification area unavailable", e))
}

/// Quits the app completely, even when closing the window only hides it; confirmed during a session.
#[tauri::command]
pub(crate) fn request_quit(app: AppHandle) {
    tray::request_quit(&app);
}

/// Quits once the interface has confirmed it.
#[tauri::command]
pub(crate) fn quit(app: AppHandle) {
    app.exit(0);
}
