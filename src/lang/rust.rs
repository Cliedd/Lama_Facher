use crate::compiler::diagnostics::Diagnostic;
use crate::compiler::rust::RustCompiler;
use crate::core::errors::Result;
use crate::lang::traits::{CompileOutput, LanguageAdapter, RunOutput};
use std::path::Path;

pub struct RustLanguage {
    compiler: RustCompiler,
}

impl RustLanguage {
    pub fn new() -> Self {
        Self {
            compiler: RustCompiler::new(),
        }
    }
}

impl LanguageAdapter for RustLanguage {
    fn name(&self) -> &str {
        "rust"
    }

    fn file_extension(&self) -> &str {
        "rs"
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
        r#"fn main() {
    // TODO: Complete exercise
    println!("Hello, Forge!");
}
"#
        .to_string()
    }
}
