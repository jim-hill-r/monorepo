//! `threat check`: reads the files and runs the rules in `threat_model`.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};
use threat_model::{
    Context, Date, Finding, THREAT_FILE, Workflow, check_project, parse_setting_ids,
};

pub const SETTINGS: &str = "standards/settings.toml";
pub const WORKFLOWS: &str = ".github/workflows";

pub struct Report {
    pub findings: Vec<Finding>,
    /// How many projects the rules ran on.
    pub checked: usize,
}

/// Every top-level directory with a `mise.toml`, sorted.
pub fn projects(root: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("reading {}", root.display()))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || !entry.path().join("mise.toml").is_file() {
            continue;
        }
        names.push(name);
    }
    names.sort();
    Ok(names)
}

/// The top-level projects `selected` names, or every project.
fn selection(all: &[String], selected: Option<&[String]>) -> BTreeSet<String> {
    let Some(selected) = selected else {
        return all.iter().cloned().collect();
    };
    selected
        .iter()
        .filter_map(|s| {
            let top = s.trim().trim_start_matches("./").split('/').next()?;
            all.iter().find(|p| p.as_str() == top).cloned()
        })
        .collect()
}

/// The `.yml` and `.yaml` files in `.github/workflows/`, sorted.
fn workflows(root: &Path) -> Result<Vec<Workflow>> {
    let dir = root.join(WORKFLOWS);
    let Ok(entries) = fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !(name.ends_with(".yml") || name.ends_with(".yaml")) {
            continue;
        }
        let text = fs::read_to_string(entry.path())
            .with_context(|| format!("reading {WORKFLOWS}/{name}"))?;
        found.push(Workflow {
            path: format!("{WORKFLOWS}/{name}"),
            text,
        });
    }
    found.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(found)
}

pub fn check(root: &Path, selected: Option<&[String]>, today: Date) -> Result<Report> {
    let settings =
        fs::read_to_string(root.join(SETTINGS)).with_context(|| format!("reading {SETTINGS}"))?;
    let settings = parse_setting_ids(&settings).map_err(|e| anyhow::anyhow!("{SETTINGS}: {e}"))?;
    let workflows = workflows(root)?;
    let read = |rel: &str| fs::read_to_string(root.join(rel)).ok();
    let cx = Context {
        settings: &settings,
        read: &read,
        workflows: &workflows,
        today,
    };
    let chosen = selection(&projects(root)?, selected);
    let mut findings = Vec::new();
    for name in &chosen {
        let text = fs::read_to_string(root.join(name).join(THREAT_FILE)).ok();
        findings.extend(check_project(name, text.as_deref(), &cx));
    }
    Ok(Report {
        findings,
        checked: chosen.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, text: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(path, text).expect("write");
    }

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();
        write(root, SETTINGS, "[[setting]]\nid = \"token\"\n");
        for p in ["cookbook", "pane", "homer"] {
            write(root, &format!("{p}/mise.toml"), "");
        }
        write(root, ".github/mise.toml", "");
        write(
            root,
            "cookbook/docs/threat.toml",
            "reviewed = \"2026-11-26\"\nsummary = \"x\"\n",
        );
        write(
            root,
            "pane/docs/threat.toml",
            "reviewed = \"2026-11-26\"\nsummary = \"x\"\n[[secret]]\nname = \"TOKEN\"\nwhere = \"github\"\nscope = \"read\"\nsetting = \"token\"\n",
        );
        write(
            root,
            ".github/workflows/pane-deploy.yml",
            "env:\n  A: ${{ secrets.TOKEN }}\n  B: ${{ secrets.EXTRA }}\n",
        );
        write(root, ".github/workflows/notes.txt", "secrets.NOPE");
        dir
    }

    fn today() -> Date {
        "2026-11-27".parse().expect("date")
    }

    fn codes(report: &Report) -> Vec<(&'static str, String)> {
        report
            .findings
            .iter()
            .map(|f| (f.code.as_str(), f.path.clone()))
            .collect()
    }

    #[test]
    fn test_projects_are_top_level_dirs_with_mise_toml() {
        let dir = fixture();
        assert_eq!(
            projects(dir.path()).expect("projects"),
            ["cookbook", "homer", "pane"]
        );
    }

    #[test]
    fn test_check_reads_files_and_workflows() {
        let dir = fixture();
        let report = check(dir.path(), None, today()).expect("check");
        assert_eq!(report.checked, 3);
        assert_eq!(
            codes(&report),
            [
                ("THR-001", "homer/docs/threat.toml".to_string()),
                ("THR-005", ".github/workflows/pane-deploy.yml:3".to_string()),
            ]
        );
    }

    #[test]
    fn test_selected_projects_only() {
        let dir = fixture();
        let selected = ["cookbook/web".to_string(), "nope".to_string()];
        let report = check(dir.path(), Some(&selected), today()).expect("check");
        assert_eq!(report.checked, 1);
        assert!(report.findings.is_empty());
        let report = check(dir.path(), Some(&[]), today()).expect("check");
        assert_eq!(report.checked, 0);
    }

    #[test]
    fn test_missing_settings_is_an_error() {
        let dir = fixture();
        fs::remove_file(dir.path().join(SETTINGS)).expect("rm");
        assert!(check(dir.path(), None, today()).is_err());
    }
}
