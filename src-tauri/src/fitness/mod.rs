//! Business logic: configuration, random draw, session, history, SQLite storage and Steam detection.
//! No dependency on Tauri, so everything can be tested with `cargo test`.

mod config;
mod history;
mod session;
mod steam;
mod store;

pub use config::*;
pub use history::*;
pub use session::*;
pub use steam::*;
pub use store::*;
