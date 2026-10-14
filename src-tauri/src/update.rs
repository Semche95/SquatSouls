//! Automatic update: checked and downloaded in the background at launch, installed on quitting or on request.

use crate::app::AppError;
use std::sync::Mutex;
use tauri::utils::platform::bundle_type;
use tauri::utils::config::BundleType;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

/// Event telling the interface that an update is downloaded, with its version.
const UPDATE_READY: &str = "update-ready";

/// Downloaded update, waiting to be installed.
#[derive(Default)]
pub(crate) struct PendingUpdate(Mutex<Option<(Update, Vec<u8>)>>);

impl PendingUpdate {
    pub(crate) fn version(&self) -> Option<String> {
        let pending = self.0.lock().ok()?;
        pending.as_ref().map(|(update, _)| update.version.clone())
    }

    fn set(&self, update: Update, bytes: Vec<u8>) {
        if let Ok(mut pending) = self.0.lock() {
            *pending = Some((update, bytes));
        }
    }

    /// Taken out so that it is installed only once.
    fn take(&self) -> Option<(Update, Vec<u8>)> {
        self.0.lock().ok()?.take()
    }
}

/// The development build is never updated, nor the Linux packages other than the AppImage, which are
/// updated by their package manager.
fn updatable() -> bool {
    !cfg!(debug_assertions)
        && (!cfg!(target_os = "linux") || bundle_type() == Some(BundleType::AppImage))
}

/// Looks for a newer version and downloads it. Offline or on failure, only a warning in the log.
pub(crate) fn check(app: &AppHandle) {
    if !updatable() {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = download(&app).await {
            log::warn!("Update not downloaded: {e}");
        }
    });
}

async fn download(app: &AppHandle) -> tauri_plugin_updater::Result<()> {
    let Some(update) = app.updater()?.check().await? else {
        return Ok(());
    };
    let bytes = update.download(|_, _| {}, || {}).await?;
    let version = update.version.clone();
    app.state::<PendingUpdate>().set(update, bytes);
    let _ = app.emit(UPDATE_READY, version);
    Ok(())
}

/// On quitting: installs the downloaded update without restarting the app.
pub(crate) fn install_on_exit(app: &AppHandle) {
    if let Some((update, bytes)) = app.state::<PendingUpdate>().take() {
        if let Err(e) = update.restart_after_install(false).install(bytes) {
            log::error!("Update not installed: {e}");
        }
    }
}

/// Installs the downloaded update and restarts the app on the new version.
pub(crate) fn install_and_restart(app: &AppHandle) -> Result<(), AppError> {
    let (update, bytes) = app
        .state::<PendingUpdate>()
        .take()
        .ok_or_else(|| AppError::internal("Update not installed", "no update downloaded"))?;
    update
        .install(bytes)
        .map_err(|e| AppError::internal("Update not installed", e))?;
    app.restart()
}
