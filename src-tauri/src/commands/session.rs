//! Session: draw, exercises and sets, start, end and cancellation.

use crate::app::{AppError, Draw, Intensity, SessionView};
use crate::state::{lock, now, AppState, Shared};

#[tauri::command]
pub(crate) fn draw(
    state: Shared,
    game_id: String,
    event_id: String,
    intensity: Intensity,
) -> Result<Draw, AppError> {
    let mut guard = lock(&state)?;
    let AppState {
        config, session, ..
    } = &mut *guard;
    session.set_intensity(intensity);
    let draw = session.draw(config, &game_id, &event_id, &mut rand::rng())?;
    session.start(config, &game_id, now())?;
    guard.save_session_summary();
    Ok(draw)
}

#[tauri::command]
pub(crate) fn set_session_intensity(
    state: Shared,
    intensity: Intensity,
) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session.set_intensity(intensity);
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn complete_exercise(
    state: Shared,
    draw_id: u64,
    exercise_id: String,
) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    let entry = s.session.complete_exercise(draw_id, &exercise_id, now())?;
    if let Err(e) = s.store.append_history(&[entry]) {
        log::error!("History not saved: {e}");
    }
    s.save_session_summary();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn dismiss(state: Shared, draw_id: u64) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session.dismiss(draw_id)?;
    s.save_session_summary();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn halve(
    state: Shared,
    draw_id: u64,
    exercise_id: String,
) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session.halve(draw_id, &exercise_id)?;
    s.save_session_summary();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn skip_exercise(
    state: Shared,
    draw_id: u64,
    exercise_id: String,
) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session.skip_exercise(draw_id, &exercise_id)?;
    s.save_session_summary();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn skip_set(state: Shared, draw_id: u64) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session.skip_set(draw_id)?;
    s.save_session_summary();
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn get_session(state: Shared) -> Result<SessionView, AppError> {
    Ok(lock(&state)?.session.view())
}

#[tauri::command]
pub(crate) fn start_session(
    state: Shared,
    game_id: String,
    intensity: Intensity,
) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    let AppState {
        config, session, ..
    } = &mut *s;
    session.set_intensity(intensity);
    session.start(config, &game_id, now())?;
    Ok(s.session.view())
}

/// Ends the session, with at least one set done and none left to do; the next one waits to be started.
#[tauri::command]
pub(crate) fn end_session(state: Shared) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session.check_end()?;
    s.end_session(None);
    Ok(s.session.view())
}

#[tauri::command]
pub(crate) fn cancel_session(state: Shared) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.cancel_session()?;
    Ok(s.session.view())
}
