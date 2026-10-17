//! Update: checked at launch when enabled, announced as soon as found, downloaded in the background
//! right away when automatic or else on request, downloaded again on request after a failure,
//! installed on quitting or on request. The `.deb` and `.rpm` packages are only told.

use crate::app::AppError;
use serde::Serialize;
use std::sync::{Mutex, MutexGuard};
use tauri::async_runtime::JoinHandle;
use tauri::utils::config::BundleType;
use tauri::utils::platform::bundle_type;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

/// Event telling the interface where the update stands, with its `UpdateView` (`null` once dropped).
const UPDATE_CHANGED: &str = "update-changed";

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum Stage {
    /// Found, downloaded only if the user asks for it.
    Available,
    Downloading,
    /// Download failed, downloaded again only if the user asks for it.
    Failed,
    Ready,
    /// To be downloaded by the user, from the release page.
    Manual,
}

/// Update as the interface shows it.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct UpdateView {
    version: String,
    /// Share downloaded, from 0 to 1, unknown while the size is.
    progress: Option<f64>,
    stage: Stage,
}

impl UpdateView {
    fn new(update: &Update, progress: Option<f64>, stage: Stage) -> Self {
        Self {
            version: update.version.clone(),
            progress,
            stage,
        }
    }
}

#[derive(Default)]
struct Pending {
    /// Bumped by each `apply`, so that a check dropped in the meantime changes nothing.
    generation: u64,
    check: Option<JoinHandle<()>>,
    view: Option<UpdateView>,
    /// Found or failed to download, waiting for the user to ask for its download.
    found: Option<Update>,
    downloaded: Option<(Update, Vec<u8>)>,
}

/// Update found, being downloaded or waiting to be installed.
#[derive(Default)]
pub(crate) struct PendingUpdate(Mutex<Pending>);

impl PendingUpdate {
    fn lock(&self) -> MutexGuard<'_, Pending> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub(crate) fn view(&self) -> Option<UpdateView> {
        self.lock().view.clone()
    }

    /// Taken out so that it is installed only once.
    fn take(&self) -> Option<(Update, Vec<u8>)> {
        self.lock().downloaded.take()
    }
}

/// The Linux packages other than the AppImage would need the system password to be installed: the
/// user installs them from the release page.
fn installable() -> bool {
    !cfg!(target_os = "linux") || bundle_type() == Some(BundleType::AppImage)
}

/// Enabled, looks for a newer version and, automatic, downloads it; disabled, drops the check, the
/// download and the downloaded update. Offline or on failure of the check, only a warning in the log.
/// The development build never looks for one.
pub(crate) fn apply(app: &AppHandle, enabled: bool, automatic: bool) {
    if cfg!(debug_assertions) {
        return;
    }
    let state = app.state::<PendingUpdate>();
    let mut pending = state.lock();
    if let Some(check) = pending.check.take() {
        check.abort();
    }
    let generation = pending.generation + 1;
    *pending = Pending {
        generation,
        ..Pending::default()
    };
    if enabled {
        let app = app.clone();
        pending.check = Some(tauri::async_runtime::spawn(async move {
            if let Err(e) = check(&app, generation, automatic).await {
                log::warn!("Update not checked: {e}");
                set_view(&app, generation, None);
            }
        }));
    } else {
        drop(pending);
        let _ = app.emit(UPDATE_CHANGED, None::<UpdateView>);
    }
}

/// Downloads the update found, or downloads it again after a failure, at the user's request.
pub(crate) fn download_found(app: &AppHandle) {
    let state = app.state::<PendingUpdate>();
    let mut pending = state.lock();
    let Some(update) = pending.found.take() else {
        return;
    };
    let generation = pending.generation;
    let app = app.clone();
    pending.check = Some(tauri::async_runtime::spawn(async move {
        download(&app, generation, update).await
    }));
}

/// Runs `change` and tells the interface, unless the check was dropped in the meantime.
fn update_pending(app: &AppHandle, generation: u64, change: impl FnOnce(&mut Pending)) {
    let state = app.state::<PendingUpdate>();
    let mut pending = state.lock();
    if pending.generation != generation {
        return;
    }
    change(&mut pending);
    let _ = app.emit(UPDATE_CHANGED, pending.view.clone());
}

fn set_view(app: &AppHandle, generation: u64, view: Option<UpdateView>) {
    update_pending(app, generation, |pending| pending.view = view);
}

async fn check(
    app: &AppHandle,
    generation: u64,
    automatic: bool,
) -> tauri_plugin_updater::Result<()> {
    let Some(update) = app.updater()?.check().await? else {
        return Ok(());
    };
    if !installable() {
        set_view(
            app,
            generation,
            Some(UpdateView::new(&update, None, Stage::Manual)),
        );
    } else if automatic {
        download(app, generation, update).await;
    } else {
        update_pending(app, generation, |pending| {
            pending.view = Some(UpdateView::new(&update, None, Stage::Available));
            pending.found = Some(update);
        });
    }
    Ok(())
}

/// On failure, a warning in the log, and the update waits for the user to ask for it again.
async fn download(app: &AppHandle, generation: u64, update: Update) {
    let view = |progress, stage| Some(UpdateView::new(&update, progress, stage));
    set_view(app, generation, view(None, Stage::Downloading));
    let mut received = 0u64;
    // Announced at each whole percent only, so that the interface is not flooded.
    let mut percent = 0;
    let downloaded = update
        .download(
            |chunk, total| {
                received += chunk as u64;
                let Some(total) = total.filter(|&t| t > 0) else {
                    return;
                };
                let now = (received * 100 / total).min(100);
                if now != percent {
                    percent = now;
                    set_view(
                        app,
                        generation,
                        view(Some(now as f64 / 100.0), Stage::Downloading),
                    );
                }
            },
            || {},
        )
        .await;
    match downloaded {
        Ok(bytes) => {
            let ready = view(Some(1.0), Stage::Ready);
            update_pending(app, generation, |pending| {
                pending.downloaded = Some((update, bytes));
                pending.view = ready;
            });
        }
        Err(e) => {
            log::warn!("Update not downloaded: {e}");
            let failed = view(None, Stage::Failed);
            update_pending(app, generation, |pending| {
                pending.found = Some(update);
                pending.view = failed;
            });
        }
    }
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
