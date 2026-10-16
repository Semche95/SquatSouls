//! Intensity of the session.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Intensity of the session: scales the drawn numbers without touching the configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Intensity {
    Easy,
    #[default]
    Normal,
    Hard,
}

impl Intensity {
    pub fn as_str(self) -> &'static str {
        match self {
            Intensity::Easy => "easy",
            Intensity::Normal => "normal",
            Intensity::Hard => "hard",
        }
    }

    /// Scales a number between an exercise's bounds: half for easy, one and a half for hard.
    pub fn scale(self, amount: u32) -> u32 {
        match self {
            Intensity::Easy => self.round_half(amount),
            Intensity::Normal => amount,
            Intensity::Hard => self.round_half(amount * 3),
        }
    }

    /// Halves a number with a joker.
    pub fn halve(self, amount: u32) -> u32 {
        self.round_half(amount)
    }

    /// Divides by two: easy rounds down without falling to 0, the others round up.
    fn round_half(self, amount: u32) -> u32 {
        match self {
            Intensity::Easy => (amount / 2).max(amount.min(1)),
            _ => amount.div_ceil(2),
        }
    }
}

impl FromStr for Intensity {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        match value {
            "easy" => Ok(Intensity::Easy),
            "normal" => Ok(Intensity::Normal),
            "hard" => Ok(Intensity::Hard),
            _ => Err(()),
        }
    }
}
