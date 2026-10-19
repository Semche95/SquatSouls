//! Time of the session in progress, recorded every minute while the app runs, so that after a crash its duration
//! counts until then.

use crate::state::AppState;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// Interval between two records: the most a crash takes from the session's duration.
const INTERVAL: Duration = Duration::from_secs(60);

/// Records the time of the session in progress at regular intervals, for as long as the app runs.
pub(crate) fn record_regularly(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(INTERVAL);
        if let Ok(s) = app.state::<Mutex<AppState>>().lock() {
            s.mark_seen();
        }
    });
}
