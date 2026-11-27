//! The `threat` command. See `threat/README.md`.

mod check;

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use threat_model::Date;

#[derive(Parser)]
#[command(
    name = "threat",
    about = "Check each project's docs/threat.toml against standards/docs/security.md"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check every project's docs/threat.toml (THR-001 to THR-009, except
    /// THR-004, which comes in phase 3).
    Check {
        /// Repo root. Defaults to the nearest directory above the current
        /// one that holds standards/settings.toml.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Only check these projects (top-level directories; a path such as
        /// cookbook/web counts as cookbook). Comma separated or repeated.
        /// Without it every project is checked.
        #[arg(long, value_delimiter = ',')]
        projects: Option<Vec<String>>,
        /// Exit with status 1 on any finding except THR-009. Without it
        /// findings are warnings and the exit status is 0.
        #[arg(long)]
        strict: bool,
        /// The date to check against, YYYY-MM-DD. Defaults to today (UTC).
        #[arg(long)]
        today: Option<Date>,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("threat: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Command::Check {
            root,
            projects,
            strict,
            today,
        } => {
            let root = find_root(root)?;
            let today = today.unwrap_or_else(utc_today);
            let report = check::check(&root, projects.as_deref(), today)?;
            for finding in &report.findings {
                println!("{finding}");
            }
            println!(
                "threat check: {} finding(s) in {} project(s)",
                report.findings.len(),
                report.checked
            );
            let fails = report.findings.iter().any(|f| f.code.can_fail());
            Ok(if strict && fails {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            })
        }
    }
}

fn utc_today() -> Date {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    Date::from_days(i64::try_from(secs / 86_400).unwrap_or(0))
}

fn find_root(root: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(root) = root {
        return Ok(root);
    }
    let cwd = std::env::current_dir()?;
    let mut dir: &Path = &cwd;
    loop {
        if dir.join(check::SETTINGS).is_file() {
            return Ok(dir.to_path_buf());
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => bail!(
                "no {} above {}; pass --root",
                check::SETTINGS,
                cwd.display()
            ),
        }
    }
}
