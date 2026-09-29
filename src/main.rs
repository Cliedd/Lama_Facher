use anyhow::{bail, Context, Result};
use clap::Parser;
use forge::cli::{Cli, Commands};
use forge::core::exercise::Exercise;
use forge::core::progress::Status;
use forge::installer::rustup::RustupInstaller;
use forge::installer::sdkman::SdkmanInstaller;
use forge::storage::local::LocalStorage;
use forge::{config, core, lang, tui};
use std::path::PathBuf;

fn main() -> Result<()> {
    core::path_manager::PathManager::ensure_toolchain_paths();
    execute(Cli::parse())
}

fn catalog() -> Result<Vec<Exercise>> {
    let dir = config::try_exercises_dir()?;
    let exercises = Exercise::load_all_from_dir(&dir)?;
    if exercises.is_empty() {
        bail!("No exercises found in {}", dir.display());
    }
    Ok(exercises)
}

fn find_exercise(id: &str) -> Result<Exercise> {
    catalog()?
        .into_iter()
        .find(|ex| ex.id == id)
        .with_context(|| {
            format!("Exercise '{id}' not found. Use `forge list` to see available IDs")
        })
}

fn language(input: &str) -> Result<&'static str> {
    match input.to_ascii_lowercase().as_str() {
        "java" => Ok("java"),
        "rust" | "rs" => Ok("rust"),
        _ => bail!("Unknown language '{input}'. Choose java or rust"),
    }
}

fn execute(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Tui) | None => tui::run_tui()?,
        Some(Commands::Start { language: chosen }) => {
            println!("\nForge — learn by building\n");
            if let Some(chosen) = chosen {
                let chosen = language(&chosen)?;
                let exercises = catalog()?;
                let progress = LocalStorage::new()?.load_progress()?;
                if let Some(ex) = exercises.iter().find(|ex| {
                    ex.language.eq_ignore_ascii_case(chosen)
                        && progress.get_status(&ex.id) != Status::Completed
                }) {
                    println!("Next {chosen} exercise: {} — {}", ex.id, ex.title);
                    println!("Read it with `forge show {}`\n", ex.id);
                } else {
                    println!("All available {chosen} exercises are complete.\n");
                }
            } else {
                println!("Choose a language in the TUI, or run `forge start rust` / `forge start java`.\n");
            }
            tui::run_tui()?;
        }
        Some(Commands::List { language: filter }) => {
            let exercises = catalog()?;
            let progress = LocalStorage::new()?.load_progress()?;
            println!(
                "{:<20} {:<6} {:<12} {:<12} TITLE",
                "ID", "LANG", "DIFFICULTY", "STATUS"
            );
            for ex in exercises.iter().filter(|ex| {
                filter
                    .as_ref()
                    .is_none_or(|f| ex.language.eq_ignore_ascii_case(f))
            }) {
                println!(
                    "{:<20} {:<6} {:<12} {:<12} {}",
                    ex.id,
                    ex.language,
                    ex.difficulty,
                    match progress.get_status(&ex.id) {
                        Status::Completed => "Completed",
                        Status::InProgress => "In progress",
                        Status::NotStarted => "Not started",
                    },
                    ex.title
                );
            }
        }
        Some(Commands::Show { id }) => {
            let ex = find_exercise(&id)?;
            println!(
                "{} ({})\n{} · {}\n\n{}",
                ex.title, ex.id, ex.language, ex.difficulty, ex.description
            );
            if !ex.hints.is_empty() {
                println!("\nHints:");
                for (i, hint) in ex.hints.iter().enumerate() {
                    println!("  {}. {}", i + 1, hint);
                }
            }
            println!("\nTry it: forge run {} --file <your-source-file>", ex.id);
        }
        Some(Commands::Progress) => show_progress()?,
        Some(Commands::Lesson { language: chosen }) => {
            let chosen = language(&chosen)?;
            let exercises = catalog()?;
            let progress = LocalStorage::new()?.load_progress()?;
            let items: Vec<_> = exercises
                .iter()
                .filter(|ex| ex.language.eq_ignore_ascii_case(chosen))
                .collect();
            let completed = items
                .iter()
                .filter(|ex| progress.get_status(&ex.id) == Status::Completed)
                .count();
            println!(
                "{} course — {completed}/{} completed",
                chosen.to_uppercase(),
                items.len()
            );
            if let Some(ex) = items
                .iter()
                .find(|ex| progress.get_status(&ex.id) != Status::Completed)
            {
                println!(
                    "Next: {} — {}\n{}\n\nRead: forge show {}",
                    ex.id, ex.title, ex.description, ex.id
                );
            } else {
                println!("All exercises complete!");
            }
        }
        Some(Commands::Doctor) | Some(Commands::Info) => {
            println!("Forge doctor\n============");
            println!("Rust: {}", RustupInstaller::check_status());
            println!("Java: {}", SdkmanInstaller::check_status());
            let dir = config::try_exercises_dir()?;
            println!("Exercises: {} ({})", catalog()?.len(), dir.display());
            let storage = LocalStorage::new()?;
            println!("Progress: {}", storage.progress_path().display());
            storage
                .load_progress()
                .context("Cannot read progress file")?;
        }
        Some(Commands::Run { id, file }) => run_exercise(&id, file)?,
        Some(Commands::Test) => test_catalog()?,
        Some(Commands::Install { toolchain }) => match toolchain.as_deref() {
            Some("rust") => RustupInstaller::install()?,
            Some("java") => SdkmanInstaller::install()?,
            None => {
                RustupInstaller::install()?;
                SdkmanInstaller::install()?;
            }
            Some(other) => bail!("Unknown toolchain '{other}'. Choose rust or java"),
        },
    }
    Ok(())
}

fn show_progress() -> Result<()> {
    let exercises = catalog()?;
    let progress = LocalStorage::new()?.load_progress()?;
    let completed = exercises
        .iter()
        .filter(|e| progress.get_status(&e.id) == Status::Completed)
        .count();
    let in_progress = exercises
        .iter()
        .filter(|e| progress.get_status(&e.id) == Status::InProgress)
        .count();
    println!(
        "Forge progress\n==============\nCompleted: {completed}/{}\nIn progress: {in_progress}",
        exercises.len()
    );
    for chosen in ["java", "rust"] {
        let items: Vec<_> = exercises
            .iter()
            .filter(|e| e.language.eq_ignore_ascii_case(chosen))
            .collect();
        let done = items
            .iter()
            .filter(|e| progress.get_status(&e.id) == Status::Completed)
            .count();
        println!("{chosen}: {done}/{} completed", items.len());
        if let Some(next) = items
            .iter()
            .find(|e| progress.get_status(&e.id) != Status::Completed)
        {
            println!("  Next: {} — {}", next.id, next.title);
        }
    }
    Ok(())
}

fn run_exercise(id: &str, file: Option<PathBuf>) -> Result<()> {
    let ex = find_exercise(id)?;
    let adapter = lang::get_adapter(&ex.language)
        .with_context(|| format!("No compiler adapter for '{}'", ex.language))?;
    let storage = LocalStorage::new()?;
    let mut progress = storage.load_progress()?;
    let code = if let Some(path) = file {
        std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read source file {}", path.display()))?
    } else {
        progress
            .exercises
            .get(id)
            .and_then(|p| p.last_code.clone())
            .unwrap_or_else(|| ex.template.clone())
    };
    let session = core::session::Session::new(ex.clone(), adapter.clone())?;
    println!("Running: {} ({})", ex.title, ex.id);
    let compiled = session.compile(&code)?;
    if !compiled.success {
        for diag in adapter.parse_errors(&compiled) {
            eprintln!("Line {}: {}", diag.line, diag.message);
            if let Some(suggestion) = diag.suggestion {
                eprintln!("  Hint: {suggestion}");
            }
        }
        if !compiled.stderr.trim().is_empty() {
            eprintln!("{}", compiled.stderr.trim());
        }
        progress.record_failed_attempt(id, &code);
        storage.save_progress(&progress)?;
        bail!("Compilation failed for {id}");
    }
    let output = session.run()?;
    if !output.stdout.is_empty() {
        print!("{}", output.stdout);
    }
    if !output.stderr.is_empty() {
        eprintln!("{}", output.stderr.trim());
    }
    if session.verify(&output) {
        progress.mark_completed(id, &code);
        storage.save_progress(&progress)?;
        println!("✓ Exercise completed");
    } else {
        progress.record_failed_attempt(id, &code);
        storage.save_progress(&progress)?;
        bail!("Exercise {id} failed: output or exit status did not match the expected result");
    }
    Ok(())
}

fn test_catalog() -> Result<()> {
    let exercises = catalog()?;
    let total = exercises.len();
    let mut failed = 0;
    for ex in exercises {
        let result: Result<()> = (|| {
            let adapter = lang::get_adapter(&ex.language)
                .with_context(|| format!("No compiler adapter for '{}'", ex.language))?;
            let session = core::session::Session::new(ex.clone(), adapter)?;
            let compiled = session.compile(&ex.template)?;
            if !compiled.success {
                bail!("Compilation failed: {}", compiled.stderr.trim());
            }
            // Starter code is intentionally incomplete. The catalog command
            // validates that every starter is accepted by its compiler; a
            // learner's solution is graded by `forge run` or the TUI.
            Ok(())
        })();
        match result {
            Ok(()) => println!("PASS {}", ex.id),
            Err(err) => {
                eprintln!("FAIL {}: {err:#}", ex.id);
                failed += 1;
            }
        }
    }
    println!("{} passed, {failed} failed", total - failed);
    if failed > 0 {
        bail!("{failed} exercise(s) failed");
    }
    Ok(())
}
