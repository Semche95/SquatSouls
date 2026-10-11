//! Logique métier : configuration, tirage aléatoire, séance, historique et stockage SQLite.
//! Aucun lien avec Tauri, ce qui permet de tout tester avec `cargo test`.

mod config;
mod history;
mod session;
mod store;

pub use config::*;
pub use history::*;
pub use session::*;
pub use store::*;
