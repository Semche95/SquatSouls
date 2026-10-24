//! Tauri commands called by the interface, by domain: configuration, catalog of games, settings, session, history,
//! Steam, notifications, notification area, updates and the opening of the database at launch.

mod catalog;
mod config;
mod history;
mod notification;
mod session;
mod settings;
mod startup;
mod steam;
mod tray;
mod update;

pub(crate) use catalog::*;
pub(crate) use config::*;
pub(crate) use history::*;
pub(crate) use notification::*;
pub(crate) use session::*;
pub(crate) use settings::*;
pub(crate) use startup::*;
pub(crate) use steam::*;
pub(crate) use tray::*;
pub(crate) use update::*;
