//! Walks a checkout of the repo and builds a [`Snapshot`] with the parsers
//! in [`crate::parse`].

use std::fs;
use std::path::{Path, PathBuf};

use pane_model::{DeliverableKind, Project, Snapshot};

use crate::parse;

/// Directories never walked: build output, dependencies and git's own.
const SKIP_DIRS: &[&str] = &["target", "node_modules", "dist"];

/// Build a snapshot of the checkout at `root`.
pub fn build(root: &Path, commit: &str, taken_at: &str) -> Result<Snapshot, String> {
    let root_mise = read(&root.join("mise.toml"))?;
    let roots = parse::config_roots(&root_mise)?;
    let lists = parse::root_lists(&read_optional(&root.join("issues/README.md")));

    let mut snapshot = Snapshot::new(commit, taken_at);
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

    let rotation = read_optional(&root.join(parse::REVIEWS_DIR).join("README.md"));
    snapshot.reviews = parse::reviews(&rotation);
    snapshot
        .deliverables
        .extend(snapshot.reviews.iter().map(parse::review_deliverable));
    Ok(snapshot)
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

fn read_optional(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}
