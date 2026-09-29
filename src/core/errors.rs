use thiserror::Error;

#[derive(Error, Debug)]
pub enum ForgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Language error: {0}")]
    Language(String),

    #[error("Compile error: {0}")]
    Compile(String),

    #[error("{phase} of {program} exceeded the {timeout_ms} ms limit. Set {variable} to a larger value if needed")]
    Timeout {
        phase: &'static str,
        program: String,
        timeout_ms: u64,
        variable: &'static str,
    },

    #[error(
        "{phase} of {program} produced more than {limit_kb} KiB of output; reduce printed output"
    )]
    OutputLimit {
        phase: &'static str,
        program: String,
        limit_kb: u64,
    },

    #[error("Exercise not found: {0}")]
    ExerciseNotFound(String),

    #[error("Toolchain error: {0}")]
    Toolchain(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, ForgeError>;
