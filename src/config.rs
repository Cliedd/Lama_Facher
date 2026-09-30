use crate::core::settings::Locale;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub exercises_dir: PathBuf,
    pub default_language: String,
    pub auto_compile: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            exercises_dir: PathBuf::from("exercises"),
            default_language: "java".to_string(),
            auto_compile: true,
        }
    }
}

pub fn exercises_dir() -> PathBuf {
    exercises_dir_for(Locale::Fr)
}

pub fn exercises_dir_for(locale: Locale) -> PathBuf {
    try_exercises_dir_for(locale).unwrap_or_else(|_| PathBuf::from("exercises"))
}

/// Locate the exercise catalog independently of the caller's working directory.
/// An explicit FORGE_HOME always wins and reports an error if it is invalid.
pub fn try_exercises_dir() -> std::io::Result<PathBuf> {
    try_exercises_dir_for(Locale::Fr)
}

pub fn try_exercises_dir_for(locale: Locale) -> std::io::Result<PathBuf> {
    if let Some(home) = std::env::var_os("FORGE_HOME") {
        let root = PathBuf::from(home).join("exercises");
        let localized = if locale == Locale::En {
            root.join("en")
        } else {
            root
        };
        return require_exercises(&localized);
    }

    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin) = exe.parent() {
            candidates.push(bin.join("../share/forge/exercises"));
            candidates.push(bin.join("../exercises"));
            candidates.push(bin.join("../../exercises"));
        }
    }
    // Useful when running a development binary from outside the repository.
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("exercises"));
    candidates.push(PathBuf::from("exercises"));
    for root in candidates {
        let path = if locale == Locale::En {
            root.join("en")
        } else {
            root
        };
        if path.is_dir() {
            return Ok(path.canonicalize().unwrap_or(path));
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Exercise catalog not found; set FORGE_HOME to the Forge installation directory",
    ))
}

fn require_exercises(path: &Path) -> std::io::Result<PathBuf> {
    if path.is_dir() {
        path.canonicalize()
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Exercise catalog not found at {}", path.display()),
        ))
    }
}
