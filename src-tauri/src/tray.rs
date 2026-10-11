//! Icône dans la zone de notification, et fenêtre masquée plutôt que fermée, selon la configuration.

use crate::fitness::Config;
use crate::state::AppState;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Window, WindowEvent};

const TRAY_ID: &str = "main";
const SHOW: &str = "show";
const QUIT: &str = "quit";

/// Affiche la fenêtre principale au premier plan.
pub(crate) fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Ajoute ou retire l'icône selon la configuration.
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

/// Clic sur l'icône : affiche la fenêtre. Menu : Afficher et Quitter (seul accès au menu sous Linux).
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

/// Avec l'icône et le réglage, fermer la fenêtre la masque : l'appli et la détection continuent.
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
