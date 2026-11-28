//! Audit checks that run another project's CLI and only warn: EVL-* (the
//! evals project's `evals check`) and THR-* (the threat project's `threat
//! check`). Ported from `standards/scripts/evals-check.sh` and
//! `threat-check.sh` (code quality phase 2, QUAL-404).
//!
//! Both are warnings only until their plans' later phases, so neither ever
//! fails the audit: when cargo is missing or the CLI can't run, they print a
//! warning and succeed. PR CI checks them.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

/// One wrapped check: which CLI to run and what to call it in warnings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrapped {
    /// The name in warnings, such as `evals check`.
    pub label: &'static str,
    /// The arguments to `cargo`.
    pub cargo_args: Vec<String>,
}

fn cargo_run(manifest: &str, package: &str, label: &'static str, args: &[String]) -> Wrapped {
    let mut cargo_args: Vec<String> = [
        "run",
        "--quiet",
        "--manifest-path",
        manifest,
        "-p",
        package,
        "--",
        "check",
        "--root",
        ".",
    ]
    .iter()
    .map(ToString::to_string)
    .collect();
    cargo_args.extend(args.iter().cloned());
    Wrapped { label, cargo_args }
}

/// The files that turn the evals check on (EVL-*): any project's
/// `docs/requirements.md` or `evals.toml`.
pub const EVALS_PATHSPECS: [&str; 3] = ["evals.toml", "*/evals.toml", "*/docs/requirements.md"];

/// The evals check, or `None` when no tracked file needs it, so the audit
/// doesn't build the evals CLI for nothing.
pub fn evals(root: &Path) -> Result<Option<Wrapped>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "--"])
        .args(EVALS_PATHSPECS)
        .output()
        .context("running git ls-files")?;
    anyhow::ensure!(
        out.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    if out.stdout.iter().all(u8::is_ascii_whitespace) {
        return Ok(None);
    }
    Ok(Some(cargo_run(
        "evals/Cargo.toml",
        "evals_cli",
        "evals check",
        &[],
    )))
}

/// The threat check (THR-*). Its own arguments win; without any,
/// `THREAT_PROJECTS`, or else `COST_PROJECTS` (hooks.sh and PR CI set it to
/// the affected projects), becomes `--projects` when set, even empty.
pub fn threat(
    args: &[String],
    threat_projects: Option<&str>,
    cost_projects: Option<&str>,
) -> Wrapped {
    let projects = if args.is_empty() {
        threat_projects.or(cost_projects)
    } else {
        None
    };
    let args: Vec<String> = match projects {
        Some(p) => vec!["--projects".to_string(), p.to_string()],
        None => args.to_vec(),
    };
    cargo_run("threat/Cargo.toml", "threat_cli", "threat check", &args)
}

/// What happened when a wrapped check ran. None of these fail the audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The CLI ran and exited 0.
    Ran,
    /// The CLI ran (or failed to build) and exited non-zero.
    Broken,
    /// cargo isn't installed.
    NoCargo,
}

/// The warning to print for an outcome, if any.
pub fn warning(label: &str, outcome: Outcome) -> Option<String> {
    match outcome {
        Outcome::Ran => None,
        Outcome::Broken => Some(format!(
            "warning: {label} could not run; see the output above"
        )),
        Outcome::NoCargo => Some(format!("warning: {label} skipped: cargo is not installed")),
    }
}

/// Run a wrapped check with `cargo` from `root`, its output going straight
/// to ours.
pub fn run(root: &Path, check: &Wrapped) -> Outcome {
    match Command::new("cargo")
        .args(&check.cargo_args)
        .current_dir(root)
        .status()
    {
        Ok(status) if status.success() => Outcome::Ran,
        Ok(_) => Outcome::Broken,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Outcome::NoCargo,
        Err(_) => Outcome::Broken,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joined(w: &Wrapped) -> String {
        w.cargo_args.join(" ")
    }

    fn git(dir: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env_remove("GIT_DIR")
            .env_remove("GIT_INDEX_FILE")
            .env_remove("GIT_WORK_TREE")
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }

    const THREAT: &str =
        "run --quiet --manifest-path threat/Cargo.toml -p threat_cli -- check --root .";

    #[test]
    fn evals_skips_cargo_without_files_and_runs_with_them() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q"]);
        assert_eq!(evals(dir.path()).unwrap(), None);

        // An untracked file doesn't count, as with git ls-files in the script.
        std::fs::create_dir_all(dir.path().join("cookbook/docs")).unwrap();
        std::fs::write(dir.path().join("cookbook/docs/requirements.md"), "# R\n").unwrap();
        assert_eq!(evals(dir.path()).unwrap(), None);

        git(dir.path(), &["add", "cookbook/docs/requirements.md"]);
        let check = evals(dir.path()).unwrap().unwrap();
        assert_eq!(check.label, "evals check");
        assert_eq!(
            joined(&check),
            "run --quiet --manifest-path evals/Cargo.toml -p evals_cli -- check --root ."
        );
    }

    #[test]
    fn evals_counts_a_root_or_project_evals_toml() {
        let dir = tempfile::tempdir().unwrap();
        git(dir.path(), &["init", "-q"]);
        std::fs::create_dir_all(dir.path().join("pane")).unwrap();
        std::fs::write(dir.path().join("pane/evals.toml"), "").unwrap();
        git(dir.path(), &["add", "pane/evals.toml"]);
        assert!(evals(dir.path()).unwrap().is_some());
    }

    #[test]
    fn threat_checks_every_project_by_default() {
        assert_eq!(joined(&threat(&[], None, None)), THREAT);
    }

    #[test]
    fn threat_passes_its_own_arguments() {
        let args = ["--projects".to_string(), "cookbook,pane".to_string()];
        assert_eq!(
            joined(&threat(&args, None, Some("bear_down"))),
            format!("{THREAT} --projects cookbook,pane")
        );
        let strict = ["--strict".to_string()];
        assert_eq!(
            joined(&threat(&strict, None, Some("cookbook"))),
            format!("{THREAT} --strict")
        );
    }

    #[test]
    fn threat_takes_the_affected_projects() {
        assert_eq!(
            joined(&threat(&[], None, Some("cookbook,pane"))),
            format!("{THREAT} --projects cookbook,pane")
        );
        assert_eq!(
            joined(&threat(&[], Some("pane"), Some("cookbook"))),
            format!("{THREAT} --projects pane"),
            "THREAT_PROJECTS wins"
        );
        // Set but empty: no project is affected, so check none.
        assert_eq!(
            joined(&threat(&[], None, Some(""))),
            format!("{THREAT} --projects ")
        );
    }

    #[test]
    fn warnings_never_fail_and_say_why() {
        assert_eq!(warning("threat check", Outcome::Ran), None);
        assert_eq!(
            warning("threat check", Outcome::Broken).unwrap(),
            "warning: threat check could not run; see the output above"
        );
        assert_eq!(
            warning("evals check", Outcome::NoCargo).unwrap(),
            "warning: evals check skipped: cargo is not installed"
        );
    }
}
