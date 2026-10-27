//! History of the sets done.

use crate::app::{AppError, DayTotals};
use crate::state::{lock, Shared};
use chrono::{Local, NaiveDate};

const HISTORY_UNREADABLE: &str = "History unreadable";

#[tauri::command]
pub(crate) fn get_history(state: Shared, days: u32) -> Result<Vec<DayTotals>, AppError> {
    lock(&state)?
        .store
        .history(days, Local::now().date_naive())
        .map_err(|e| AppError::internal(HISTORY_UNREADABLE, e))
}

#[tauri::command]
pub(crate) fn get_full_history(
    state: Shared,
    before: Option<String>,
    days: usize,
    game: Option<String>,
) -> Result<Vec<DayTotals>, AppError> {
    let before = before
        .map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d"))
        .transpose()
        .map_err(|e| AppError::internal(HISTORY_UNREADABLE, e))?;
    lock(&state)?
        .store
        .full_history(before, days, game.as_deref())
        .map_err(|e| AppError::internal(HISTORY_UNREADABLE, e))
}

#[tauri::command]
pub(crate) fn count_active_days(state: Shared, game: Option<String>) -> Result<usize, AppError> {
    lock(&state)?
        .store
        .active_day_count(game.as_deref())
        .map_err(|e| AppError::internal(HISTORY_UNREADABLE, e))
}

#[tauri::command]
pub(crate) fn get_history_months(
    state: Shared,
    game: Option<String>,
) -> Result<Vec<String>, AppError> {
    lock(&state)?
        .store
        .history_months(game.as_deref())
        .map_err(|e| AppError::internal(HISTORY_UNREADABLE, e))
}

#[tauri::command]
pub(crate) fn get_history_games(state: Shared) -> Result<Vec<String>, AppError> {
    lock(&state)?
        .store
        .history_games()
        .map_err(|e| AppError::internal(HISTORY_UNREADABLE, e))
}
