use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    NotStarted,
    InProgress,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseProgress {
    pub exercise_id: String,
    pub status: Status,
    pub attempts: usize,
    pub last_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProgress {
    pub exercises: HashMap<String, ExerciseProgress>,
}

impl UserProgress {
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
