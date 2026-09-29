use crate::core::errors::ForgeError;
use crate::core::errors::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub id: String,
    pub title: String,
    pub language: String,
    pub difficulty: String,
    pub description: String,
    pub template: String,
    #[serde(default)]
    pub chapter: String,
    #[serde(default)]
    pub lesson: String,
    #[serde(default)]
    pub tip: String,
    #[serde(default)]
    pub expected_output: Option<String>,
    #[serde(default)]
    pub test_code: Option<String>,
    #[serde(default)]
    pub hints: Vec<String>,
}

impl Exercise {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let exercise: Exercise = serde_yaml::from_str(&content)?;
        Ok(exercise)
    }

    pub fn load_all_from_dir(dir: &Path) -> Result<Vec<Self>> {
        let mut exercises = Vec::new();
        if !dir.is_dir() {
            return Err(ForgeError::Generic(format!(
                "Exercise catalog not found: {}",
                dir.display()
            )));
        }

        let mut seen = HashSet::new();
        for entry in walkdir::WalkDir::new(dir).sort_by_file_name() {
            let entry = entry
                .map_err(|e| ForgeError::Generic(format!("Cannot read exercise catalog: {e}")))?;
            let path = entry.path();
            if path.is_file()
                && (path
                    .extension()
                    .map_or(false, |ext| ext == "yaml" || ext == "yml"))
            {
                let ex = Self::load_from_file(path)
                    .map_err(|e| ForgeError::Generic(format!("{}: {e}", path.display())))?;
                if ex.id.trim().is_empty() || ex.language.trim().is_empty() {
                    return Err(ForgeError::Generic(format!(
                        "{}: exercise ID and language are required",
                        path.display()
                    )));
                }
                if !seen.insert(ex.id.clone()) {
                    return Err(ForgeError::Generic(format!(
                        "Duplicate exercise ID '{}' in {}",
                        ex.id,
                        path.display()
                    )));
                }
                exercises.push(ex);
            }
        }

        exercises.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(exercises)
    }
}
