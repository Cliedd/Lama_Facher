use crate::compiler::diagnostics::Diagnostic;
use crate::core::errors::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct CompileOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub executable: Option<PathBuf>,
}

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RunOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

pub trait LanguageAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn file_extension(&self) -> &str;
    fn compile(&self, code: &str, workdir: &Path) -> Result<CompileOutput>;
    fn run(&self, workdir: &Path) -> Result<RunOutput>;
    fn parse_errors(&self, output: &CompileOutput) -> Vec<Diagnostic>;
    fn starter_code(&self, exercise_name: &str) -> String;
}
