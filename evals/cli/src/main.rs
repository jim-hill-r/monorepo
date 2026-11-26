//! The `evals` command. See `evals/README.md`.

mod check;

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "evals",
    about = "Check product requirements and the evals that test them"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Validate every project's docs/requirements.md and evals.toml
    /// (EVL-001 to EVL-004, and malformed files).
    Check {
        /// Repo root. Defaults to the nearest directory above the current
        /// one that holds standards/docs/naming.md.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Exit with status 1 when there are findings. Without it findings
        /// are warnings and the exit status is 0.
        #[arg(long)]
        strict: bool,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("evals: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Command::Check { root, strict } => {
            let root = match root {
                Some(root) => root,
                None => {
                    let cwd = std::env::current_dir().context("reading the current directory")?;
                    check::find_root(&cwd).ok_or_else(|| {
                        anyhow!("no {} above {}; pass --root", check::NAMING, cwd.display())
                    })?
                }
            };
            let findings = check::check_repo(&root)?;
            for finding in &findings {
                println!("{finding}");
            }
            println!("evals check: {} finding(s)", findings.len());
            Ok(if strict && !findings.is_empty() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            })
        }
    }
}
