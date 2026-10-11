mod commands;
pub mod fitness;
mod state;

use fitness::Store;
use state::AppState;
use std::fs;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            fs::create_dir_all(&dir)?;
            let store = Store::open(&dir.join("squatsouls.db"))?;
            app.manage(Mutex::new(AppState::new(store)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::draw,
            commands::complete,
            commands::dismiss,
            commands::use_joker,
            commands::get_session,
            commands::new_session,
            commands::cancel_session,
            commands::get_last_game,
            commands::set_last_game,
            commands::get_history,
            commands::get_full_history
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement de Squat Souls");
}
