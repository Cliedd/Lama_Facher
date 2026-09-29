use crate::core::errors::Result;
use crate::core::exercise::Exercise;
use crate::lang::traits::{CompileOutput, LanguageAdapter, RunOutput};
use std::sync::Arc;
use tempfile::TempDir;

pub struct Session {
    pub exercise: Exercise,
    pub adapter: Arc<dyn LanguageAdapter>,
    pub workdir: TempDir,
}

impl Session {
    pub fn new(exercise: Exercise, adapter: Arc<dyn LanguageAdapter>) -> Result<Self> {
        let workdir = tempfile::Builder::new().prefix("forge_").tempdir()?;
        Ok(Self {
            exercise,
            adapter,
            workdir,
        })
    }

    pub fn compile(&self, code: &str) -> Result<CompileOutput> {
        self.adapter.compile(code, self.workdir.path())
    }

    pub fn run(&self) -> Result<RunOutput> {
        self.adapter.run(self.workdir.path())
    }

    pub fn verify(&self, output: &RunOutput) -> bool {
        if !output.success {
            return false;
        }
        if let Some(expected) = &self.exercise.expected_output {
            return output.stdout.trim() == expected.trim();
        }
        output.success
    }
}
