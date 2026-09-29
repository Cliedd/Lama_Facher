use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub code: Option<String>,
    pub suggestion: Option<String>,
}

impl Diagnostic {
    pub fn new(severity: Severity, line: usize, column: usize, message: String) -> Self {
        Self {
            severity,
            line,
            column,
            message,
            code: None,
            suggestion: None,
        }
    }
}
