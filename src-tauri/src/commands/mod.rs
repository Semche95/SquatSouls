//! Tauri commands called by the interface, by domain: configuration, settings, session, history,
//! Steam, notifications, notification area and updates.

mod config;
mod history;
mod notification;
mod session;
mod settings;
mod steam;
mod tray;
mod update;

pub(crate) use config::*;
pub(crate) use history::*;
pub(crate) use notification::*;
pub(crate) use session::*;
pub(crate) use settings::*;
pub(crate) use steam::*;
pub(crate) use tray::*;
pub(crate) use update::*;
