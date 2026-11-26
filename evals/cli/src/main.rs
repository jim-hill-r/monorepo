//! The `evals` command. See `evals/README.md`.

mod check;
mod publish;
mod reports;
mod run;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand, ValueEnum};
use evals_model::{ResultStatus, Stage, parse_evals};

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
    /// Turn the test reports a project's tasks already wrote into one result
    /// per eval, appended to a JSONL file. Collects `cargo` and `playwright`
    /// evals; runs no tests itself.
    Run {
        /// The stage to collect, such as `ci`.
        #[arg(long, value_enum)]
        stage: StageArg,
        /// The top-level project directory, such as `cookbook`.
        #[arg(long)]
        project: String,
        /// Repo root. Defaults as for `check`.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Saved `cargo test` output. Repeat for several crates.
        #[arg(long = "cargo-output")]
        cargo_output: Vec<PathBuf>,
        /// A Playwright JSON reporter file. Repeat for several apps.
        #[arg(long = "playwright-report")]
        playwright_report: Vec<PathBuf>,
        /// The commit the reports are for. Defaults to $GITHUB_SHA.
        #[arg(long)]
        commit: Option<String>,
        /// The JSONL file to append results to.
        #[arg(long, default_value = "evals-results.jsonl")]
        out: PathBuf,
    },
    /// Write the job summary for a results file: one table per project.
    Publish {
        /// The JSONL results file.
        #[arg(long, default_value = "evals-results.jsonl")]
        results: PathBuf,
        /// Where to append the Markdown. Defaults to $GITHUB_STEP_SUMMARY,
        /// or standard output when that isn't set.
        #[arg(long)]
        summary: Option<PathBuf>,
    },
}

/// The stages `evals run` accepts. `human` results come from Jim, not a run.
#[derive(Clone, Copy, ValueEnum)]
enum StageArg {
    Ci,
    Cd,
    Chaos,
    Observe,
    Judge,
}

impl From<StageArg> for Stage {
    fn from(stage: StageArg) -> Self {
        match stage {
            StageArg::Ci => Stage::Ci,
            StageArg::Cd => Stage::Cd,
            StageArg::Chaos => Stage::Chaos,
            StageArg::Observe => Stage::Observe,
            StageArg::Judge => Stage::Judge,
        }
    }
}

fn main() -> ExitCode {
    match dispatch(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("evals: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn dispatch(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Command::Check { root, strict } => {
            let root = resolve_root(root)?;
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
        Command::Run {
            stage,
            project,
            root,
            cargo_output,
            playwright_report,
            commit,
            out,
        } => {
            let root = resolve_root(root)?;
            let evals_path = root.join(&project).join(check::EVALS);
            let text = fs::read_to_string(&evals_path)
                .with_context(|| format!("reading {}", evals_path.display()))?;
            let file = parse_evals(&text).map_err(|errors| {
                let list: Vec<String> = errors.iter().map(ToString::to_string).collect();
                anyhow!("{} is invalid: {}", evals_path.display(), list.join("; "))
            })?;

            let reports = run::Reports {
                cargo: read_all(&cargo_output)?.map(|texts| {
                    texts
                        .iter()
                        .flat_map(|t| reports::parse_cargo_output(t))
                        .collect()
                }),
                playwright: match read_all(&playwright_report)? {
                    None => None,
                    Some(texts) => {
                        let mut cases = Vec::new();
                        for text in texts {
                            cases.extend(reports::parse_playwright_report(&text)?);
                        }
                        Some(cases)
                    }
                },
            };
            let env = |name: &str| std::env::var(name).ok().filter(|v| !v.is_empty());
            let run_id = env("GITHUB_RUN_ID");
            let evidence = match (env("GITHUB_SERVER_URL"), env("GITHUB_REPOSITORY"), &run_id) {
                (Some(server), Some(repo), Some(id)) => {
                    Some(format!("{server}/{repo}/actions/runs/{id}"))
                }
                _ => None,
            };
            let ctx = run::RunContext {
                project,
                stage: stage.into(),
                commit: commit
                    .or_else(|| env("GITHUB_SHA"))
                    .unwrap_or_else(|| "unknown".into()),
                run: run_id,
                evidence,
                at: chrono::Utc::now(),
            };
            let (results, notes) = run::collect(&file, &reports, &ctx);
            for note in notes {
                println!("note: {note}");
            }
            append(&out, &publish::to_jsonl(&results)?)?;
            let failed = results
                .iter()
                .filter(|r| r.status == ResultStatus::Fail)
                .count();
            println!(
                "evals run: {} result(s), {failed} failed, appended to {}",
                results.len(),
                out.display()
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::Publish { results, summary } => {
            let text = fs::read_to_string(&results)
                .with_context(|| format!("reading {}", results.display()))?;
            let markdown = publish::summary(&publish::parse_results(&text)?);
            let target = summary.or_else(|| {
                std::env::var_os("GITHUB_STEP_SUMMARY")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            });
            match target {
                Some(path) => append(&path, &markdown)?,
                None => print!("{markdown}"),
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn resolve_root(root: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(root) = root {
        return Ok(root);
    }
    let cwd = std::env::current_dir().context("reading the current directory")?;
    check::find_root(&cwd)
        .ok_or_else(|| anyhow!("no {} above {}; pass --root", check::NAMING, cwd.display()))
}

/// Reads every file, or returns `None` when no paths were given.
fn read_all(paths: &[PathBuf]) -> Result<Option<Vec<String>>> {
    if paths.is_empty() {
        return Ok(None);
    }
    let mut texts = Vec::new();
    for path in paths {
        if !path.is_file() {
            bail!("report {} does not exist", path.display());
        }
        texts
            .push(fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?);
    }
    Ok(Some(texts))
}

fn append(path: &Path, text: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("opening {}", path.display()))?;
    file.write_all(text.as_bytes())
        .with_context(|| format!("writing {}", path.display()))
}
