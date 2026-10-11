//! Logique métier : configuration, tirage aléatoire, séance et historique.
//! Aucun lien avec Tauri, ce qui permet de tout tester avec `cargo test`.

mod config;
mod history;
mod session;

pub use config::*;
pub use history::*;
pub use session::*;
