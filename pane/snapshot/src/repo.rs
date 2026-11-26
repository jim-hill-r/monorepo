//! Walks a checkout of the repo and builds a [`Snapshot`] with the parsers
//! in [`crate::parse`].

use std::fs;
use std::path::{Path, PathBuf};

use pane_model::{Project, Snapshot};

use crate::parse;

/// Directories never walked: build output, dependencies and git's own.
const SKIP_DIRS: &[&str] = &["target", "node_modules", "dist"];

/// Build a snapshot of the checkout at `root`.
pub fn build(root: &Path, commit: &str, taken_at: &str) -> Result<Snapshot, String> {
    let root_mise = read(&root.join("mise.toml"))?;
    let roots = parse::config_roots(&root_mise)?;
    let lists = parse::root_lists(&read_optional(&root.join("ISSUES.md")));

    let mut snapshot = Snapshot::new(commit, taken_at);
    for path in &roots {
        let dir = root.join(path);
        let mise = read_optional(&dir.join("mise.toml"));
        snapshot.projects.push(Project {
            path: path.clone(),
            kind: parse::project_kind(&mise).unwrap_or_else(|| "unknown".to_owned()),
            deploy_url: parse::deploy_url(&mise),
            summary: parse::readme_summary(&read_optional(&dir.join("README.md"))),
            status: lists.status_of(path),
        });
    }

    for file in issues_files(root)? {
        let rel = relative(root, &file)?;
        let project = parse::owning_project(&rel, &roots);
        snapshot
            .work_items
            .extend(parse::issues_items(&rel, project, &read(&file)?));
    }
    Ok(snapshot)
}

/// Every `ISSUES.md` under `root`, sorted, skipping hidden directories and
/// [`SKIP_DIRS`].
pub fn issues_files(root: &Path) -> Result<Vec<PathBuf>, String> {
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
            } else if file_type.is_file() && name == "ISSUES.md" {
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
