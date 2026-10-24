//! Session: draw, intensity and limit, exercises and sets, start, end and cancellation.

use crate::app::{AppError, Config, Draw, Intensity, Session, SessionLimit, SessionView};
use crate::state::{lock, now, Shared};

#[tauri::command]
pub(crate) fn draw(
    state: Shared,
    game_id: String,
    event_id: String,
    intensity: Intensity,
    limit: Option<SessionLimit>,
) -> Result<Draw, AppError> {
    lock(&state)?.change_session(|session, config| {
        session.set_intensity(intensity);
        session.set_limit(limit)?;
        let now = now();
        let draw = session.draw(config, &game_id, &event_id, now, &mut rand::rng())?;
        session.start(config, &game_id, now)?;
        Ok(draw)
    })
}

#[tauri::command]
pub(crate) fn set_session_intensity(
    state: Shared,
    intensity: Intensity,
) -> Result<SessionView, AppError> {
    change(&state, |session, _| {
        session.set_intensity(intensity);
        Ok(())
    })
}

#[tauri::command]
pub(crate) fn set_session_limit(
    state: Shared,
    limit: Option<SessionLimit>,
) -> Result<SessionView, AppError> {
    change(&state, |session, _| session.set_limit(limit))
}

#[tauri::command]
pub(crate) fn complete_exercise(
    state: Shared,
    draw_id: String,
    exercise_id: String,
) -> Result<SessionView, AppError> {
    change(&state, |session, _| {
        session
            .complete_exercise(&draw_id, &exercise_id, now())
            .map(drop)
    })
}

#[tauri::command]
pub(crate) fn dismiss(state: Shared, draw_id: String) -> Result<SessionView, AppError> {
    change(&state, |session, _| session.dismiss(&draw_id))
}

#[tauri::command]
pub(crate) fn halve(
    state: Shared,
    draw_id: String,
    exercise_id: String,
) -> Result<SessionView, AppError> {
    change(&state, |session, _| session.halve(&draw_id, &exercise_id))
}

#[tauri::command]
pub(crate) fn skip_exercise(
    state: Shared,
    draw_id: String,
    exercise_id: String,
) -> Result<SessionView, AppError> {
    change(&state, |session, _| {
        session.skip_exercise(&draw_id, &exercise_id)
    })
}

#[tauri::command]
pub(crate) fn skip_set(state: Shared, draw_id: String) -> Result<SessionView, AppError> {
    change(&state, |session, _| session.skip_set(&draw_id))
}

#[tauri::command]
pub(crate) fn get_session(state: Shared) -> Result<SessionView, AppError> {
    Ok(lock(&state)?.session().view())
}

#[tauri::command]
pub(crate) fn start_session(
    state: Shared,
    game_id: String,
    intensity: Intensity,
    limit: Option<SessionLimit>,
) -> Result<SessionView, AppError> {
    change(&state, |session, config| {
        session.set_intensity(intensity);
        session.set_limit(limit)?;
        session.start(config, &game_id, now()).map(drop)
    })
}

/// Ends the session, with at least one set done and none left to do; the next one waits to be started.
#[tauri::command]
pub(crate) fn end_session(state: Shared) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.session().check_end()?;
    s.end_session(None);
    Ok(s.session().view())
}

#[tauri::command]
pub(crate) fn cancel_session(state: Shared) -> Result<SessionView, AppError> {
    let mut s = lock(&state)?;
    s.cancel_session()?;
    Ok(s.session().view())
}

/// Applies `change` to the session, saved with it, and returns what the session shows.
fn change(
    state: &Shared,
    change: impl FnOnce(&mut Session, &Config) -> Result<(), AppError>,
) -> Result<SessionView, AppError> {
    let mut s = lock(state)?;
    s.change_session(change)?;
    Ok(s.session().view())
}
