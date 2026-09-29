use crate::core::errors::{ForgeError, Result};
use crate::core::exercise::Exercise;
use crate::lang::traits::{CompileOutput, LanguageAdapter, RunOutput};
use std::sync::Arc;
use std::sync::Mutex;
use tempfile::TempDir;

pub struct Session {
    pub exercise: Exercise,
    pub adapter: Arc<dyn LanguageAdapter>,
    pub workdir: TempDir,
    source: Mutex<Option<String>>,
}

impl Session {
    pub fn new(exercise: Exercise, adapter: Arc<dyn LanguageAdapter>) -> Result<Self> {
        let workdir = tempfile::Builder::new().prefix("forge_").tempdir()?;
        Ok(Self {
            exercise,
            adapter,
            workdir,
            source: Mutex::new(None),
        })
    }

    pub fn compile(&self, code: &str) -> Result<CompileOutput> {
        *self.source.lock().expect("session source lock poisoned") = None;
        let compiled = self.adapter.compile(code, self.workdir.path())?;
        if compiled.success {
            *self.source.lock().expect("session source lock poisoned") = Some(code.to_owned());
        }
        Ok(compiled)
    }

    pub fn run(&self) -> Result<RunOutput> {
        if self
            .source
            .lock()
            .expect("session source lock poisoned")
            .is_none()
        {
            return Err(ForgeError::Generic(
                "Compile successfully before running this exercise".into(),
            ));
        }
        self.adapter.run(self.workdir.path())
    }

    pub fn verify(&self, output: &RunOutput) -> bool {
        if !output.success
            || self
                .source
                .lock()
                .expect("session source lock poisoned")
                .is_none()
        {
            return false;
        }
        if let Some(expected) = &self.exercise.expected_output {
            if output.stdout.trim() != expected.trim() {
                return false;
            }
        }
        if self.exercise.test_cases.is_empty() {
            return true;
        }
        let source = self
            .source
            .lock()
            .expect("session source lock poisoned")
            .clone();
        let Some(source) = source else { return false };
        self.exercise.test_cases.iter().all(|case| {
            let Ok(source) = case.source_for(&source) else {
                return false;
            };
            let Ok(workdir) = tempfile::Builder::new().prefix("forge_case_").tempdir() else {
                return false;
            };
            let Ok(compiled) = self.adapter.compile(&source, workdir.path()) else {
                return false;
            };
            if !compiled.success {
                return false;
            }
            let Ok(actual) = self.adapter.run(workdir.path()) else {
                return false;
            };
            actual.success && actual.stdout.trim() == case.expected_output.trim()
        })
    }
}
