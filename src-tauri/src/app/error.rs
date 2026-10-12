//! Errors sent to the interface: a stable code and the values to show, never a translated text.

use serde::Serialize;
use serde_json::{Map, Value};
use std::fmt::Display;

/// Code of technical errors (SQLite, input/output...), whose detail only goes to the logs.
pub const INTERNAL: &str = "internal";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AppError {
    /// Stable snake_case code, translated by the interface.
    pub code: &'static str,
    /// Values shown in the message (exercise name, bounds, id...).
    pub params: Map<String, Value>,
}

impl AppError {
    pub fn new(code: &'static str) -> Self {
        AppError {
            code,
            params: Map::new(),
        }
    }

    pub fn with(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Technical error: logged with its context, sent to the interface as `internal`.
    pub fn internal(context: &str, error: impl Display) -> Self {
        Self::logged(INTERNAL, context, error)
    }

    /// Error whose technical detail is logged with its context, never sent to the interface.
    pub fn logged(code: &'static str, context: &str, error: impl Display) -> Self {
        log::error!("{context}: {error}");
        Self::new(code)
    }
}

impl From<AppError> for Value {
    fn from(error: AppError) -> Self {
        serde_json::to_value(error).expect("serializable error")
    }
}
