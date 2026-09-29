use crate::core::errors::{ForgeError, Result};
use crate::core::progress::{MergeReport, UserProgress};
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct LocalStorage {
    base_dir: PathBuf,
}

impl LocalStorage {
    pub fn new() -> Result<Self> {
        let base_dir = if let Some(path) = std::env::var_os("FORGE_PROGRESS_DIR") {
            PathBuf::from(path)
        } else if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
            PathBuf::from(path).join("forge")
        } else {
            dirs::home_dir()
                .ok_or_else(|| ForgeError::Generic("Could not find home directory".into()))?
                .join(".config")
                .join("forge")
        };
        Ok(Self { base_dir })
    }

    pub fn at(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn progress_path(&self) -> PathBuf {
        self.base_dir.join("progress.json")
    }

    pub fn load_progress(&self) -> Result<UserProgress> {
        let path = self.progress_path();
        if !path.exists() {
            return Ok(UserProgress::default());
        }
        let content = std::fs::read_to_string(path)?;
        let progress: UserProgress =
            serde_json::from_str(&content).map_err(|e| ForgeError::Serialization(e.to_string()))?;
        Ok(progress)
    }

    pub fn save_progress(&self, progress: &UserProgress) -> Result<()> {
        std::fs::create_dir_all(&self.base_dir)?;
        let path = self.progress_path();
        let content = serde_json::to_string_pretty(progress)
            .map_err(|e| ForgeError::Serialization(e.to_string()))?;
        let mut tmp = tempfile::NamedTempFile::new_in(&self.base_dir)?;
        tmp.write_all(content.as_bytes())?;
        tmp.flush()?;
        tmp.persist(path).map_err(|e| ForgeError::Io(e.error))?;
        Ok(())
    }

    pub fn import_progress(&self, source: &Path) -> Result<MergeReport> {
        let content = std::fs::read_to_string(source)?;
        let imported: UserProgress = serde_json::from_str(&content)
            .map_err(|e| ForgeError::Serialization(format!("invalid backup JSON: {e}")))?;
        imported
            .validate_import()
            .map_err(ForgeError::Serialization)?;
        let mut local = self.load_progress()?;
        let report = local.merge_from(imported);
        if report.added > 0 || report.updated > 0 {
            self.save_progress(&local)?;
        }
        Ok(report)
    }

    pub fn reset_progress(&self) -> Result<()> {
        let path = self.progress_path();
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}
