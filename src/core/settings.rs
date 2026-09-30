use crate::core::errors::{ForgeError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Locale {
    #[default]
    Fr,
    En,
}

impl Locale {
    pub fn code(self) -> &'static str {
        match self {
            Self::Fr => "fr",
            Self::En => "en",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Fr => "Français",
            Self::En => "English",
        }
    }

    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "fr" | "français" | "francais" | "french" => Ok(Self::Fr),
            "en" | "english" | "anglais" => Ok(Self::En),
            other => Err(ForgeError::Generic(format!(
                "Unknown interface language '{other}'. Choose fr or en"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct UserSettings {
    pub locale: Locale,
}

impl UserSettings {
    pub fn path(base_dir: &PathBuf) -> PathBuf {
        base_dir.join("settings.json")
    }
}
