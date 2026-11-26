//! `pane_snapshot`: write `snapshot.json` for a checkout of the monorepo.
//!
//! ```text
//! pane_snapshot [--repo DIR] [--out FILE] [--commit SHA] [--taken-at TIME]
//! ```
//!
//! `--repo` defaults to the git top level of the current directory, `--out`
//! to `snapshot.json`, `--commit` to `git rev-parse HEAD` in the repo, and
//! `--taken-at` to now (UTC).

use std::path::PathBuf;
use std::process::{Command, ExitCode};

use pane_snapshot::{repo, time};

struct Args {
    repo: Option<PathBuf>,
    out: PathBuf,
    commit: Option<String>,
    taken_at: Option<String>,
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut parsed = Args {
        repo: None,
        out: PathBuf::from("snapshot.json"),
        commit: None,
        taken_at: None,
    };
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{flag} needs a value"));
        match flag.as_str() {
            "--repo" => parsed.repo = Some(PathBuf::from(value()?)),
            "--out" => parsed.out = PathBuf::from(value()?),
            "--commit" => parsed.commit = Some(value()?),
            "--taken-at" => parsed.taken_at = Some(value()?),
            "-h" | "--help" => {
                return Err(
                    "usage: pane_snapshot [--repo DIR] [--out FILE] [--commit SHA] [--taken-at TIME]"
                        .to_owned(),
                );
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(parsed)
}

fn git(dir: Option<&PathBuf>, args: &[&str]) -> Option<String> {
    let mut cmd = Command::new("git");
    if let Some(dir) = dir {
        cmd.arg("-C").arg(dir);
    }
    let out = cmd.args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn run() -> Result<(), String> {
    let args = parse_args(std::env::args().skip(1))?;
    let repo_dir = match args.repo {
        Some(dir) => dir,
        None => git(None, &["rev-parse", "--show-toplevel"])
            .map(PathBuf::from)
            .ok_or("not in a git checkout; pass --repo")?,
    };
    let commit = args
        .commit
        .or_else(|| git(Some(&repo_dir), &["rev-parse", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_owned());
    let taken_at = args.taken_at.unwrap_or_else(time::now_rfc3339);

    let snapshot = repo::build(&repo_dir, &commit, &taken_at)?;
    let json = snapshot.to_json().map_err(|e| e.to_string())?;
    if let Some(parent) = args.out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    std::fs::write(&args.out, json).map_err(|e| format!("{}: {e}", args.out.display()))?;
    eprintln!(
        "pane_snapshot: {} projects, {} work items -> {}",
        snapshot.projects.len(),
        snapshot.work_items.len(),
        args.out.display()
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("pane_snapshot: {e}");
            ExitCode::FAILURE
        }
    }
}
