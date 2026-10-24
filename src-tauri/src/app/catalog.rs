//! Catalog of games published with each release (`scripts/catalog.js`, from the seed's games): its
//! address and the reading of its content, a list of share texts.

use crate::app::{AppError, Language};

/// Largest catalog read, far above the one published: an answer from the network is never trusted.
pub const MAX_CATALOG_SIZE: u64 = 1024 * 1024;

/// Address of the catalogs of the latest release of `repository`.
pub fn release_catalogs(repository: &str) -> String {
    format!("{repository}/releases/latest/download")
}

/// Catalog in `language` among those published at `base`, with or without a trailing slash.
pub fn catalog_url(base: &str, language: Language) -> String {
    format!(
        "{}/catalog.{}.json",
        base.trim_end_matches('/'),
        language.as_str()
    )
}

/// Share texts of the catalog: the interface checks each one as a pasted text.
pub fn read_catalog(body: &str) -> Result<Vec<String>, AppError> {
    serde_json::from_str(body)
        .map_err(|e| AppError::logged("catalog_invalid", "Catalog unreadable", e))
}
