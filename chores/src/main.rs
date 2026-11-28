//! `chores`: deterministic commands for routine agent work, so agents spend
//! tokens only on novel work. See `chores/README.md` and the plan in
//! `standards/docs/plans/2026-11-baked-skills.md`.

#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

mod badges;
mod gate;
mod issues;
mod new_issue;
mod next_task;
mod pr_body;
mod tiers;
mod user_docs;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "chores",
    about = "Deterministic commands for routine agent work"
)]
struct Cli {
    /// Repo root. Defaults to the nearest directory above the current one
    /// that holds AGENTS.md and standards/docs/issues/README.md.
    #[arg(long, global = true)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Pick the next issue to work, in the start-a-new-task skill's order,
    /// and print it with its text and epic notes.
    NextTask {
        /// Only issues of these projects (`cookbook`, `cahokia/web`, or
        /// `repo` for repo-wide work in standards). Comma separated or repeated.
        #[arg(long, value_delimiter = ',')]
        project: Vec<String>,
        /// Skip these projects or epics (`epic:<name>`), such as ones other
        /// threads are working. Comma separated or repeated.
        #[arg(long, value_delimiter = ',')]
        exclude: Vec<String>,
        /// Skip these issue files (paths from the repo root), such as ones
        /// an open PR already deletes.
        #[arg(long, value_delimiter = ',')]
        skip: Vec<String>,
        /// How many to pick, each from a different project or epic.
        #[arg(long, default_value_t = 1)]
        count: usize,
        /// Projects (or `epic:<name>`, `repo`) of the threads working now,
        /// one entry per thread, for the product/platform balance. Comma
        /// separated or repeated.
        #[arg(long, value_delimiter = ',')]
        working: Vec<String>,
        /// Days of `main` history to count merged PRs by tier (0 for none).
        #[arg(long, default_value_t = 7)]
        days: u32,
        /// Use the plain order, without preferring product issues when
        /// products are under half of recent work.
        #[arg(long)]
        no_balance: bool,
        /// Print JSON instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Write a PR description (attribution lines, Attention, What changed,
    /// Why, Cost changes, Risk changes, Details, footer) and check it
    /// against PR-001. Exits 1, printing the warnings, when a section is
    /// missing or empty, an attention tag is unknown, or a summary is over
    /// its word limit. With --check, checks an existing description instead.
    PrBody {
        /// Check this description file (`-` for stdin) instead of writing one.
        #[arg(long, conflicts_with_all = ["what", "why", "cost", "risk", "details"])]
        check: Option<PathBuf>,
        /// What changed, at most 50 words.
        #[arg(long, required_unless_present = "check")]
        what: Option<String>,
        /// Why it changed, and any tradeoffs, at most 50 words.
        #[arg(long, required_unless_present = "check")]
        why: Option<String>,
        /// Cost changes, under 25 words ("None." when nothing changes).
        #[arg(long, default_value = "None.")]
        cost: String,
        /// Risk changes, under 25 words ("None." when nothing changes).
        #[arg(long, default_value = "None.")]
        risk: String,
        /// Anything else: the issue it closes, testing, review notes, links.
        #[arg(long, required_unless_present = "check")]
        details: Option<String>,
        /// GitHub login for the claude-projects-attribution comment.
        #[arg(long)]
        login: Option<String>,
        /// Who asked for the work, for the "Requested by" line.
        #[arg(long)]
        requested_by: Option<String>,
        /// The project thread link for the "Requested by" line.
        #[arg(long)]
        thread_url: Option<String>,
        /// Closing lines, such as the session link the platform asks for.
        #[arg(long)]
        footer: Option<String>,
        /// The branch the PR merges into, for the attention tags.
        #[arg(long, default_value = "origin/main")]
        base: String,
        /// Leave out the Attention section (only when git can't compare
        /// with the base; say so in the PR).
        #[arg(long)]
        no_badges: bool,
        /// Also write the description to this file.
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Print the Attention section for the top of a PR description: whether
    /// a requirement changed (red for a must) and whether a threat or risk
    /// file changed, against the merge base with --base.
    Badges {
        /// The branch the PR merges into.
        #[arg(long, default_value = "origin/main")]
        base: String,
        /// Print JSON instead of the Markdown line.
        #[arg(long)]
        json: bool,
    },
    /// The `human-review` label (PR-004): a PR with a red attention tag or a
    /// cost change, from its diff, its description or the `cost` label,
    /// passes only once a human (not an `-agt` account, the reviewer account
    /// or a bot) has approved its head commit.
    /// Prints "pass: ..." or "hold: ..."; exits 1 while it holds.
    Gate {
        /// The branch the PR merges into, for the diff's tags.
        #[arg(long, default_value = "origin/main")]
        base: String,
        /// The PR description, as a file.
        #[arg(long)]
        body: Option<PathBuf>,
        /// The PR's labels, comma separated.
        #[arg(long, default_value = "", value_delimiter = ',')]
        labels: Vec<String>,
        /// The PR author's login.
        #[arg(long)]
        author: String,
        /// The PR author's repo role (`author_association`, such as OWNER
        /// or COLLABORATOR); only a collaborator's own PR passes.
        #[arg(long, default_value = "NONE")]
        author_association: String,
        /// The PR's head commit.
        #[arg(long)]
        head: String,
        /// The PR's reviews as JSON, from `GET /repos/{repo}/pulls/{n}/reviews`.
        #[arg(long)]
        reviews: Option<PathBuf>,
        /// Also write the verdict as JSON to this file.
        #[arg(long)]
        json: Option<PathBuf>,
    },
    /// Check every project's user guide, <project>/docs/user/README.md
    /// (DOC-005 to DOC-008, "User docs" in standards/docs/documentation.md),
    /// against the baseline in standards/user-docs.toml. Exits 1 on any
    /// error. Run by `mise run //:audit`.
    UserDocs {
        /// Print JSON instead of text.
        #[arg(long)]
        json: bool,
    },
    /// Write an agent issue file, <folder>/docs/issues/<slug>.md, tagged
    /// agent-generated and dated today.
    NewIssue {
        /// The project or sub-project folder (`cookbook`, `cahokia/web`), or
        /// `.` for repo-wide work, which lives in standards.
        #[arg(long)]
        folder: String,
        /// Kebab-case file name without .md.
        #[arg(long)]
        slug: String,
        #[arg(long, default_value = "todo", value_parser = ["todo", "fix"])]
        kind: String,
        #[arg(long, default_value = "backlog", value_parser = ["priority", "backlog"])]
        priority: String,
        /// `Epic` or `Epic / Phase N: name`.
        #[arg(long)]
        epic: Option<String>,
        /// Orders the issues of one epic; use steps of 10.
        #[arg(long)]
        order: Option<u64>,
        /// Tag it agent-ignore as well, so agents leave it alone.
        #[arg(long)]
        ignore: bool,
        /// The issue's text: what to do and why, with links.
        #[arg(long)]
        text: String,
        /// The `added` date, YYYY-MM-DD. Defaults to today (UTC).
        #[arg(long)]
        date: Option<String>,
    },
    /// Write a task only a person can do, <folder>/docs/issues/human/<slug>.md, with the
    /// three sections the audit needs, and start its runbook from
    /// standards/docs/runbooks/TEMPLATE.md unless --runbook names one that
    /// exists.
    NewHumanTask {
        /// The project the task belongs to (`cost`, `infra`); repo-wide
        /// tasks go in `standards`.
        #[arg(long, default_value = "standards")]
        folder: String,
        /// Kebab-case file name without .md.
        #[arg(long)]
        slug: String,
        /// The runbook's title and link text, in plain words.
        #[arg(long)]
        title: String,
        /// What to do, where, and what it unblocks.
        #[arg(long)]
        requirement: String,
        /// Why no workflow or agent can do it.
        #[arg(long)]
        why: String,
        /// What it unblocks, for the front matter.
        #[arg(long)]
        unblocks: Option<String>,
        /// The runbook's file name without .md. Defaults to the slug.
        #[arg(long)]
        runbook: Option<String>,
        /// The `added` date, YYYY-MM-DD. Defaults to today (UTC).
        #[arg(long)]
        date: Option<String>,
    },
}

fn find_root(start: &Path) -> Result<PathBuf> {
    for dir in start.ancestors() {
        if dir.join("AGENTS.md").is_file() && dir.join("standards/docs/issues/README.md").is_file()
        {
            return Ok(dir.to_path_buf());
        }
    }
    bail!(
        "no repo root (a folder with AGENTS.md and standards/docs/issues/README.md) above {}; pass --root",
        start.display()
    )
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("chores: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let root = match cli.root {
        Some(r) => r,
        None => find_root(&std::env::current_dir()?)?,
    };
    match cli.command {
        Command::NextTask {
            project,
            exclude,
            skip,
            count,
            working,
            days,
            no_balance,
            json,
        } => {
            let options = next_task::Options {
                projects: project,
                exclude,
                skip,
                count,
                working,
                days,
                no_balance,
            };
            let outcome = next_task::run(&root, &options)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&outcome)?);
            } else {
                print!("{}", next_task::render(&outcome));
            }
        }
        Command::PrBody {
            check,
            what,
            why,
            cost,
            risk,
            details,
            login,
            requested_by,
            thread_url,
            footer,
            base,
            no_badges,
            out,
        } => {
            let body = match check {
                Some(path) => read_input(&path)?,
                None => pr_body::render(&pr_body::Parts {
                    login,
                    requested_by,
                    thread_url,
                    attention: if no_badges {
                        None
                    } else {
                        Some(badges::render(&badges::for_repo(&root, &base)?))
                    },
                    what: what.unwrap_or_default(),
                    why: why.unwrap_or_default(),
                    cost,
                    risk,
                    details: details.unwrap_or_default(),
                    footer,
                }),
            };
            let (counts, warnings) = pr_body::check(&body);
            if let Some(out) = out {
                std::fs::write(&out, &body)?;
            }
            if warnings.is_empty() {
                print!("{body}");
                eprintln!("{}", pr_body::ok_line(&counts));
            } else {
                for w in &warnings {
                    eprintln!("warning[PR-001] {w}");
                }
                return Ok(ExitCode::FAILURE);
            }
        }
        Command::Badges { base, json } => {
            let attention = badges::for_repo(&root, &base)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&attention)?);
            } else {
                println!("{}", badges::render(&attention));
            }
        }
        Command::Gate {
            base,
            body,
            labels,
            author,
            author_association,
            head,
            reviews,
            json,
        } => {
            let args = gate::Args {
                base,
                body,
                labels,
                author,
                author_association,
                head,
                reviews,
                json,
            };
            if !gate::run(&root, &args)? {
                return Ok(ExitCode::FAILURE);
            }
        }
        Command::UserDocs { json } => {
            let report = user_docs::check(&root)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print!("{}", user_docs::render(&report));
            }
            if !report.errors.is_empty() {
                return Ok(ExitCode::FAILURE);
            }
        }
        Command::NewIssue {
            folder,
            slug,
            kind,
            priority,
            epic,
            order,
            ignore,
            text,
            date,
        } => {
            let path = new_issue::write_agent_issue(
                &root,
                &new_issue::AgentIssue {
                    folder,
                    slug,
                    kind,
                    priority,
                    epic,
                    order,
                    ignore,
                    text,
                    date,
                },
            )?;
            println!("wrote {path}");
        }
        Command::NewHumanTask {
            folder,
            slug,
            title,
            requirement,
            why,
            unblocks,
            runbook,
            date,
        } => {
            let (task, runbook, new) = new_issue::write_human_task(
                &root,
                &new_issue::HumanTask {
                    folder,
                    slug,
                    title,
                    requirement,
                    why,
                    unblocks,
                    runbook,
                    date,
                },
            )?;
            println!("wrote {task}");
            if new {
                println!(
                    "wrote {runbook} from TEMPLATE.md: fill in every section (Kind, Time, Cost, \
                     Steps, Where secrets go, Verify, Undo) before committing"
                );
            } else {
                println!("linked the existing runbook {runbook}");
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn read_input(path: &Path) -> Result<String> {
    if path.as_os_str() == "-" {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut s)?;
        Ok(s)
    } else {
        Ok(std::fs::read_to_string(path)?)
    }
}
