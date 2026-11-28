//! The `evals` command. See `evals/README.md`.

mod check;
mod coverage;
mod drill;
mod http;
mod human_checks;
mod publish;
mod reports;
mod run;
mod store;
mod triage;

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand, ValueEnum};
use evals_model::{ResultStatus, Stage, parse_evals, parse_requirements};

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
    /// (EVL-001 to EVL-006, and malformed files).
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
    /// per eval, appended to a JSONL file. Collects `cargo`, `trycmd` and
    /// `playwright` evals (it runs no tests itself) and makes the requests of `http`
    /// evals.
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
        /// Saved `cargo test` output, which holds trycmd's case lines too.
        /// Repeat for several crates.
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
        /// The reports cover only part of the project (one sub-project's CI
        /// job): evals with no matching test get no result instead of
        /// `skip` or `error`.
        #[arg(long)]
        partial: bool,
        /// The deployed site that `http` evals with a `path` are joined to.
        /// Defaults to $DEPLOY_URL.
        #[arg(long)]
        deploy_url: Option<String>,
        /// Exit with status 1 when an eval of a `must` requirement failed.
        /// The `cd` step after a deploy uses it.
        #[arg(long)]
        fail_on_must: bool,
        /// Tries per `http` eval before it fails, 5 seconds apart.
        #[arg(long, default_value_t = 3)]
        http_attempts: u32,
    },
    /// Settings made by hand (standards/settings.toml) that Jim checks on a
    /// schedule, and his answers (standards/settings-checks.jsonl).
    HumanChecks {
        #[command(subcommand)]
        command: HumanChecksCommand,
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
    /// Add results to a checkout of the `evals-results` branch: append each
    /// to results/<project>/<yyyy-mm>.jsonl, rewrite verdicts.json and the
    /// eval coverage badges (badge.svg and badges/<project>.svg).
    /// `standards/scripts/evals.sh store` commits and pushes it.
    Store {
        /// JSONL results files. Repeat for several.
        #[arg(long, required = true)]
        results: Vec<PathBuf>,
        /// The checkout of the `evals-results` branch.
        #[arg(long)]
        dir: PathBuf,
        /// Repo root (main), for the requirements, evals and human-check
        /// answers. Defaults as for `check`.
        #[arg(long)]
        root: Option<PathBuf>,
        /// The time to judge against (RFC 3339). Defaults to now.
        #[arg(long)]
        now: Option<chrono::DateTime<chrono::Utc>>,
    },
    /// Print every requirement's current verdict, from the results in a
    /// checkout of the `evals-results` branch and the requirements, evals and
    /// human-check answers on main. `standards/scripts/evals.sh verdicts`
    /// fetches the branch for you.
    Verdicts {
        /// The checkout of the `evals-results` branch.
        #[arg(long)]
        dir: PathBuf,
        /// Only this top-level project, such as `cookbook`.
        #[arg(long)]
        project: Option<String>,
        /// Print verdicts.json instead of text.
        #[arg(long)]
        json: bool,
        /// Repo root. Defaults as for `check`.
        #[arg(long)]
        root: Option<PathBuf>,
        /// The time to judge against (RFC 3339). Defaults to now.
        #[arg(long)]
        now: Option<chrono::DateTime<chrono::Utc>>,
    },
    /// File the work the verdicts call for: a `fix` issue for a requirement
    /// that fails in production, a `todo` for an eval that is broken, flaky
    /// or stale and for a `must` with no production evidence, unless an open
    /// issue file names the requirement. Without --write it only lists them.
    /// The daily evals-triage.yml workflow runs it.
    Triage {
        /// The checkout of the `evals-results` branch.
        #[arg(long)]
        dir: PathBuf,
        /// Write the issue files into the repo at --root.
        #[arg(long)]
        write: bool,
        /// With --write, write at most this many new issue files; the next
        /// run files the rest.
        #[arg(long, default_value_t = 10)]
        limit: usize,
        /// Repo root. Defaults as for `check`.
        #[arg(long)]
        root: Option<PathBuf>,
        /// The time to judge against (RFC 3339). Defaults to now.
        #[arg(long)]
        now: Option<chrono::DateTime<chrono::Utc>>,
    },
    /// The eval fire drill: break one `http` eval on purpose against the
    /// live site and check its requirement turns red, triage would file it,
    /// and the next good run turns it back. Reads a checkout of the
    /// `evals-results` branch and writes nothing; exit 1 when the loop
    /// didn't hold. See "Fire drill" in standards/docs/evals.md.
    Drill {
        /// The checkout of the `evals-results` branch (read only).
        #[arg(long)]
        dir: PathBuf,
        /// The top-level project directory.
        #[arg(long, default_value = "cookbook")]
        project: String,
        /// An `http` eval in the project's evals.toml.
        #[arg(long, default_value = "recipe-page-live")]
        eval: String,
        /// The live site. Defaults to DEPLOY_URL in a mise.toml of the
        /// project.
        #[arg(long)]
        deploy_url: Option<String>,
        /// Repo root. Defaults as for `check`.
        #[arg(long)]
        root: Option<PathBuf>,
        /// Where to append the Markdown report. Defaults to
        /// $GITHUB_STEP_SUMMARY, or standard output.
        #[arg(long)]
        summary: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum HumanChecksCommand {
    /// List each check that is due: never checked, last answered longer
    /// ago than its `every`, or last answered "not ok". With --write, also
    /// write its human task, standards/docs/issues/human/settings-check-<id>.md.
    Due {
        /// Write a task file for each due check that has none yet.
        #[arg(long)]
        write: bool,
        /// With --write, write at most this many new task files.
        #[arg(long, default_value_t = 5)]
        limit: usize,
        /// Repo root. Defaults as for `check`.
        #[arg(long)]
        root: Option<PathBuf>,
        /// The time to judge against (RFC 3339). Defaults to now.
        #[arg(long)]
        now: Option<chrono::DateTime<chrono::Utc>>,
    },
    /// Record Jim's answer to a check, appended to
    /// standards/settings-checks.jsonl, and delete its human task file.
    Answer {
        /// The setting's id in standards/settings.toml.
        setting: String,
        /// Jim said it is as expected.
        #[arg(long, conflicts_with = "not_ok", required_unless_present = "not_ok")]
        ok: bool,
        /// Jim said it is wrong: his words on what is wrong.
        #[arg(long, value_name = "WHAT_IS_WRONG")]
        not_ok: Option<String>,
        /// Repo root. Defaults as for `check`.
        #[arg(long)]
        root: Option<PathBuf>,
        /// When Jim answered (RFC 3339). Defaults to now.
        #[arg(long)]
        at: Option<chrono::DateTime<chrono::Utc>>,
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
            partial,
            deploy_url,
            fail_on_must,
            http_attempts,
        } => {
            let root = resolve_root(root)?;
            let evals_path = root.join(&project).join(check::EVALS);
            let text = fs::read_to_string(&evals_path)
                .with_context(|| format!("reading {}", evals_path.display()))?;
            let file = parse_evals(&text).map_err(|errors| {
                let list: Vec<String> = errors.iter().map(ToString::to_string).collect();
                anyhow!("{} is invalid: {}", evals_path.display(), list.join("; "))
            })?;

            let cargo_texts = read_all(&cargo_output)?;
            let reports = run::Reports {
                cargo: cargo_texts.as_ref().map(|texts| {
                    texts
                        .iter()
                        .flat_map(|t| reports::parse_cargo_output(t))
                        .collect()
                }),
                trycmd: cargo_texts.as_ref().map(|texts| {
                    texts
                        .iter()
                        .flat_map(|t| reports::parse_trycmd_output(t))
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
                partial,
                deploy_url: deploy_url.or_else(|| env("DEPLOY_URL")),
            };
            let client = http::Client::new();
            let runner = run::HttpRunner {
                fetch: &client,
                retry: http::Retry {
                    attempts: http_attempts,
                    ..http::Retry::default()
                },
            };
            let (results, notes) = run::collect(&file, &reports, &ctx, &runner);
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
            if fail_on_must {
                let path = root.join(&ctx.project).join(check::REQUIREMENTS);
                let text = fs::read_to_string(&path)
                    .with_context(|| format!("reading {}", path.display()))?;
                let (requirements, _) = parse_requirements(&text);
                let failed = run::failed_musts(&results, &requirements);
                if !failed.is_empty() {
                    for r in &failed {
                        eprintln!(
                            "evals: {} ({}) failed: {}",
                            r.eval,
                            r.requirement,
                            r.note.as_deref().unwrap_or_default()
                        );
                    }
                    eprintln!(
                        "evals run: {} eval(s) of a must requirement failed",
                        failed.len()
                    );
                    return Ok(ExitCode::FAILURE);
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::HumanChecks { command } => match command {
            HumanChecksCommand::Due {
                write,
                limit,
                root,
                now,
            } => {
                let root = resolve_root(root)?;
                let lines = human_checks::due(
                    &root,
                    now.unwrap_or_else(chrono::Utc::now),
                    write.then_some(limit),
                )?;
                if lines.is_empty() {
                    println!("No human checks are due.");
                } else {
                    print!("{lines}");
                }
                Ok(ExitCode::SUCCESS)
            }
            HumanChecksCommand::Answer {
                setting,
                ok,
                not_ok,
                root,
                at,
            } => {
                let root = resolve_root(root)?;
                let (status, note) = match not_ok {
                    Some(words) if !ok => (ResultStatus::Fail, Some(words)),
                    _ => (ResultStatus::Pass, Some("ok".to_string())),
                };
                let check = human_checks::answer(
                    &root,
                    &setting,
                    status,
                    note,
                    at.unwrap_or_else(chrono::Utc::now),
                )?;
                let path = root.join(check::SETTING_CHECKS);
                append(&path, &format!("{}\n", serde_json::to_string(&check)?))?;
                println!("evals: recorded {setting} as {}", status_word(status));
                if human_checks::close_task(&root, &setting)? {
                    println!(
                        "evals: deleted its task, {}/settings-check-{setting}.md",
                        human_checks::HUMAN_ISSUES
                    );
                }
                Ok(ExitCode::SUCCESS)
            }
        },
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
        Command::Store {
            results,
            dir,
            root,
            now,
        } => {
            let root = resolve_root(root)?;
            let mut new = Vec::new();
            for path in &results {
                let text = fs::read_to_string(path)
                    .with_context(|| format!("reading {}", path.display()))?;
                new.extend(publish::parse_results(&text)?);
            }
            let files = store::append_results(&dir, &new)?;
            for file in &files {
                println!("evals store: appended to {file}");
            }
            let all = store::read_results(&dir)?;
            let verdicts = store::compute(&root, &all, None, now.unwrap_or_else(chrono::Utc::now))?;
            store::write_verdicts(&dir, &verdicts)?;
            let badges = coverage::write_badges(&dir, &verdicts)?;
            println!(
                "evals store: wrote {} eval coverage badge(s), repo {}",
                badges.len(),
                coverage::summary(verdicts.coverage.percent, verdicts.coverage.musts_percent)
            );
            println!(
                "evals store: {} result(s) added, {} written",
                new.len(),
                store::VERDICTS_FILE
            );
            Ok(ExitCode::SUCCESS)
        }
        Command::Verdicts {
            dir,
            project,
            json,
            root,
            now,
        } => {
            let root = resolve_root(root)?;
            let all = store::read_results(&dir)?;
            let verdicts = store::compute(
                &root,
                &all,
                project.as_deref(),
                now.unwrap_or_else(chrono::Utc::now),
            )?;
            if json {
                println!("{}", serde_json::to_string_pretty(&verdicts)?);
            } else {
                print!("{}", store::render(&verdicts));
            }
            Ok(ExitCode::SUCCESS)
        }
        Command::Drill {
            dir,
            project,
            eval,
            deploy_url,
            root,
            summary,
        } => {
            let root = resolve_root(root)?;
            let deploy_url = match deploy_url {
                Some(url) => url,
                None => drill::deploy_url(&root, &project)?
                    .ok_or_else(|| anyhow!("no DEPLOY_URL for {project}; pass --deploy-url"))?,
            };
            let plan = drill::Drill {
                root: &root,
                dir: &dir,
                project: &project,
                eval: &eval,
                deploy_url: &deploy_url,
                now: chrono::Utc::now(),
            };
            let client = http::Client::new();
            let runner = run::HttpRunner {
                fetch: &client,
                retry: http::Retry::default(),
            };
            let report = drill::run(&plan, &runner)?;
            let markdown = report.markdown(&plan);
            let target = summary.or_else(|| {
                std::env::var_os("GITHUB_STEP_SUMMARY")
                    .filter(|v| !v.is_empty())
                    .map(PathBuf::from)
            });
            match target {
                Some(path) => append(&path, &markdown)?,
                None => print!("{markdown}"),
            }
            match &report.failure {
                None => {
                    println!("evals drill: passed ({})", report.requirement);
                    Ok(ExitCode::SUCCESS)
                }
                Some(why) => {
                    eprintln!("evals drill: FAILED: {why}");
                    Ok(ExitCode::FAILURE)
                }
            }
        }
        Command::Triage {
            dir,
            write,
            limit,
            root,
            now,
        } => {
            let root = resolve_root(root)?;
            let now = now.unwrap_or_else(chrono::Utc::now);
            let all = store::read_results(&dir)?;
            let verdicts = store::compute(&root, &all, None, now)?;
            let findings = triage::triage(&root, &verdicts, &all, now, write.then_some(limit))?;
            for finding in &findings {
                println!("{}", finding.line());
            }
            let wrote = findings
                .iter()
                .filter(|f| matches!(f.outcome, triage::Outcome::Wrote(_)))
                .count();
            println!(
                "evals triage: {} requirement(s) need work, {wrote} new issue file(s){}",
                findings.len(),
                if write { "" } else { " (dry run)" }
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn status_word(status: ResultStatus) -> &'static str {
    if status == ResultStatus::Pass {
        "ok"
    } else {
        "not ok"
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
