//! Notification area icon, and window hidden rather than closed, depending on the configuration.

use crate::fitness::Config;
use crate::state::AppState;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Window, WindowEvent};

const TRAY_ID: &str = "main";
const SHOW: &str = "show";
const QUIT: &str = "quit";

/// Shows the main window in the foreground.
pub(crate) fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Adds or removes the icon depending on the configuration.
pub(crate) fn apply(app: &AppHandle, config: &Config) -> tauri::Result<()> {
    match (config.tray, app.tray_by_id(TRAY_ID)) {
        (true, None) => build(app),
        (false, Some(_)) => {
            app.remove_tray_by_id(TRAY_ID);
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Click on the icon: shows the window. Menu: Show and Quit (the only way to the menu on Linux).
fn build(app: &AppHandle) -> tauri::Result<()> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, SHOW, "Afficher", true, None::<&str>)?,
            &MenuItem::with_id(app, QUIT, "Quitter", true, None::<&str>)?,
        ],
    )?;
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Squat Souls")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW => show_main(app),
            QUIT => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// With the icon and the setting, closing the window hides it: the app and detection keep running.
pub(crate) fn on_window_event(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    let state = window.state::<Mutex<AppState>>();
    let hide = state
        .lock()
        .is_ok_and(|s| s.config.tray && s.config.close_to_tray);
    if hide {
        api.prevent_close();
        let _ = window.hide();
    }
}
