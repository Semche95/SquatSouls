//! Notification area icon, window hidden rather than closed depending on the configuration, and quitting,
//! confirmed by the interface during a session, installing the downloaded update.

use crate::state::{AppState, NativeTexts};
use crate::update;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, RunEvent, Window, WindowEvent};

const TRAY_ID: &str = "main";
const SHOW: &str = "show";
const QUIT: &str = "quit";
/// Event asking the interface to confirm quitting.
const QUIT_REQUESTED: &str = "quit-requested";

/// Shows the main window in the foreground.
pub(crate) fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Without state (minimal mode at launch), never.
fn in_session(app: &AppHandle) -> bool {
    app.try_state::<Mutex<AppState>>()
        .is_some_and(|state| state.lock().is_ok_and(|s| s.session.started_at.is_some()))
}

/// Shows the window with the quit confirmation.
fn confirm_quit(app: &AppHandle) {
    show_main(app);
    let _ = app.emit(QUIT_REQUESTED, ());
}

/// Quits right away without a session in progress, otherwise lets the interface confirm.
pub(crate) fn request_quit(app: &AppHandle) {
    if in_session(app) {
        confirm_quit(app);
    } else {
        app.exit(0);
    }
}

/// Quitting by the system (Cmd+Q in the macOS menu) during a session waits for the confirmation.
/// `app.exit` gives a code and is never held back. Once quitting, the downloaded update is installed.
pub(crate) fn on_run_event(app: &AppHandle, event: RunEvent) {
    match event {
        RunEvent::ExitRequested {
            code: None, api, ..
        } if in_session(app) => {
            api.prevent_exit();
            confirm_quit(app);
        }
        RunEvent::Exit => update::install_on_exit(app),
        _ => {}
    }
}

/// Adds or removes the icon depending on the configuration. No icon until the interface has sent its texts.
pub(crate) fn apply(app: &AppHandle, state: &AppState) -> tauri::Result<()> {
    let texts = state.native_texts.as_ref().filter(|_| state.config.tray);
    match (texts, app.tray_by_id(TRAY_ID)) {
        (Some(texts), None) => build(app, texts),
        (None, Some(_)) => {
            app.remove_tray_by_id(TRAY_ID);
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Gives the existing icon's menu the texts just received. The icon is kept rather than rebuilt: on Linux,
/// an icon removed then recreated under the same id collides with the old one on D-Bus and disappears.
pub(crate) fn update_texts(app: &AppHandle, state: &AppState) -> tauri::Result<()> {
    match (&state.native_texts, app.tray_by_id(TRAY_ID)) {
        (Some(texts), Some(tray)) => tray.set_menu(Some(menu(app, texts)?)),
        _ => apply(app, state),
    }
}

/// Menu: Show and Quit (the only way to the menu on Linux).
fn menu(app: &AppHandle, texts: &NativeTexts) -> tauri::Result<Menu<tauri::Wry>> {
    Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, SHOW, &texts.show, true, None::<&str>)?,
            &MenuItem::with_id(app, QUIT, &texts.quit, true, None::<&str>)?,
        ],
    )
}

/// Click on the icon: shows the window.
fn build(app: &AppHandle, texts: &NativeTexts) -> tauri::Result<()> {
    let mut tray = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Squat Souls")
        .menu(&menu(app, texts)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            SHOW => show_main(app),
            QUIT => request_quit(app),
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
/// Otherwise, closing it during a session waits for the quit confirmation.
pub(crate) fn on_window_event(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    let hide = window.try_state::<Mutex<AppState>>().is_some_and(|state| {
        state
            .lock()
            .is_ok_and(|s| s.config.tray && s.config.close_to_tray)
    });
    if hide {
        api.prevent_close();
        let _ = window.hide();
    } else if in_session(window.app_handle()) {
        api.prevent_close();
        confirm_quit(window.app_handle());
    }
}
