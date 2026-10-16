//! Settings outside the configuration: language, last game, intensity and onboarding.

use crate::app::{AppError, Intensity, LanguageSetting};
use crate::state::{lock, LanguageView, Shared};

#[tauri::command]
pub(crate) fn get_language(state: Shared) -> Result<LanguageView, AppError> {
    lock(&state)?.language_view()
}

#[tauri::command]
pub(crate) fn set_language(state: Shared, setting: String) -> Result<LanguageView, AppError> {
    let setting = setting
        .parse::<LanguageSetting>()
        .map_err(|_| AppError::new("invalid_language").with("value", setting.as_str()))?;
    let mut s = lock(&state)?;
    s.store_language(setting)?;
    s.language_view()
}

#[tauri::command]
pub(crate) fn get_last_game(state: Shared) -> Result<String, AppError> {
    lock(&state)?
        .store
        .last_game()
        .map_err(|e| AppError::internal("Last game unreadable", e))
}

#[tauri::command]
pub(crate) fn set_last_game(state: Shared, game_id: String) -> Result<(), AppError> {
    lock(&state)?
        .store
        .set_last_game(&game_id)
        .map_err(|e| AppError::internal("Last game not saved", e))
}

#[tauri::command]
pub(crate) fn get_intensity(state: Shared) -> Result<Intensity, AppError> {
    lock(&state)?
        .store
        .intensity()
        .map_err(|e| AppError::internal("Intensity unreadable", e))
}

#[tauri::command]
pub(crate) fn set_intensity(state: Shared, intensity: Intensity) -> Result<(), AppError> {
    lock(&state)?
        .store
        .set_intensity(intensity)
        .map_err(|e| AppError::internal("Intensity not saved", e))
}

#[tauri::command]
pub(crate) fn get_onboarding_seen(state: Shared) -> Result<bool, AppError> {
    lock(&state)?
        .store
        .onboarding_seen()
        .map_err(|e| AppError::internal("Onboarding state unreadable", e))
}

#[tauri::command]
pub(crate) fn set_onboarding_seen(state: Shared) -> Result<(), AppError> {
    lock(&state)?
        .store
        .set_onboarding_seen()
        .map_err(|e| AppError::internal("Onboarding state not saved", e))
}
