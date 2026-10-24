//! Interface language: setting and effective language. Texts are translated by the interface only.

use serde::Serialize;
use std::str::FromStr;

/// Effective language of the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Fr,
}

impl Language {
    pub fn as_str(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Fr => "fr",
        }
    }
}

/// Language chosen in the settings: the system's, or a fixed one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LanguageSetting {
    #[default]
    System,
    En,
    Fr,
}

impl LanguageSetting {
    pub fn as_str(self) -> &'static str {
        match self {
            LanguageSetting::System => "system",
            LanguageSetting::En => "en",
            LanguageSetting::Fr => "fr",
        }
    }

    /// Effective language: for `System`, French if the system locale is French
    /// (`fr`, `fr-FR`, `fr_CA.UTF-8`...), English otherwise.
    pub fn resolve(self, system_locale: Option<&str>) -> Language {
        match self {
            LanguageSetting::En => Language::En,
            LanguageSetting::Fr => Language::Fr,
            LanguageSetting::System => {
                if system_locale.is_some_and(|l| l.to_lowercase().starts_with("fr")) {
                    Language::Fr
                } else {
                    Language::En
                }
            }
        }
    }
}

impl FromStr for LanguageSetting {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        match value {
            "system" => Ok(LanguageSetting::System),
            "en" => Ok(LanguageSetting::En),
            "fr" => Ok(LanguageSetting::Fr),
            _ => Err(()),
        }
    }
}
