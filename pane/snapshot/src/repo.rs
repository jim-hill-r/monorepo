//! Walks a checkout of the repo and builds a [`Snapshot`] with the parsers
//! in [`crate::parse`].

use std::fs;
use std::path::{Path, PathBuf};

use cost_model::{COST_FILE, CostFile, LimitsFile, Month, Radar};
use pane_model::costs::{
    Costs, ProjectCosts, build_costs, build_history, months_ending, shared_cents,
};
use pane_model::signoff::{self, SignOff};
use pane_model::{DeliverableKind, Project, Snapshot, WorkKind, WorkSource};

use crate::{blame, merged, parse};

/// pane's traffic-light thresholds, from the repo root.
pub const HEALTH_TOML: &str = "pane/health.toml";

/// The vendor blips and their monthly budgets.
pub const RADAR: &str = "standards/radar/radar.json";
/// The free limits and our budgets inside them.
pub const LIMITS: &str = "standards/radar/limits.toml";

/// Directories never walked: build output, dependencies and git's own.
const SKIP_DIRS: &[&str] = &["target", "node_modules", "dist"];

/// Build a snapshot of the checkout at `root`.
pub fn build(root: &Path, commit: &str, taken_at: &str) -> Result<Snapshot, String> {
    let root_mise = read(&root.join("mise.toml"))?;
    let roots = parse::config_roots(&root_mise)?;
    let lists = parse::root_lists(&read_optional(&root.join("issues/README.md")));

    let mut snapshot = Snapshot::new(commit, taken_at);
    let health = root.join(HEALTH_TOML);
    if health.exists() {
        snapshot.thresholds = parse::health_thresholds(&read(&health)?)?;
    }
    for path in &roots {
        let dir = root.join(path);
        let mise = read_optional(&dir.join("mise.toml"));
        let deploy_url = parse::deploy_url(&mise);
        if let Some(url) = &deploy_url {
            snapshot.deliverables.push(parse::live_site(path, url));
        }
        if let Some(tag) = parse::release_tag(&mise) {
            // Asset names start with the Cargo package name; fall back to
            // the folder name for a project without one.
            let package = parse::package_name(&read_optional(&dir.join("Cargo.toml")))
                .unwrap_or_else(|| path.rsplit('/').next().unwrap_or(path).to_owned());
            snapshot
                .deliverables
                .push(parse::release(path, &tag, &package));
        }
        snapshot.projects.push(Project {
            path: path.clone(),
            kind: parse::project_kind(&mise).unwrap_or_else(|| "unknown".to_owned()),
            deploy_url,
            summary: parse::readme_summary(&read_optional(&dir.join("README.md"))),
            status: lists.status_of(path),
        });
    }

    for file in all_files(root)? {
        let rel = relative(root, &file)?;
        let project = parse::owning_project(&rel, &roots);
        let name = rel.rsplit('/').next().unwrap_or(&rel);
        let ext = name.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
        let issue_kind = parse::issue_file_kind(&rel);
        if issue_kind == Some(parse::IssueFile::Work) {
            snapshot
                .work_items
                .extend(parse::issue_file(&rel, project, &read(&file)?));
        } else if issue_kind == Some(parse::IssueFile::Human) {
            snapshot
                .human_issues
                .extend(parse::human_issue_file(&rel, &read(&file)?));
        } else if let Some(kind) = parse::doc_kind(&rel) {
            let contents = read(&file)?;
            if kind == DeliverableKind::Retro {
                snapshot.retros.push(parse::retro(&rel, project, &contents));
            }
            snapshot
                .deliverables
                .push(parse::doc_deliverable(kind, &rel, project, &contents));
        } else if parse::CODE_EXTENSIONS.contains(&ext) {
            // Skip files that aren't UTF-8 rather than failing the snapshot.
            if let Ok(contents) = fs::read_to_string(&file) {
                snapshot
                    .work_items
                    .extend(parse::code_comments(&rel, project, &contents));
            }
        }
    }

    // Age each FIX comment by the commit that added its line. TODOs don't
    // need an age, and there are few FIXes, so this stays cheap. A shallow
    // clone can't date old lines, so they stay unknown there.
    let blame = !blame::is_shallow(root);
    for item in &mut snapshot.work_items {
        if blame && item.kind == WorkKind::Fix && item.source == WorkSource::CodeComment {
            item.added = blame::line_date(root, &item.file, item.line);
        }
    }

    let rotation = read_optional(&root.join(parse::REVIEWS_DIR).join("README.md"));
    snapshot.reviews = parse::reviews(&rotation);
    for review in &mut snapshot.reviews {
        // A deck that isn't there yet (its PR not merged) has no scorecard.
        review.scorecard = parse::scorecard(&read_optional(&root.join(&review.path)));
        review.signoffs = signoffs(root, review.folder())?;
    }
    snapshot
        .deliverables
        .extend(snapshot.reviews.iter().map(parse::review_deliverable));
    snapshot.costs = costs(root, &roots, taken_at)?;
    Ok(snapshot)
}

/// The sign-offs in `docs/reviews/<folder>/signoffs/`, oldest first, each
/// feedback item open while its issue file is still there. A sign-off that
/// doesn't parse fails the snapshot, so a bad one is caught in CI rather
/// than shown as unreviewed.
pub fn signoffs(root: &Path, folder: &str) -> Result<Vec<SignOff>, String> {
    let dir = root
        .join(parse::REVIEWS_DIR)
        .join(folder)
        .join(signoff::SIGNOFFS_DIR);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let file = entry.map_err(|e| format!("{}: {e}", dir.display()))?.path();
        let rel = relative(root, &file)?;
        if signoff::signoff_folder(&rel).is_none() {
            continue;
        }
        let mut s = signoff::parse(&rel, &read(&file)?)?;
        for item in &mut s.feedback {
            item.open = item.issue.as_ref().is_some_and(|i| root.join(i).is_file());
        }
        found.push(s);
    }
    found.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(found)
}

/// Every project's `docs/cost.toml` with the radar's budgets and the free
/// limits, for the month of `taken_at`. A project without a cost file is
/// left out; one that doesn't parse, or a radar or limits file that doesn't,
/// fails the snapshot, so a bad file never reaches pane as missing money. A
/// checkout without `standards/radar/` (a test fixture) gets no budgets or
/// limits.
pub fn costs(root: &Path, roots: &[String], taken_at: &str) -> Result<Costs, String> {
    let month: Month = taken_at
        .get(..7)
        .ok_or_else(|| format!("taken at `{taken_at}` doesn't start with YYYY-MM"))?
        .parse()
        .map_err(|e| format!("taken at `{taken_at}`: {e}"))?;
    let radar = match read_if_exists(&root.join(RADAR))? {
        Some(text) => cost_model::parse_radar(&text).map_err(|e| format!("{RADAR}: {e}"))?,
        None => Radar::default(),
    };
    let limits = match read_if_exists(&root.join(LIMITS))? {
        Some(text) => cost_model::parse_limits(&text).map_err(|e| format!("{LIMITS}: {e}"))?,
        None => LimitsFile::default(),
    };
    let mut files: Vec<(&str, CostFile, String)> = Vec::new();
    for project in roots {
        let rel = format!("{project}/{COST_FILE}");
        if let Some(text) = read_if_exists(&root.join(&rel))? {
            let file = cost_model::parse_cost_file(&text).map_err(|e| format!("{rel}: {e}"))?;
            files.push((project, file, text));
        }
    }
    let parsed: Vec<(&str, &CostFile)> = files.iter().map(|(p, f, _)| (*p, f)).collect();
    let mut costs = build_costs(month, &parsed, &radar, &limits);
    for (project, (_, _, text)) in costs.projects.iter_mut().zip(&files) {
        set_lines(project, text);
    }

    // The Money page's twelve months, for the top-level projects.
    let months = months_ending(month, HISTORY_MONTHS);
    let top: Vec<&str> = roots
        .iter()
        .map(String::as_str)
        .filter(|r| !r.contains('/'))
        .collect();
    let merged = months
        .first()
        .and_then(|&since| merged::by_month(root, since, &top.iter().copied().collect()));
    let rows: Vec<(&str, Option<&CostFile>)> = top
        .iter()
        .map(|&p| (p, parsed.iter().find(|(q, _)| *q == p).map(|(_, f)| *f)))
        .collect();
    costs.history_from_git = merged.is_some();
    costs.history = build_history(
        &months,
        &rows,
        shared_cents(&radar),
        &merged.unwrap_or_default(),
    );
    Ok(costs)
}

/// How many months the Money page charts.
pub const HISTORY_MONTHS: u32 = 12;

/// Give each service, cost and revenue line of `project` the line of its
/// header in `text`, its `docs/cost.toml`, so pane links to it.
fn set_lines(project: &mut ProjectCosts, text: &str) {
    let services = parse::table_lines(text, "service");
    for (s, line) in project.services.iter_mut().zip(services) {
        s.line = line;
    }
    let costs = parse::table_lines(text, "cost");
    for (c, line) in project.costs.iter_mut().zip(costs) {
        c.line = line;
    }
    let revenue = parse::table_lines(text, "revenue");
    for (r, line) in project.revenue.iter_mut().zip(revenue) {
        r.line = line;
    }
}

/// Every file under `root`, sorted, skipping hidden directories and
/// [`SKIP_DIRS`].
pub fn all_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let file_type = entry
                .file_type()
                .map_err(|e| format!("{}: {e}", path.display()))?;
            if file_type.is_dir() {
                if !name.starts_with('.') && !SKIP_DIRS.contains(&name.as_ref()) {
                    stack.push(path);
                }
            } else if file_type.is_file() {
                found.push(path);
            }
        }
    }
    found.sort();
    Ok(found)
}

fn relative(root: &Path, path: &Path) -> Result<String, String> {
    let rel = path
        .strip_prefix(root)
        .map_err(|_| format!("{} is outside {}", path.display(), root.display()))?;
    Ok(rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The file's contents, `None` when it doesn't exist, an error otherwise.
fn read_if_exists(path: &Path) -> Result<Option<String>, String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

fn read_optional(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

/// The audit findings in `file` (the saved output of `mise run //:audit`),
/// each given its project from the snapshot's config roots. `None`, with a
/// warning, when the audit didn't finish (no closing `audit: ...` line), so
/// pane shows Standards as not run rather than clean. A file that can't be
/// read fails the snapshot.
pub fn audit(file: &Path, snapshot: &Snapshot) -> Result<Option<pane_model::audit::Audit>, String> {
    let output = read(file)?;
    let roots: Vec<String> = snapshot.projects.iter().map(|p| p.path.clone()).collect();
    let audit = pane_model::audit::parse(&output, &roots);
    if audit.is_none() {
        eprintln!(
            "pane_snapshot: {} has no closing `audit:` line; Standards stays grey",
            file.display()
        );
    }
    Ok(audit)
}
