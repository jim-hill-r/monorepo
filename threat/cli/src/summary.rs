//! `threat summary --base <ref>`: compares the working tree with the merge
//! base of `<ref>` and prints the PR's security summary
//! (`threat_model::summarize`).

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context as _, Result, bail};
use threat_model::{FilePair, Summary, is_security_file, summarize};

fn git(root: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .context("running git")?;
    if !out.status.success() {
        bail!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn summary(root: &Path, base: &str) -> Result<Summary> {
    let merge_base = git(root, &["merge-base", base, "HEAD"])
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| base.to_string());
    let changed: Vec<String> = git(root, &["diff", "--name-only", &merge_base])
        .context("listing changed files")?
        .lines()
        .map(str::to_string)
        .collect();
    let mut pairs = Vec::new();
    for path in changed.iter().filter(|p| is_security_file(p)) {
        pairs.push(FilePair {
            path: path.clone(),
            before: git(root, &["show", &format!("{merge_base}:{path}")]).ok(),
            after: fs::read_to_string(root.join(path)).ok(),
        });
    }
    Ok(summarize(&changed, &pairs))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, text: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    }

    fn run(root: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(args)
            .output()
            .expect("git")
            .status
            .success();
        assert!(ok, "git {args:?}");
    }

    const MODEL: &str = "reviewed = \"2026-11-27\"\nsummary = \"x\"\n\n[[surface]]\nname = \"site\"\nkind = \"pages\"\naccess = \"cloudflare-access\"\npreviews = \"cloudflare-access\"\ndata = \"private\"\n";

    fn repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        run(root, &["init", "-q", "-b", "main"]);
        write(root, "pane/docs/threat.toml", MODEL);
        write(root, "pane/src/lib.rs", "fn x() {}\n");
        run(root, &["add", "-A"]);
        run(root, &["commit", "-qm", "init"]);
        dir
    }

    #[test]
    fn test_code_change_is_not_security() {
        let dir = repo();
        write(dir.path(), "pane/src/lib.rs", "fn y() {}\n");
        let s = summary(dir.path(), "main").expect("summary");
        assert!(s.labels().is_empty());
    }

    #[test]
    fn test_more_public_surface_needs_human() {
        let dir = repo();
        run(dir.path(), &["checkout", "-qb", "feature"]);
        write(
            dir.path(),
            "pane/docs/threat.toml",
            &MODEL.replacen("access = \"cloudflare-access\"", "access = \"public\"", 1),
        );
        run(dir.path(), &["commit", "-qam", "open it"]);
        let s = summary(dir.path(), "main").expect("summary");
        assert_eq!(s.files, ["pane/docs/threat.toml"]);
        assert_eq!(s.labels(), ["security", "needs-human"]);
    }

    #[test]
    fn test_new_workflow_is_security() {
        let dir = repo();
        write(
            dir.path(),
            ".github/workflows/x.yml",
            "on: push\npermissions:\n  contents: read\n",
        );
        run(dir.path(), &["add", "-A"]);
        let s = summary(dir.path(), "main").expect("summary");
        assert_eq!(s.labels(), ["security"]);
        assert_eq!(s.guards, ["SEC-005 (zizmor)"]);
    }
}
