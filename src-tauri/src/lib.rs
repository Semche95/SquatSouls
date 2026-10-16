pub mod app;
mod commands;
mod startup;
mod state;
mod tray;
mod update;

use state::AppState;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
use std::sync::Mutex;
#[cfg(target_os = "linux")]
use tauri::utils::{config::BundleType, platform::bundle_type};
use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};
use update::PendingUpdate;

/// Size beyond which the log file is archived, and number of archives kept.
const LOG_MAX_BYTES: u128 = 1_000_000;
const LOG_ARCHIVES: usize = 3;

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
    let context = tauri::generate_context!();
    // The AppImage bundles its own GStreamer plugins: kept in the GStreamer cache of the system, they
    // would replace its list, rescanned by every other GStreamer app. The AppImage gets a cache of its own.
    #[cfg(target_os = "linux")]
    if bundle_type() == Some(BundleType::AppImage) && std::env::var_os("GST_REGISTRY_1_0").is_none()
    {
        let cache = std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")));
        if let Some(cache) = cache {
            let registry = cache
                .join(&context.config().identifier)
                .join("gstreamer-registry.bin");
            std::env::set_var("GST_REGISTRY_1_0", registry);
        }
    }
    // A panic is logged too, before the usual report.
    let report = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!("{info}");
        report(info);
    }));
    let builder = tauri::Builder::default();
    // A second launch, the window being hidden in the notification area for instance, shows the one
    // already running instead of fighting it for the database. Registered first, so that it quits before
    // anything else starts. Not in the development build, which shares the installed app's identifier:
    // both can run side by side.
    #[cfg(not(debug_assertions))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
        tray::show_main(app)
    }));
    builder
        // Warnings and errors, on the terminal and in the app's log folder. The development build has
        // its own file, like its database.
        .plugin(
            tauri_plugin_log::Builder::new()
                .clear_targets()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some(
                            if cfg!(debug_assertions) {
                                "squatsouls-dev"
                            } else {
                                "squatsouls"
                            }
                            .into(),
                        ),
                    }),
                ])
                .level(log::LevelFilter::Warn)
                .timezone_strategy(TimezoneStrategy::UseLocal)
                .max_file_size(LOG_MAX_BYTES)
                .rotation_strategy(RotationStrategy::KeepSome(LOG_ARCHIVES))
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let system_locale = sys_locale::get_locale();
            app.manage(PendingUpdate::default());
            let Some(store) = startup::open(app.handle(), system_locale.as_deref()) else {
                return Ok(());
            };
            app.manage(Mutex::new(store.steam_names()?));
            // The tray icon waits for the interface's texts (`set_native_texts`).
            let state = AppState::new(store, system_locale);
            if !(state.config.tray && state.config.start_hidden) {
                tray::show_main(app.handle());
            }
            let update_check = state.config.update_check;
            app.manage(Mutex::new(state));
            startup::copy_regularly(app.handle());
            update::apply(app.handle(), update_check);
            Ok(())
        })
        .on_window_event(tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_config,
            commands::get_language,
            commands::set_language,
            commands::set_native_texts,
            commands::request_quit,
            commands::quit,
            commands::get_startup_failure,
            commands::was_restored,
            commands::relaunch,
            commands::get_update,
            commands::restart_to_update,
            commands::draw,
            commands::set_session_intensity,
            commands::complete_exercise,
            commands::dismiss,
            commands::halve,
            commands::skip_exercise,
            commands::skip_set,
            commands::get_session,
            commands::start_session,
            commands::end_session,
            commands::cancel_session,
            commands::get_last_game,
            commands::set_last_game,
            commands::get_intensity,
            commands::set_intensity,
            commands::get_onboarding_seen,
            commands::set_onboarding_seen,
            commands::get_history,
            commands::get_full_history,
            commands::count_active_days,
            commands::running_steam_app,
            commands::steam_game_name,
            commands::notify
        ])
        .build(context)
        .expect("error while launching Squat Souls")
        .run(tray::on_run_event);
}
