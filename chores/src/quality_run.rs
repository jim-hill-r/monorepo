//! `chores quality`: run clippy's strict profile and the file length check
//! over a project, and check, update or report its `docs/quality.toml`
//! (QUAL-501). The rules are in `quality`; this module runs the tools.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Serialize;

use crate::new_issue::today;
use crate::quality::{
    Baseline, Comparison, Finding, STRICT_FLAGS, check_date, compare, count, file_length_findings,
    new_findings, parse_clippy_json, top_lints,
};

/// What `chores quality` does with each project's findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Check,
    Update,
    Report,
}

/// The command's arguments.
#[derive(clap::Args)]
pub struct Args {
    /// Project folders (`cookbook`, `sparkhill/webapp`). Defaults to every
    /// Rust project.
    pub projects: Vec<String>,
    #[command(flatten)]
    pub modes: Modes,
    /// With --check, name only the findings in files changed since the
    /// merge base with this ref (such as `origin/main`) when any are.
    #[arg(long)]
    pub base: Option<String>,
    /// The baseline's `updated` date, YYYY-MM-DD. Defaults to today (UTC).
    #[arg(long)]
    pub date: Option<String>,
    /// With --report, print JSON with every finding.
    #[arg(long)]
    pub json: bool,
}

/// Exactly one of the three modes.
#[derive(clap::Args)]
#[group(required = true, multiple = false)]
pub struct Modes {
    /// Fail when any lint's count rose above the baseline, naming the
    /// findings.
    #[arg(long)]
    pub check: bool,
    /// Write each project's docs/quality.toml from its current counts.
    #[arg(long)]
    pub update: bool,
    /// Print each project's findings and baseline as a table.
    #[arg(long)]
    pub report: bool,
}

impl Modes {
    fn mode(&self) -> Mode {
        if self.check {
            Mode::Check
        } else if self.update {
            Mode::Update
        } else {
            Mode::Report
        }
    }
}

/// One project's measured findings.
#[derive(Debug, Serialize)]
pub struct Measured {
    pub project: String,
    pub total: u64,
    pub lints: BTreeMap<String, u64>,
    pub baseline_total: Option<u64>,
    pub comparison: Option<Comparison>,
    pub findings: Vec<Finding>,
}

const BASELINE: &str = "docs/quality.toml";
const SHARED_CONF: &str = "standards/quality";
const THRESHOLD_KEYS: &[&str] = &[
    "too-many-lines-threshold",
    "cognitive-complexity-threshold",
    "too-many-arguments-threshold",
    "excessive-nesting-threshold",
];

/// Run the command. Returns false when `--check` finds a count that rose
/// or a project without a baseline, or `--report` can't build a project.
///
/// # Errors
///
/// When a project has no `Cargo.toml`, clippy fails to build it, or a
/// baseline can't be read or written.
pub fn run(root: &Path, args: &Args) -> Result<bool> {
    if let Some(d) = &args.date {
        check_date(d)?;
    }
    let projects = if args.projects.is_empty() {
        discover_projects(root)?
    } else {
        args.projects
            .iter()
            .map(|p| normalize(root, p))
            .collect::<Result<_>>()?
    };
    let changed = match &args.base {
        Some(base) => changed_files(root, base)?,
        None => BTreeSet::new(),
    };
    let mode = args.modes.mode();
    let mut ok = true;
    let mut measured = Vec::new();
    for project in &projects {
        let m = match measure(root, project) {
            Ok(m) => m,
            // A report covers every project it can; one that doesn't
            // build (a missing system library) is named, not fatal.
            Err(e) if mode == Mode::Report => {
                eprintln!("error: {project}: {e:#}");
                ok = false;
                continue;
            }
            Err(e) => return Err(e),
        };
        ok &= match mode {
            Mode::Check => print_check(&m, &changed),
            Mode::Update => update(root, &m, args.date.as_deref())?,
            Mode::Report => true,
        };
        measured.push(m);
    }
    if mode == Mode::Report {
        if args.json {
            println!("{}", serde_json::to_string_pretty(&measured)?);
        } else {
            print!("{}", render_report(&measured));
        }
    }
    Ok(ok)
}

fn normalize(root: &Path, project: &str) -> Result<String> {
    let p = project.trim_end_matches('/').trim_start_matches("./");
    if !root.join(p).join("Cargo.toml").is_file() {
        bail!("{p} has no Cargo.toml; chores quality measures Rust projects");
    }
    Ok(p.to_string())
}

/// Every Rust project: each folder with a tracked `Cargo.toml` that no
/// folder above it also has (a workspace root or a lone crate), leaving out
/// test fixtures.
///
/// # Errors
///
/// When git can't list the files.
pub fn discover_projects(root: &Path) -> Result<Vec<String>> {
    let dirs: BTreeSet<String> = git_lines(root, &["ls-files", "--", "*Cargo.toml"])?
        .into_iter()
        .filter(|p| !p.contains("/fixtures/"))
        .filter_map(|p| p.strip_suffix("/Cargo.toml").map(String::from))
        .collect();
    Ok(outermost(&dirs))
}

fn outermost(dirs: &BTreeSet<String>) -> Vec<String> {
    dirs.iter()
        .filter(|d| !dirs.iter().any(|other| d.starts_with(&format!("{other}/"))))
        .cloned()
        .collect()
}

fn measure(root: &Path, project: &str) -> Result<Measured> {
    let mut findings = clippy_findings(root, project)?;
    findings.extend(file_length_findings(&rust_files(root, project)?));
    findings.sort();
    let lints = count(&findings);
    let baseline = read_baseline(root, project)?;
    Ok(Measured {
        project: project.to_string(),
        total: lints.values().sum(),
        baseline_total: baseline.as_ref().map(Baseline::total),
        comparison: baseline.as_ref().map(|b| compare(&b.lints, &lints)),
        lints,
        findings,
    })
}

fn clippy_findings(root: &Path, project: &str) -> Result<Vec<Finding>> {
    let dir = root.join(project);
    let conf = conf_dir(root, &dir);
    let out = Command::new("cargo")
        .current_dir(&dir)
        .env("CLIPPY_CONF_DIR", &conf)
        .args([
            "clippy",
            "--workspace",
            "--all-targets",
            "--message-format=json",
            "--",
        ])
        .args(STRICT_FLAGS)
        .output()
        .with_context(|| format!("running cargo clippy in {project}"))?;
    if !out.status.success() {
        bail!(
            "cargo clippy failed in {project}:\n{}",
            tail(&String::from_utf8_lossy(&out.stderr), 20)
        );
    }
    parse_clippy_json(&String::from_utf8_lossy(&out.stdout), project)
}

/// The project's own `clippy.toml` folder when it has one (cahokia, for
/// its Dioxus rules), otherwise the shared strict profile. A project's own
/// file should copy the shared thresholds; say so when it doesn't.
fn conf_dir(root: &Path, dir: &Path) -> PathBuf {
    let own = dir.join("clippy.toml");
    match std::fs::read_to_string(&own) {
        Ok(text) => {
            let missing: Vec<&str> = THRESHOLD_KEYS
                .iter()
                .copied()
                .filter(|k| !text.contains(k))
                .collect();
            if !missing.is_empty() {
                eprintln!(
                    "note: {} lacks the shared thresholds ({}); copy them from {SHARED_CONF}/clippy.toml",
                    own.display(),
                    missing.join(", ")
                );
            }
            dir.to_path_buf()
        }
        Err(_) => root.join(SHARED_CONF),
    }
}

fn rust_files(root: &Path, project: &str) -> Result<Vec<(String, String)>> {
    let mut files = Vec::new();
    for path in git_lines(root, &["ls-files", "--", &format!("{project}/*.rs")])? {
        if path.contains("/fixtures/") {
            continue;
        }
        let text =
            std::fs::read_to_string(root.join(&path)).with_context(|| format!("reading {path}"))?;
        files.push((path, text));
    }
    Ok(files)
}

fn read_baseline(root: &Path, project: &str) -> Result<Option<Baseline>> {
    let path = root.join(project).join(BASELINE);
    match std::fs::read_to_string(&path) {
        Ok(text) => Baseline::parse(&text)
            .with_context(|| format!("in {}", path.display()))
            .map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

fn print_check(m: &Measured, changed: &BTreeSet<String>) -> bool {
    let Some(c) = &m.comparison else {
        println!(
            "error[QUAL-501] {}: no {BASELINE}; write one with `chores quality --update {}`",
            m.project, m.project
        );
        return false;
    };
    print!("{}", render_check(m, c, changed));
    c.rose.is_empty()
}

fn render_check(m: &Measured, c: &Comparison, changed: &BTreeSet<String>) -> String {
    let mut s = String::new();
    for r in &c.rose {
        let _ = writeln!(
            s,
            "error[QUAL-501] {}: {} rose from {} to {}",
            m.project, r.lint, r.before, r.after
        );
    }
    for f in new_findings(c, &m.findings, changed) {
        let _ = writeln!(s, "  {}", f.render());
    }
    if !c.fell.is_empty() {
        let fell: Vec<String> = c
            .fell
            .iter()
            .map(|f| format!("{} {} to {}", f.lint, f.before, f.after))
            .collect();
        let _ = writeln!(
            s,
            "{}: fewer findings ({}); lower the baseline with `chores quality --update {}`",
            m.project,
            fell.join(", "),
            m.project
        );
    }
    if c.rose.is_empty() {
        let _ = writeln!(
            s,
            "{}: {} findings, baseline {}",
            m.project,
            m.total,
            m.baseline_total.unwrap_or(0)
        );
    }
    s
}

fn update(root: &Path, m: &Measured, date: Option<&str>) -> Result<bool> {
    let baseline = Baseline {
        updated: date.map_or_else(today, String::from),
        lints: m.lints.clone(),
    };
    let path = root.join(&m.project).join(BASELINE);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, baseline.render())
        .with_context(|| format!("writing {}", path.display()))?;
    let before = m
        .baseline_total
        .map_or_else(|| "none".to_string(), |t| t.to_string());
    println!(
        "wrote {}/{BASELINE}: {} findings (was {before})",
        m.project, m.total
    );
    if m.comparison.as_ref().is_some_and(|c| !c.rose.is_empty()) {
        eprintln!(
            "warning[QUAL-501] {}: the baseline rose; fix the new findings instead unless a reviewer agrees",
            m.project
        );
    }
    Ok(true)
}

fn render_report(measured: &[Measured]) -> String {
    let mut s =
        String::from("| Project | Findings | Baseline | Top lints |\n| --- | --- | --- | --- |\n");
    for m in measured {
        let top: Vec<String> = top_lints(&m.lints, 3)
            .into_iter()
            .map(|(l, n)| format!("`{l}` {n}"))
            .collect();
        let baseline = m
            .baseline_total
            .map_or_else(|| "none".to_string(), |t| t.to_string());
        let _ = writeln!(
            s,
            "| {} | {} | {baseline} | {} |",
            m.project,
            m.total,
            top.join(", ")
        );
    }
    let total: u64 = measured.iter().map(|m| m.total).sum();
    let _ = writeln!(s, "\n{total} findings in {} projects", measured.len());
    s
}

fn changed_files(root: &Path, base: &str) -> Result<BTreeSet<String>> {
    let mb = git_lines(root, &["merge-base", base, "HEAD"])?;
    let Some(mb) = mb.first() else {
        bail!("no merge base with {base}");
    };
    let mut files: BTreeSet<String> = git_lines(root, &["diff", "--name-only", mb])?
        .into_iter()
        .collect();
    files.extend(git_lines(
        root,
        &["ls-files", "--others", "--exclude-standard"],
    )?);
    Ok(files)
}

/// The last `n` lines of `text`, so a failed build names its error
/// without the whole download log.
fn tail(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

fn git_lines(root: &Path, args: &[&str]) -> Result<Vec<String>> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(String::from)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measured(lints: &[(&str, u64)], baseline: &[(&str, u64)]) -> Measured {
        let lints: BTreeMap<String, u64> =
            lints.iter().map(|(l, n)| ((*l).to_string(), *n)).collect();
        let base: BTreeMap<String, u64> = baseline
            .iter()
            .map(|(l, n)| ((*l).to_string(), *n))
            .collect();
        let findings = lints
            .iter()
            .flat_map(|(l, n)| {
                (0..*n).map(move |i| Finding {
                    lint: l.clone(),
                    file: "demo/src/lib.rs".into(),
                    line: i + 1,
                    column: 1,
                    message: "m".into(),
                })
            })
            .collect();
        Measured {
            project: "demo".into(),
            total: lints.values().sum(),
            baseline_total: Some(base.values().sum()),
            comparison: Some(compare(&base, &lints)),
            lints,
            findings,
        }
    }

    #[test]
    fn tail_keeps_the_last_lines() {
        assert_eq!(tail("a\nb\nc", 2), "b\nc");
        assert_eq!(tail("a", 5), "a");
    }

    #[test]
    fn outermost_keeps_workspace_roots_and_lone_crates() {
        let dirs: BTreeSet<String> = [
            "cookbook",
            "cookbook/web",
            "sparkhill/webapp",
            "sparkhill/website",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
        assert_eq!(
            outermost(&dirs),
            vec!["cookbook", "sparkhill/webapp", "sparkhill/website"]
        );
    }

    #[test]
    fn outermost_does_not_treat_a_name_prefix_as_a_parent() {
        let dirs: BTreeSet<String> = ["cost", "cost_extra"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert_eq!(outermost(&dirs), vec!["cost", "cost_extra"]);
    }

    #[test]
    fn check_fails_on_a_rise_and_names_findings() {
        let m = measured(&[("clippy::a", 2)], &[("clippy::a", 1)]);
        let c = m.comparison.clone().expect("comparison");
        let text = render_check(&m, &c, &BTreeSet::new());
        assert!(text.contains("error[QUAL-501] demo: clippy::a rose from 1 to 2"));
        assert!(text.contains("demo/src/lib.rs:2:1: clippy::a: m"));
        assert!(!print_check(&m, &BTreeSet::new()));
    }

    #[test]
    fn check_passes_on_a_fall_and_asks_for_update() {
        let m = measured(&[("clippy::a", 1)], &[("clippy::a", 3)]);
        let c = m.comparison.clone().expect("comparison");
        let text = render_check(&m, &c, &BTreeSet::new());
        assert!(text.contains("chores quality --update demo"));
        assert!(text.contains("demo: 1 findings, baseline 3"));
        assert!(print_check(&m, &BTreeSet::new()));
    }

    #[test]
    fn check_fails_without_a_baseline() {
        let mut m = measured(&[], &[]);
        m.comparison = None;
        assert!(!print_check(&m, &BTreeSet::new()));
    }

    #[test]
    fn report_is_a_table_with_a_total() {
        let text = render_report(&[measured(&[("clippy::a", 2), ("clippy::b", 1)], &[])]);
        assert!(text.contains("| demo | 3 | 0 | `clippy::a` 2, `clippy::b` 1 |"));
        assert!(text.ends_with("3 findings in 1 projects\n"));
    }

    #[test]
    fn update_writes_the_baseline() {
        let dir = tempfile::tempdir().expect("tempdir");
        let m = measured(&[("clippy::a", 2)], &[]);
        assert!(update(dir.path(), &m, Some("2026-11-28")).expect("update"));
        let text = std::fs::read_to_string(dir.path().join("demo/docs/quality.toml"))
            .expect("baseline written");
        let b = Baseline::parse(&text).expect("parses");
        assert_eq!(b.updated, "2026-11-28");
        assert_eq!(b.total(), 2);
    }

    #[test]
    fn a_project_without_cargo_toml_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(normalize(dir.path(), "nothing/").is_err());
        std::fs::create_dir_all(dir.path().join("demo")).expect("mkdir");
        std::fs::write(dir.path().join("demo/Cargo.toml"), "").expect("write");
        assert_eq!(normalize(dir.path(), "./demo/").expect("ok"), "demo");
    }

    #[test]
    fn own_clippy_toml_wins_over_the_shared_profile() {
        let dir = tempfile::tempdir().expect("tempdir");
        let project = dir.path().join("demo");
        std::fs::create_dir_all(&project).expect("mkdir");
        assert_eq!(conf_dir(dir.path(), &project), dir.path().join(SHARED_CONF));
        std::fs::write(project.join("clippy.toml"), "").expect("write");
        assert_eq!(conf_dir(dir.path(), &project), project);
    }

    #[test]
    fn shared_profile_has_every_threshold() {
        let text = include_str!("../../standards/quality/clippy.toml");
        for key in THRESHOLD_KEYS {
            assert!(text.contains(key), "{key}");
        }
        assert!(STRICT_FLAGS.contains(&"clippy::pedantic"));
    }
}
