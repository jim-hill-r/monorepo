//! `chores quality`: the code quality ratchet (QUAL-501 in
//! standards/docs/code-quality.md, decision 0015).
//!
//! For each project it runs clippy with the Rust strict profile in
//! `standards/quality/` (`--all-targets --message-format=json`), adds a
//! finding for each Rust file over the length limit, and compares the counts
//! per lint with the project's committed `docs/quality.toml`:
//!
//!   --check   fails when any count rose, naming that lint's findings
//!   --update  writes the baseline, refusing a rise unless told why
//!   --report  every project's totals, worst lints and worst files
//!
//! The parsing and comparing are pure (`findings`, `baseline`); this module
//! is the I/O around them: git for the file list, cargo for clippy.

pub mod baseline;
pub mod findings;
pub mod source;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use baseline::{Baseline, Change};
use findings::{FILE_TOO_LONG, Finding, Scope};
use source::{
    cargo_metadata, cargo_roots, is_manifest, line_counts, run_clippy, rust_projects, tracked_files,
};

/// The strict profile's folder, from the repo root. Clippy reads its
/// `clippy.toml` through `CLIPPY_CONF_DIR`.
pub const PROFILE_DIR: &str = "standards/quality";

/// How many findings of a risen lint `--check` prints.
const SHOW_FINDINGS: usize = 20;

/// How many lints and files `--report` lists per project.
const SHOW_WORST: usize = 5;

/// The Rust strict profile, `standards/quality/profile.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Profile {
    pub warn: Vec<String>,
    pub allow: Vec<String>,
    pub max_file_lines: usize,
    #[serde(default)]
    pub generated: Vec<String>,
}

impl Profile {
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(PROFILE_DIR).join("profile.toml");
        let text =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }

    /// The lint flags after clippy's `--`: warnings first, so the allowed
    /// lints, which come later, win.
    pub fn clippy_args(&self) -> Vec<String> {
        let warn = self.warn.iter().flat_map(|l| ["-W".to_string(), l.clone()]);
        let allow = self
            .allow
            .iter()
            .flat_map(|l| ["-A".to_string(), l.clone()]);
        warn.chain(allow).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Check,
    Update,
    Report,
}

#[derive(Debug, Clone)]
pub struct Options {
    /// Project folders from the repo root; empty means every folder with
    /// Rust code.
    pub projects: Vec<String>,
    pub mode: Mode,
    /// Let `--update` raise a count (a toolchain or profile change).
    pub accept_rise: bool,
    /// The baseline's `updated` date; defaults to today (UTC).
    pub date: Option<String>,
    /// With `--check`, the branch the change merges into: a risen lint
    /// lists only its findings in files changed since the merge base.
    pub base: Option<String>,
}

/// One project measured now, next to its committed baseline.
#[derive(Debug, Clone, Serialize)]
pub struct Measured {
    pub project: String,
    pub findings: Vec<Finding>,
    pub counts: BTreeMap<String, u64>,
    pub baseline: Option<Baseline>,
    /// Why the project could not be measured, such as a build that needs
    /// system packages the sandbox lacks. Only `--report` keeps going.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Measured {
    fn failed(project: &str, error: &anyhow::Error) -> Self {
        Self {
            project: project.to_string(),
            findings: Vec::new(),
            counts: BTreeMap::new(),
            baseline: None,
            error: Some(format!("{error:#}")),
        }
    }

    pub fn total(&self) -> u64 {
        self.counts.values().sum()
    }

    pub fn changes(&self) -> Vec<Change> {
        let before = self.baseline.as_ref().map_or_default(|b| b.lints.clone());
        baseline::compare(&before, &self.counts)
    }
}

/// Runs one mode over the projects, printing what it found. Returns whether
/// it passed.
pub fn run(root: &Path, options: &Options) -> Result<bool> {
    if options.mode == Mode::Report {
        let measured = measure_all(root, &options.projects)?;
        print!("{}", render_report(&measured));
        return Ok(measured.iter().all(|m| m.error.is_none()));
    }
    let (profile, tracked, projects) = prepare(root, &options.projects)?;
    let changed = match &options.base {
        Some(base) if options.mode == Mode::Check => Some(source::changed_files(root, base)?),
        _ => None,
    };
    let mut passed = true;
    for project in &projects {
        let m = measure(root, project, &profile, &tracked)?;
        if options.mode == Mode::Check {
            let (text, ok) = render_check(&m, changed.as_ref());
            print!("{text}");
            passed &= ok;
        } else {
            passed &= update(root, &m, options)?;
        }
    }
    Ok(passed)
}

/// Measures every project, or every folder with Rust code when none are
/// given. A project that fails to build is kept with its error.
pub fn measure_all(root: &Path, projects: &[String]) -> Result<Vec<Measured>> {
    let (profile, tracked, projects) = prepare(root, projects)?;
    Ok(projects
        .iter()
        .map(|p| measure(root, p, &profile, &tracked).unwrap_or_else(|e| Measured::failed(p, &e)))
        .collect())
}

fn prepare(root: &Path, projects: &[String]) -> Result<(Profile, Vec<String>, Vec<String>)> {
    let profile = Profile::load(root)?;
    let tracked = tracked_files(root)?;
    let projects = if projects.is_empty() {
        rust_projects(&tracked)
    } else {
        projects
            .iter()
            .map(|p| p.trim_end_matches('/').to_string())
            .collect()
    };
    Ok((profile, tracked, projects))
}

/// Measures one project: clippy in each of its cargo roots, plus long files.
pub fn measure(
    root: &Path,
    project: &str,
    profile: &Profile,
    tracked: &[String],
) -> Result<Measured> {
    if !root.join(project).is_dir() {
        bail!("no project folder {project}");
    }
    let scope = Scope {
        root,
        project,
        generated: &profile.generated,
    };
    let in_project = |f: &&String| f.starts_with(&format!("{project}/"));
    let manifests: Vec<String> = tracked
        .iter()
        .filter(in_project)
        .filter(|f| is_manifest(f))
        .cloned()
        .collect();
    let mut found = BTreeSet::new();
    for dir in cargo_roots(root, &manifests, cargo_metadata)? {
        let json = run_clippy(root, &dir, profile)?;
        found.extend(findings::from_clippy(&json, &dir, &scope));
    }
    let lengths = line_counts(
        root,
        tracked
            .iter()
            .filter(in_project)
            .filter(|f| Path::new(f.as_str()).extension().is_some_and(|e| e == "rs")),
    );
    found.extend(findings::long_files(
        &lengths,
        profile.max_file_lines,
        &scope,
    ));
    let counts = findings::counts(&found);
    Ok(Measured {
        project: project.to_string(),
        findings: found.into_iter().collect(),
        counts,
        baseline: read_baseline(root, project)?,
        error: None,
    })
}

fn read_baseline(root: &Path, project: &str) -> Result<Option<Baseline>> {
    let path = root.join(project).join(baseline::FILE);
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path)?;
    baseline::parse(&text)
        .map(Some)
        .with_context(|| format!("reading {}", path.display()))
}

fn update_command(project: &str) -> String {
    format!("`chores quality --update --project {project}`")
}

fn location(f: &Finding) -> String {
    if f.lint == FILE_TOO_LONG {
        format!("{} ({} lines)", f.file, f.line)
    } else {
        format!("{}:{}:{}", f.file, f.line, f.column)
    }
}

/// Prints each risen lint and its findings. Counts can't say which finding
/// is new, so with `changed` (files changed since the merge base) only the
/// findings in those files are listed, or all of them when none are there.
fn render_rises(
    out: &mut String,
    measured: &Measured,
    risen: &[&Change],
    changed: Option<&BTreeSet<String>>,
) {
    for change in risen {
        let _ = writeln!(
            out,
            "  {} {} -> {} (+{}); the findings, one or more of them new:",
            change.lint,
            change.before,
            change.after,
            change.after - change.before
        );
        let of_lint: Vec<&Finding> = measured
            .findings
            .iter()
            .filter(|f| f.lint == change.lint)
            .collect();
        let in_changed: Vec<&Finding> = changed.map_or_default(|files| {
            of_lint
                .iter()
                .copied()
                .filter(|f| files.contains(&f.file))
                .collect()
        });
        let of_lint = if in_changed.is_empty() {
            of_lint
        } else {
            in_changed
        };
        for f in of_lint.iter().take(SHOW_FINDINGS) {
            let _ = writeln!(out, "    {}", location(f));
        }
        if of_lint.len() > SHOW_FINDINGS {
            let _ = writeln!(out, "    ... and {} more", of_lint.len() - SHOW_FINDINGS);
        }
    }
}

/// The `--check` text for one project, and whether it passed.
pub fn render_check(
    measured: &Measured,
    changed_files: Option<&BTreeSet<String>>,
) -> (String, bool) {
    let project = &measured.project;
    let total = measured.total();
    let Some(base) = &measured.baseline else {
        let text = format!(
            "quality: {project}: {total} findings, no {} yet, so nothing to hold; {} writes it\n",
            baseline::FILE,
            update_command(project)
        );
        return (text, true);
    };
    let changes = measured.changes();
    let risen: Vec<&Change> = changes.iter().filter(|c| c.rose()).collect();
    let mut out = String::new();
    if !risen.is_empty() {
        let _ = writeln!(
            out,
            "quality: {project}: FAIL, findings rose ({} -> {total}, QUAL-501). Fix the new ones, \
             or waive one with #[expect(lint, reason = \"...\")] (QUAL-504):",
            base.total()
        );
        render_rises(&mut out, measured, &risen, changed_files);
        return (out, false);
    }
    if changes.is_empty() {
        let _ = writeln!(
            out,
            "quality: {project}: {total} findings, the same as the baseline"
        );
    } else {
        let _ = writeln!(
            out,
            "quality: {project}: {} -> {total} findings; lower the baseline with {} and commit {project}/{}",
            base.total(),
            update_command(project),
            baseline::FILE
        );
    }
    (out, true)
}

fn update(root: &Path, measured: &Measured, options: &Options) -> Result<bool> {
    let project = &measured.project;
    let changes = measured.changes();
    let risen: Vec<&Change> = changes.iter().filter(|c| c.rose()).collect();
    if measured.baseline.is_some() && !risen.is_empty() && !options.accept_rise {
        let mut out = format!(
            "quality: {project}: not written, findings rose (QUAL-501: never raise a baseline). \
             Fix them, or pass --accept-rise for a toolchain or profile change and say why in the PR:\n"
        );
        render_rises(&mut out, measured, &risen, None);
        print!("{out}");
        return Ok(false);
    }
    if measured.baseline.is_some() && changes.is_empty() {
        println!(
            "quality: {project}: {} findings, baseline unchanged",
            measured.total()
        );
        return Ok(true);
    }
    let new = Baseline {
        updated: options.date.clone().unwrap_or_else(crate::new_issue::today),
        lints: measured.counts.clone(),
    };
    let path = root.join(project).join(baseline::FILE);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, baseline::render(&new))
        .with_context(|| format!("writing {}", path.display()))?;
    println!(
        "quality: wrote {project}/{} ({} findings)",
        baseline::FILE,
        new.total()
    );
    Ok(true)
}

fn worst<'a>(keys: impl Iterator<Item = &'a str>) -> Vec<(String, u64)> {
    let mut counts: BTreeMap<&str, u64> = BTreeMap::new();
    for key in keys {
        *counts.entry(key).or_insert(0) += 1;
    }
    let mut sorted: Vec<(String, u64)> = counts
        .into_iter()
        .map(|(k, n)| (k.to_string(), n))
        .collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted.truncate(SHOW_WORST);
    sorted
}

/// The `--report` text: per project, the total against the baseline and the
/// worst lints and files.
pub fn render_report(measured: &[Measured]) -> String {
    let mut out = String::new();
    let all: u64 = measured.iter().map(Measured::total).sum();
    let _ = writeln!(
        out,
        "quality: {all} findings in {} projects",
        measured.len()
    );
    for m in measured {
        if let Some(error) = &m.error {
            let _ = writeln!(out, "\n{}: not measured: {error}", m.project);
            continue;
        }
        let base = m.baseline.as_ref().map_or("no baseline".to_string(), |b| {
            format!("baseline {}", b.total())
        });
        let _ = writeln!(out, "\n{}: {} findings ({base})", m.project, m.total());
        let lints = worst(m.findings.iter().map(|f| f.lint.as_str()));
        let files = worst(m.findings.iter().map(|f| f.file.as_str()));
        for (label, list) in [("lints", lints), ("files", files)] {
            if list.is_empty() {
                continue;
            }
            let joined: Vec<String> = list.iter().map(|(k, n)| format!("{k} {n}")).collect();
            let _ = writeln!(out, "  worst {label}: {}", joined.join(", "));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(lint: &str, path: &str, row: u64) -> Finding {
        Finding {
            lint: lint.into(),
            file: path.into(),
            line: row,
            column: 1,
        }
    }

    fn measured(findings: Vec<Finding>, base: Option<&[(&str, u64)]>) -> Measured {
        Measured {
            project: "cost".into(),
            counts: findings::counts(&findings),
            findings,
            baseline: base.map(|pairs| Baseline {
                updated: "2026-11-27".into(),
                lints: pairs.iter().map(|(l, n)| ((*l).to_string(), *n)).collect(),
            }),
            error: None,
        }
    }

    #[test]
    fn clippy_args_put_allows_last() {
        let profile = Profile {
            warn: vec!["clippy::pedantic".into()],
            allow: vec!["clippy::must_use_candidate".into()],
            max_file_lines: 800,
            generated: vec![],
        };
        assert_eq!(
            profile.clippy_args(),
            ["-W", "clippy::pedantic", "-A", "clippy::must_use_candidate"]
        );
    }

    #[test]
    fn the_committed_profile_loads() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let profile = Profile::load(&root).unwrap();
        assert_eq!(profile.max_file_lines, 800);
        assert!(profile.warn.contains(&"clippy::pedantic".to_string()));
        assert!(
            profile
                .generated
                .contains(&"infra/src/tfschema/".to_string())
        );
    }

    #[test]
    fn check_fails_on_a_rise_and_names_the_findings() {
        let m = measured(
            vec![
                at("clippy::too_many_lines", "cost/src/a.rs", 3),
                at("clippy::too_many_lines", "cost/src/b.rs", 9),
            ],
            Some(&[("clippy::too_many_lines", 1)]),
        );
        let (text, ok) = render_check(&m, None);
        assert!(!ok);
        assert!(text.contains("FAIL, findings rose (1 -> 2"), "{text}");
        assert!(
            text.contains("clippy::too_many_lines 1 -> 2 (+1)"),
            "{text}"
        );
        assert!(
            text.contains("    cost/src/a.rs:3:1\n    cost/src/b.rs:9:1\n"),
            "{text}"
        );
    }

    #[test]
    fn check_passes_when_counts_fall_or_hold() {
        let m = measured(
            vec![at("clippy::a", "cost/src/a.rs", 1)],
            Some(&[("clippy::a", 2)]),
        );
        let (text, ok) = render_check(&m, None);
        assert!(ok);
        assert!(
            text.contains("2 -> 1 findings; lower the baseline"),
            "{text}"
        );
        let m = measured(
            vec![at("clippy::a", "cost/src/a.rs", 1)],
            Some(&[("clippy::a", 1)]),
        );
        let (text, ok) = render_check(&m, None);
        assert!(ok);
        assert!(text.contains("the same as the baseline"), "{text}");
    }

    #[test]
    fn a_fall_in_one_lint_does_not_hide_a_rise_in_another() {
        let m = measured(
            vec![at("clippy::b", "cost/src/a.rs", 1)],
            Some(&[("clippy::a", 5)]),
        );
        let (text, ok) = render_check(&m, None);
        assert!(!ok, "{text}");
        assert!(text.contains("clippy::b 0 -> 1"), "{text}");
    }

    #[test]
    fn check_passes_without_a_baseline() {
        let (text, ok) = render_check(
            &measured(vec![at("clippy::a", "cost/src/a.rs", 1)], None),
            None,
        );
        assert!(ok);
        assert!(text.contains("no docs/quality.toml yet"), "{text}");
    }

    #[test]
    fn long_files_show_their_length() {
        let m = measured(vec![at(FILE_TOO_LONG, "cost/src/a.rs", 900)], Some(&[]));
        let (text, _) = render_check(&m, None);
        assert!(text.contains("cost/src/a.rs (900 lines)"), "{text}");
    }

    #[test]
    fn update_refuses_a_rise_unless_accepted_and_writes_otherwise() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("cost")).unwrap();
        let mut options = Options {
            projects: vec![],
            mode: Mode::Update,
            accept_rise: false,
            date: Some("2026-11-28".into()),
            base: None,
        };
        let rose = measured(vec![at("clippy::a", "cost/src/a.rs", 1)], Some(&[]));
        assert!(!update(dir.path(), &rose, &options).unwrap());
        assert!(!dir.path().join("cost/docs/quality.toml").exists());

        options.accept_rise = true;
        assert!(update(dir.path(), &rose, &options).unwrap());
        let text = fs::read_to_string(dir.path().join("cost/docs/quality.toml")).unwrap();
        assert!(text.contains("updated = 2026-11-28\ntotal = 1\n"), "{text}");

        options.accept_rise = false;
        let first = measured(vec![at("clippy::a", "cost/src/a.rs", 1)], None);
        assert!(
            update(dir.path(), &first, &options).unwrap(),
            "a first baseline is not a rise"
        );
    }

    #[test]
    fn report_lists_worst_lints_and_files() {
        let m = measured(
            vec![
                at("clippy::a", "cost/src/x.rs", 1),
                at("clippy::a", "cost/src/x.rs", 2),
                at("clippy::b", "cost/src/y.rs", 1),
            ],
            None,
        );
        let text = render_report(&[m]);
        assert!(text.contains("quality: 3 findings in 1 projects"), "{text}");
        assert!(text.contains("cost: 3 findings (no baseline)"), "{text}");
        assert!(
            text.contains("worst lints: clippy::a 2, clippy::b 1"),
            "{text}"
        );
        assert!(
            text.contains("worst files: cost/src/x.rs 2, cost/src/y.rs 1"),
            "{text}"
        );
    }

    #[test]
    fn report_keeps_a_project_that_failed_to_build() {
        let failed = Measured::failed("controlled_chaos", &anyhow::anyhow!("cargo clippy failed"));
        let text = render_report(&[failed]);
        assert!(
            text.contains("controlled_chaos: not measured: cargo clippy failed"),
            "{text}"
        );
    }

    #[test]
    fn with_a_base_a_rise_lists_only_findings_in_changed_files() {
        let m = measured(
            vec![
                at("clippy::a", "cost/src/old.rs", 1),
                at("clippy::a", "cost/src/new.rs", 2),
            ],
            Some(&[("clippy::a", 1)]),
        );
        let changed = BTreeSet::from(["cost/src/new.rs".to_string()]);
        let (text, _) = render_check(&m, Some(&changed));
        assert!(text.contains("cost/src/new.rs:2:1"), "{text}");
        assert!(!text.contains("old.rs"), "{text}");
        // No finding in a changed file: list them all.
        let elsewhere = BTreeSet::from(["README.md".to_string()]);
        let (text, _) = render_check(&m, Some(&elsewhere));
        assert!(text.contains("old.rs") && text.contains("new.rs"), "{text}");
    }
}
