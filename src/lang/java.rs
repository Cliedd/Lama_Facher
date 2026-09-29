use crate::compiler::diagnostics::Diagnostic;
use crate::compiler::java::JavaCompiler;
use crate::core::errors::Result;
use crate::lang::traits::{CompileOutput, LanguageAdapter, RunOutput};
use std::path::Path;

pub struct JavaLanguage {
    compiler: JavaCompiler,
}

impl JavaLanguage {
    pub fn new() -> Self {
        Self {
            compiler: JavaCompiler::new(),
        }
    }
}

impl LanguageAdapter for JavaLanguage {
    fn name(&self) -> &str {
        "java"
    }

    fn file_extension(&self) -> &str {
        "java"
    }

    fn compile(&self, code: &str, workdir: &Path) -> Result<CompileOutput> {
        self.compiler.compile(code, workdir)
    }

    fn run(&self, workdir: &Path) -> Result<RunOutput> {
        self.compiler.run(workdir)
    }

    fn parse_errors(&self, output: &CompileOutput) -> Vec<Diagnostic> {
        self.compiler.parse_errors(output)
    }

    fn starter_code(&self, _exercise_name: &str) -> String {
        r#"public class Main {
    public static void main(String[] args) {
        // TODO: Complete exercise
        System.out.println("Hello, Forge!");
    }
}
"#
        .to_string()
    }
}
