//! Catalog of games: fetched from the latest release, in the effective language.

use crate::app::{catalog_url, read_catalog, AppError, MAX_CATALOG_SIZE};
use crate::state::{lock, AppState};
use std::sync::Mutex;
use std::time::Duration;
use tauri::async_runtime::spawn_blocking;
use tauri::{AppHandle, Manager};

/// Maximum duration of the catalog download.
const CATALOG_TIMEOUT: Duration = Duration::from_secs(10);

fn fetch(url: &str) -> Result<String, AppError> {
    let unavailable =
        |e: ureq::Error| AppError::logged("catalog_unavailable", "Catalog not fetched", e);
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(CATALOG_TIMEOUT))
        .build()
        .into();
    agent
        .get(url)
        .call()
        .map_err(unavailable)?
        .body_mut()
        .with_config()
        .limit(MAX_CATALOG_SIZE)
        .read_to_string()
        .map_err(unavailable)
}

/// Share texts of the games of the catalog, read off the main thread.
#[tauri::command]
pub(crate) async fn game_catalog(app: AppHandle) -> Result<Vec<String>, AppError> {
    let language = lock(&app.state::<Mutex<AppState>>())?.language;
    let url = catalog_url(env!("CARGO_PKG_REPOSITORY"), language);
    spawn_blocking(move || read_catalog(&fetch(&url)?))
        .await
        .map_err(|e| AppError::internal("Catalog download interrupted", e))?
}
