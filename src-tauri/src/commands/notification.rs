//! Notifications of launched Steam games.

use crate::app::AppError;
use crate::state::{lock, Shared};
#[cfg(target_os = "linux")]
use tauri::async_runtime::spawn_blocking;
use tauri::AppHandle;
#[cfg(target_os = "linux")]
use tauri::Manager;

/// Notification of a launched Steam game, with the texts sent by the interface (none before them):
/// `{name}` is replaced by the game's name, its body tells whether the game is already configured.
#[tauri::command]
pub(crate) async fn notify(
    app: AppHandle,
    state: Shared<'_>,
    name: String,
    configured: bool,
) -> Result<(), AppError> {
    let Some(texts) = lock(&state)?.native_texts.clone() else {
        return Ok(());
    };
    let body = if configured {
        texts.launched_configured
    } else {
        texts.launched_unknown
    };
    let fill = |template: &str| template.replace("{name}", &name);
    show_notification(app, fill(&texts.launched_title), fill(&body))
        .await
        .map_err(|e| AppError::internal("Notification not sent", e))
}

/// System notification. On Linux, the plugin would sign it with the executable name and a system
/// icon missing outside an installation: here it carries the app name and its icon.
async fn show_notification(app: AppHandle, title: String, body: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let icon = app
            .path()
            .app_cache_dir()
            .map_err(|e| e.to_string())?
            .join("icon.png");
        spawn_blocking(move || {
            std::fs::create_dir_all(icon.parent().expect("cache directory"))?;
            std::fs::write(&icon, include_bytes!("../../icons/128x128.png"))?;
            notify_rust::Notification::new()
                .appname(&app.package_info().name)
                .summary(&title)
                .body(&body)
                .icon(&icon.to_string_lossy())
                .show()
                .map(drop)
                .map_err(std::io::Error::other)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())
    }
    #[cfg(not(target_os = "linux"))]
    {
        use tauri_plugin_notification::NotificationExt;
        app.notification()
            .builder()
            .title(title)
            .body(body)
            .show()
            .map_err(|e| e.to_string())
    }
}
