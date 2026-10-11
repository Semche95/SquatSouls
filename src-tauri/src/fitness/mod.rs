//! Logique métier : configuration, tirage aléatoire, séance, historique, stockage SQLite et détection Steam.
//! Aucun lien avec Tauri, ce qui permet de tout tester avec `cargo test`.

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
