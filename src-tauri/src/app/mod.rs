//! Business logic: configuration, catalog of games, language, errors, random draw, session, history, SQLite storage and Steam detection.
//! No dependency on Tauri, so everything can be tested with `cargo test`.

mod catalog;
mod config;
mod error;
mod history;
mod language;
mod session;
mod steam;
mod store;

pub use catalog::*;
pub use config::*;
pub use error::*;
pub use history::*;
pub use language::*;
pub use session::*;
pub use steam::*;
pub use store::*;
