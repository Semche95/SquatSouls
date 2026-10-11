mod commands;
pub mod fitness;
mod state;
mod tray;

use fitness::Store;
use state::AppState;
use std::fs;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // WebKitGTK crashes under Wayland (clipboard, DMABUF rendering): go through XWayland
    // without DMABUF rendering. A variable already set at launch is respected, so Wayland can be
    // restored (`GDK_BACKEND=wayland`). Set before any thread, as set_var is not thread-safe.
    #[cfg(target_os = "linux")]
    for (key, value) in [
        ("GDK_BACKEND", "x11"),
        ("WEBKIT_DISABLE_DMABUF_RENDERER", "1"),
    ] {
        if std::env::var_os(key).is_none() {
            std::env::set_var(key, value);
        }
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            fs::create_dir_all(&dir)?;
            // The development build has its own database so it never touches real data
            let db = if cfg!(debug_assertions) { "squatsouls-dev.db" } else { "squatsouls.db" };
            let store = Store::open(&dir.join(db))?;
            app.manage(Mutex::new(store.steam_names()?));
            let state = AppState::new(store);
            tray::apply(app.handle(), &state.config)?;
            if !(state.config.tray && state.config.start_hidden) {
                tray::show_main(app.handle());
            }
            app.manage(Mutex::new(state));
            Ok(())
        })
        .on_window_event(tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::draw,
            commands::complete_exercise,
            commands::dismiss,
            commands::halve,
            commands::skip_exercise,
            commands::skip_set,
            commands::get_session,
            commands::new_session,
            commands::cancel_session,
            commands::get_last_game,
            commands::set_last_game,
            commands::get_history,
            commands::get_full_history,
            commands::running_steam_app,
            commands::steam_game_name,
            commands::notify
        ])
        .run(tauri::generate_context!())
        .expect("error while launching Squat Souls");
}
