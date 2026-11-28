//! Finding a project's code: git's list of tracked files, the folders to
//! run cargo in, clippy's JSON output and file lengths.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use super::{PROFILE_DIR, Profile};

/// A `Cargo.toml` that belongs to the project, not a test fixture or a build
/// folder.
pub fn is_manifest(path: &str) -> bool {
    let skipped = ["fixtures", "target", "node_modules"];
    path.ends_with("/Cargo.toml") && !path.split('/').any(|part| skipped.contains(&part))
}

/// Top-level folders with a tracked `Cargo.toml`, for "every project".
pub fn rust_projects(tracked: &[String]) -> Vec<String> {
    let projects: BTreeSet<String> = tracked
        .iter()
        .filter(|f| is_manifest(f))
        .filter_map(|f| f.split_once('/').map(|(top, _)| top.to_string()))
        .collect();
    projects.into_iter().collect()
}

/// The folders to run clippy in: each manifest's workspace root, once. A
/// manifest another root's workspace already covers is skipped, so a
/// workspace and its members run once, while a crate the workspace excludes
/// (pane/cloudflare) runs on its own. `metadata` gives a folder's workspace
/// root and member manifests.
pub fn cargo_roots(
    root: &Path,
    manifests: &[String],
    mut metadata: impl FnMut(&Path) -> Result<(PathBuf, Vec<PathBuf>)>,
) -> Result<Vec<PathBuf>> {
    let mut sorted: Vec<&String> = manifests.iter().collect();
    sorted.sort_by_key(|m| (m.matches('/').count(), m.as_str()));
    let mut covered = BTreeSet::new();
    let mut roots = Vec::new();
    for manifest in sorted {
        let path = root.join(manifest);
        if covered.contains(&path) {
            continue;
        }
        let dir = path.parent().unwrap_or(root);
        let (workspace, members) = metadata(dir)?;
        covered.insert(path);
        covered.extend(members);
        if !roots.contains(&workspace) {
            roots.push(workspace);
        }
    }
    Ok(roots)
}

#[derive(Deserialize)]
struct Metadata {
    workspace_root: PathBuf,
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    manifest_path: PathBuf,
}

pub fn cargo_metadata(dir: &Path) -> Result<(PathBuf, Vec<PathBuf>)> {
    let out = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(dir)
        .output()
        .context("running cargo metadata")?;
    if !out.status.success() {
        bail!(
            "cargo metadata failed in {}: {}",
            dir.display(),
            tail(&out.stderr)
        );
    }
    let meta: Metadata = serde_json::from_slice(&out.stdout).context("parsing cargo metadata")?;
    let members = meta.packages.into_iter().map(|p| p.manifest_path).collect();
    Ok((meta.workspace_root, members))
}

pub fn run_clippy(root: &Path, dir: &Path, profile: &Profile) -> Result<String> {
    let mut cmd = Command::new("cargo");
    cmd.args(["clippy", "--all-targets", "--message-format=json", "--"])
        .args(profile.clippy_args())
        .current_dir(dir);
    if !has_own_clippy_toml(root, dir) {
        cmd.env("CLIPPY_CONF_DIR", root.join(PROFILE_DIR));
    }
    let out = cmd.output().context("running cargo clippy")?;
    if !out.status.success() {
        bail!(
            "cargo clippy failed in {}: {}",
            dir.display(),
            tail(&out.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Whether clippy would find a project's own `clippy.toml` (in the folder or
/// above it, inside the repo), which then wins over the shared one. Such a
/// project copies the shared thresholds (QUAL-402).
fn has_own_clippy_toml(root: &Path, dir: &Path) -> bool {
    dir.ancestors()
        .take_while(|d| d.starts_with(root) && *d != root)
        .any(|d| d.join("clippy.toml").is_file() || d.join(".clippy.toml").is_file())
}

fn tail(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(20)..].join("\n")
}

pub fn tracked_files(root: &Path) -> Result<Vec<String>> {
    let out = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()
        .context("running git ls-files")?;
    if !out.status.success() {
        bail!("git ls-files failed: {}", tail(&out.stderr));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|f| !f.is_empty())
        .map(str::to_string)
        .collect())
}

pub fn line_counts<'a>(
    root: &Path,
    files: impl Iterator<Item = &'a String>,
) -> BTreeMap<String, usize> {
    let mut out = BTreeMap::new();
    for file in files {
        // A tracked file deleted in the working tree has no length to check.
        let Ok(text) = fs::read_to_string(root.join(file)) else {
            continue;
        };
        out.insert(file.clone(), text.lines().count());
    }
    out
}

/// Files changed since the merge base with `base`, committed or not, plus
/// untracked ones, from the repo root.
pub fn changed_files(root: &Path, base: &str) -> Result<BTreeSet<String>> {
    let merge_base = git(root, &["merge-base", base, "HEAD"])?;
    let Some(merge_base) = merge_base.lines().next() else {
        bail!("no merge base with {base}");
    };
    let diff = git(root, &["diff", "--name-only", merge_base.trim()])?;
    let untracked = git(root, &["ls-files", "--others", "--exclude-standard"])?;
    Ok(diff
        .lines()
        .chain(untracked.lines())
        .map(str::to_string)
        .collect())
}

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))?;
    if !out.status.success() {
        bail!("git {} failed: {}", args.join(" "), tail(&out.stderr));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_roots_run_a_workspace_once_and_excluded_crates_alone() {
        let root = Path::new("/repo");
        let manifests = [
            "pane/model/Cargo.toml",
            "pane/Cargo.toml",
            "pane/web/Cargo.toml",
            "pane/cloudflare/Cargo.toml",
        ]
        .map(String::from);
        let mut asked = Vec::new();
        let roots = cargo_roots(root, &manifests, |dir| {
            asked.push(dir.to_path_buf());
            if dir.ends_with("cloudflare") {
                Ok((dir.to_path_buf(), vec![dir.join("Cargo.toml")]))
            } else {
                let ws = PathBuf::from("/repo/pane");
                Ok((
                    ws.clone(),
                    vec![ws.join("model/Cargo.toml"), ws.join("web/Cargo.toml")],
                ))
            }
        })
        .unwrap();
        assert_eq!(
            roots,
            [
                PathBuf::from("/repo/pane"),
                PathBuf::from("/repo/pane/cloudflare")
            ]
        );
        assert_eq!(
            asked,
            [
                PathBuf::from("/repo/pane"),
                PathBuf::from("/repo/pane/cloudflare")
            ]
        );
    }

    #[test]
    fn rust_projects_skip_fixtures() {
        let tracked = [
            "cookbook/Cargo.toml",
            "cookbook/web/Cargo.toml",
            "royalty/cli/tests/fixtures/acme_shop/Cargo.toml",
            "sparkhill/webapp/Cargo.toml",
            "README.md",
        ]
        .map(String::from);
        assert_eq!(rust_projects(&tracked), ["cookbook", "sparkhill"]);
    }
}
