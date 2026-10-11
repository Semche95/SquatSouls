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
    // WebKitGTK plante sous Wayland (presse-papiers, rendu DMABUF) : passer par XWayland
    // sans rendu DMABUF. Fixé avant tout fil, set_var n'étant pas sûr en multi-fil.
    #[cfg(target_os = "linux")]
    for (key, value) in [
        ("GDK_BACKEND", "x11"),
        ("WEBKIT_DISABLE_DMABUF_RENDERER", "1"),
    ] {
        std::env::set_var(key, value);
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            fs::create_dir_all(&dir)?;
            // Le build de développement a sa propre base pour ne pas toucher aux données réelles
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
        .expect("erreur au lancement de Squat Souls");
}
