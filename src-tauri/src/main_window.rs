//! Main window, created at launch from its configuration, hidden until shown. It keeps its size, position
//! and maximized state from one launch to the next, except in the minimal mode, whose small centered window
//! would replace them.

use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

/// Label of the window, in its configuration.
pub(crate) const LABEL: &str = "main";

/// File of the size, position and maximized state, in the configuration folder. The development build has
/// its own, like its database.
const STATE_FILE: &str = if cfg!(debug_assertions) {
    "squatsouls-dev.window.json"
} else {
    "squatsouls.window.json"
};

/// What the window keeps from one launch to the next.
const FLAGS: StateFlags = StateFlags::SIZE
    .union(StateFlags::POSITION)
    .union(StateFlags::MAXIMIZED);

/// Present once the window keeps its state: absent in the minimal mode.
struct Remembered;

/// Creates the main window. With `remember`, it takes back the size, position and maximized state it had when
/// the app last quit, and saves them when it quits. A position on a screen no longer connected is left out.
pub(crate) fn create(app: &AppHandle, remember: bool) -> tauri::Result<WebviewWindow> {
    // Registered before the window exists: the plugin restores and follows only the windows created after it.
    if remember {
        app.plugin(
            tauri_plugin_window_state::Builder::new()
                .with_filename(STATE_FILE)
                .with_state_flags(FLAGS)
                .build(),
        )?;
        app.manage(Remembered);
    }
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == LABEL)
        .ok_or(tauri::Error::WindowNotFound)?;
    WebviewWindowBuilder::from_config(app, config)?.build()
}

/// Saves the size, position and maximized state now, for a quit that skips the app's exit (restart to update);
/// nothing in the minimal mode.
pub(crate) fn save(app: &AppHandle) {
    if app.try_state::<Remembered>().is_none() {
        return;
    }
    if let Err(e) = app.save_window_state(FLAGS) {
        log::error!("Window state not saved: {e}");
    }
}
