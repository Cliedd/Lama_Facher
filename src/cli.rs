use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "forge")]
#[command(about = "Learn Rust and Java in your terminal", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Launch the Terminal User Interface
    Tui,
    /// Show a guided welcome and start the learning journey
    Start {
        /// Language to learn: 'java' or 'rust'
        language: Option<String>,
    },
    /// List all available exercises
    List {
        #[arg(short, long, value_parser = ["rust", "java"])]
        language: Option<String>,
    },
    /// Read an exercise's goal and hints
    Show {
        /// Exercise ID
        id: String,
    },
    /// Show learning progress and the next recommended exercises
    Progress {
        #[command(subcommand)]
        command: Option<ProgressCommands>,
    },
    /// Show the lesson for a language and topic
    Lesson {
        /// Language to learn: 'java' or 'rust'
        language: String,
    },
    /// Check Forge and language toolchain installation
    Doctor,
    /// Run and verify a specific exercise
    Run {
        /// ID of the exercise to run
        id: String,
        /// Read your solution from a source file
        #[arg(short, long)]
        file: Option<std::path::PathBuf>,
    },
    /// Run all exercises test suite
    Test,
    /// Display toolchain status and environment info
    Info,
    /// Install required language toolchains (Rustup / SDKMAN)
    Install {
        /// Toolchain to install: 'rust' or 'java'
        toolchain: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ProgressCommands {
    /// Export saved statuses, attempts, and code to standard output
    Export {
        #[arg(long, value_enum, default_value_t = ExportFormat::Json)]
        format: ExportFormat,
    },
    /// Erase one exercise's progress, or all progress with --yes
    Reset {
        /// Reset only this exercise ID
        #[arg(long)]
        exercise: Option<String>,
        /// Confirm erasing all progress and saved code
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    Json,
    Csv,
}
