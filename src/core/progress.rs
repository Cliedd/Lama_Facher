use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    NotStarted,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExerciseProgress {
    pub exercise_id: String,
    pub status: Status,
    pub attempts: usize,
    pub last_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct UserProgress {
    pub exercises: HashMap<String, ExerciseProgress>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct MergeReport {
    pub added: usize,
    pub updated: usize,
    pub unchanged: usize,
    pub code_conflicts: usize,
}

impl UserProgress {
    pub fn validate_import(&self) -> std::result::Result<(), String> {
        for (id, entry) in &self.exercises {
            if id.is_empty() || id.trim() != id || id.chars().any(char::is_control) {
                return Err(format!("invalid exercise ID {id:?}"));
            }
            if entry.exercise_id != *id {
                return Err(format!(
                    "exercise ID mismatch: key {id:?}, record {:?}",
                    entry.exercise_id
                ));
            }
        }
        Ok(())
    }

    /// Merge without discarding local code or double-counting shared attempts.
    pub fn merge_from(&mut self, imported: UserProgress) -> MergeReport {
        let mut report = MergeReport::default();
        for (id, incoming) in imported.exercises {
            match self.exercises.get_mut(&id) {
                None => {
                    self.exercises.insert(id, incoming);
                    report.added += 1;
                }
                Some(local) => {
                    let before = local.clone();
                    if local.last_code.is_some()
                        && incoming.last_code.is_some()
                        && local.last_code != incoming.last_code
                    {
                        report.code_conflicts += 1;
                    }
                    if local.last_code.is_none() {
                        local.last_code = incoming.last_code;
                    }
                    local.attempts = local.attempts.max(incoming.attempts);
                    local.status = match (&local.status, &incoming.status) {
                        (Status::Completed, _) | (_, Status::Completed) => Status::Completed,
                        (Status::InProgress, _) | (_, Status::InProgress) => Status::InProgress,
                        _ => Status::NotStarted,
                    };
                    if *local == before {
                        report.unchanged += 1;
                    } else {
                        report.updated += 1;
                    }
                }
            }
        }
        report
    }

    /// Remove a single exercise, including its saved code and attempts.
    pub fn reset_exercise(&mut self, id: &str) -> bool {
        self.exercises.remove(id).is_some()
    }

    /// RFC 4180 CSV with stable ordering for diffs and spreadsheet import.
    pub fn to_csv(&self) -> String {
        fn field(value: &str) -> String {
            format!("\"{}\"", value.replace('"', "\"\""))
        }

        let mut rows = String::from("exercise_id,status,attempts,last_code\r\n");
        let mut entries: Vec<_> = self.exercises.iter().collect();
        entries.sort_by_key(|(id, _)| *id);
        for (id, item) in entries {
            let status = match item.status {
                Status::NotStarted => "NotStarted",
                Status::InProgress => "InProgress",
                Status::Completed => "Completed",
            };
            rows.push_str(&format!(
                "{},{},{},{}\r\n",
                field(id),
                field(status),
                item.attempts,
                field(item.last_code.as_deref().unwrap_or(""))
            ));
        }
        rows
    }

    pub fn get_status(&self, id: &str) -> Status {
        self.exercises
            .get(id)
            .map(|p| p.status.clone())
            .unwrap_or(Status::NotStarted)
    }

    pub fn mark_completed(&mut self, id: &str, code: &str) {
        let entry = self
            .exercises
            .entry(id.to_string())
            .or_insert_with(|| ExerciseProgress {
                exercise_id: id.to_string(),
                status: Status::NotStarted,
                attempts: 0,
                last_code: None,
            });
        entry.status = Status::Completed;
        entry.attempts += 1;
        entry.last_code = Some(code.to_string());
    }

    pub fn record_failed_attempt(&mut self, id: &str, code: &str) {
        let entry = self
            .exercises
            .entry(id.to_string())
            .or_insert_with(|| ExerciseProgress {
                exercise_id: id.to_string(),
                status: Status::InProgress,
                attempts: 0,
                last_code: None,
            });
        if entry.status == Status::NotStarted {
            entry.status = Status::InProgress;
        }
        entry.attempts += 1;
        entry.last_code = Some(code.to_string());
    }

    pub fn save_draft(&mut self, id: &str, code: &str) {
        let entry = self
            .exercises
            .entry(id.to_string())
            .or_insert_with(|| ExerciseProgress {
                exercise_id: id.to_string(),
                status: Status::InProgress,
                attempts: 0,
                last_code: None,
            });
        if entry.status == Status::NotStarted {
            entry.status = Status::InProgress;
        }
        entry.last_code = Some(code.to_string());
    }
}
