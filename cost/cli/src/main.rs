//! The `cost` command. See `cost/README.md`.

mod check;
mod git;
mod render;
mod summary;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, bail};
use chrono::{Datelike, NaiveDate};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "cost",
    about = "Check each project's docs/cost.toml against the budgets"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Check every project's docs/cost.toml (COST-001 to COST-010).
    Check {
        /// Repo root. Defaults to the nearest directory above the current
        /// one that holds standards/radar/limits.toml.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Only report per-project findings for these projects (top-level
        /// directories; a path such as cookbook/web counts as cookbook).
        /// Account-wide rules always read every project. Comma separated
        /// or repeated. Without it every project is checked.
        #[arg(long, value_delimiter = ',')]
        projects: Option<Vec<String>>,
        /// Exit with status 1 on any finding except COST-008 to COST-010.
        /// Without it findings are warnings and the exit status is 0.
        #[arg(long)]
        strict: bool,
        /// The date to check against, YYYY-MM-DD. Defaults to today (UTC).
        #[arg(long)]
        today: Option<NaiveDate>,
    },
    /// Compare the working tree with the merge base of --base and print the
    /// PR's cost summary as Markdown: cost files changed, services, usage,
    /// money and budgets that moved, and the labels the PR gets.
    Summary {
        /// Repo root. Defaults as for `check`.
        #[arg(long)]
        root: Option<PathBuf>,
        /// The branch or commit the change is compared with, such as
        /// origin/main.
        #[arg(long)]
        base: String,
        /// Also write the summary as JSON here (labels, needs_human, ...).
        #[arg(long)]
        json: Option<PathBuf>,
        /// The date to price recurring money in, YYYY-MM-DD. Defaults to
        /// today (UTC).
        #[arg(long)]
        today: Option<NaiveDate>,
    },
    /// List the [[cost]] entries whose `renews` date (the next charge) is
    /// within --within days, soonest first, including any already passed.
    Renewals {
        /// Repo root. Defaults as for `check`.
        #[arg(long)]
        root: Option<PathBuf>,
        /// How many days ahead to look.
        #[arg(long, default_value_t = 60)]
        within: i64,
        /// The date to count from, YYYY-MM-DD. Defaults to today (UTC).
        #[arg(long)]
        today: Option<NaiveDate>,
    },
    /// Rewrite the limit tables in standards/docs/free-tiers.md from
    /// standards/radar/limits.toml.
    Render {
        /// Repo root. Defaults as for `check`.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Change nothing; exit 1 when the page is out of date.
        #[arg(long)]
        check: bool,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("cost: {e:#}");
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
            let today = today.unwrap_or_else(|| chrono::Utc::now().date_naive());
            let report = check::check(&root, projects.as_deref(), today)?;
            for finding in &report.findings {
                println!("{finding}");
            }
            println!(
                "cost check: {} finding(s) in {} project(s)",
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
        Command::Summary {
            root,
            base,
            json,
            today,
        } => {
            let root = find_root(root)?;
            let today = today.unwrap_or_else(|| chrono::Utc::now().date_naive());
            let month = cost_model::Month::new(today.year(), today.month())
                .ok_or_else(|| anyhow::anyhow!("today's month is out of range"))?;
            let summary = summary::summary(&root, &base, month)?;
            print!("{}", cost_model::render_summary(&summary));
            if let Some(path) = json {
                std::fs::write(&path, serde_json::to_string_pretty(&summary)? + "\n")
                    .map_err(|e| anyhow::anyhow!("writing {}: {e}", path.display()))?;
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Renewals {
            root,
            within,
            today,
        } => {
            let root = find_root(root)?;
            let today = today.unwrap_or_else(|| chrono::Utc::now().date_naive());
            let projects = check::parsed_projects(&root)?;
            let due = cost_model::renewals(&projects, today, within);
            print!("{}", cost_model::render_renewals(&due, today, within));
            Ok(ExitCode::SUCCESS)
        }
        Command::Render { root, check } => {
            let root = find_root(root)?;
            render::render(&root, check)
        }
    }
}

fn find_root(root: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(root) = root {
        return Ok(root);
    }
    let cwd = std::env::current_dir()?;
    let mut dir: &Path = &cwd;
    loop {
        if dir.join(check::LIMITS).is_file() {
            return Ok(dir.to_path_buf());
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => bail!("no {} above {}; pass --root", check::LIMITS, cwd.display()),
        }
    }
}
