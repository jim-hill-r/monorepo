//! `evals human-checks`: which hand-made settings Jim should check now, and
//! recording his answers. See "Human checks" in `standards/docs/evals.md`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use evals_model::{
    By, Kind, ResultStatus, Setting, SettingCheck, due_checks, human_issue_file,
    human_issue_file_name, human_issue_line, parse_evals, parse_settings,
};

use crate::check::{EVALS, SETTING_CHECKS, SETTINGS};

/// Where human tasks live, relative to the repo root.
/// Settings checks are repo-wide (`standards/settings.toml`), so their
/// tasks go in the standards project.
pub const HUMAN_ISSUES: &str = "standards/docs/issues/human";

/// Reads and validates the manifest.
pub fn read_settings(root: &Path) -> Result<Vec<Setting>> {
    let path = root.join(SETTINGS);
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    parse_settings(&text).map_err(|errors| {
        let list: Vec<String> = errors.iter().map(ToString::to_string).collect();
        anyhow!("{} is invalid: {}", path.display(), list.join("; "))
    })
}

/// Reads the answers file; a missing file means no answers yet.
pub fn read_checks(root: &Path) -> Result<Vec<SettingCheck>> {
    let path = root.join(SETTING_CHECKS);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
    };
    text.lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            serde_json::from_str(line)
                .with_context(|| format!("{} line {}", path.display(), index + 1))
        })
        .collect()
}

/// For each setting, the requirements whose `human-check` evals name it.
/// Projects whose `evals.toml` doesn't parse are skipped (`evals check`
/// reports them).
pub fn covered_requirements(root: &Path) -> Result<BTreeMap<String, Vec<String>>> {
    let mut covers: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in fs::read_dir(root).with_context(|| format!("listing {}", root.display()))? {
        let path = entry?.path().join(EVALS);
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(file) = parse_evals(&text) else {
            continue;
        };
        for eval in file.evals {
            if let Kind::HumanCheck { setting } = eval.kind {
                let list = covers.entry(setting).or_default();
                if !list.contains(&eval.requirement) {
                    list.push(eval.requirement);
                }
            }
        }
    }
    for list in covers.values_mut() {
        list.sort();
    }
    Ok(covers)
}

/// One line per check due at `now`. With `write`, also writes a human task
/// file in `standards/docs/issues/human/` for up to `limit` due checks that don't have one
/// yet, and marks those lines `wrote <path>:`; a check whose file already
/// exists is marked `open <path>:`.
pub fn due(root: &Path, now: DateTime<Utc>, write: Option<usize>) -> Result<String> {
    let settings = read_settings(root)?;
    let checks = read_checks(root)?;
    let covers = covered_requirements(root)?;
    let added = now.format("%Y-%m-%d").to_string();
    let mut left = write.unwrap_or(0);
    let mut out = String::new();
    for check in due_checks(&settings, &checks, now) {
        let requirements = covers.get(&check.setting.id).cloned().unwrap_or_default();
        let rel = format!(
            "{HUMAN_ISSUES}/{}",
            human_issue_file_name(&check.setting.id)
        );
        let path = root.join(&rel);
        if path.exists() {
            out.push_str(&format!("open {rel}: "));
        } else if write.is_some() && left > 0 {
            fs::create_dir_all(root.join(HUMAN_ISSUES))?;
            fs::write(&path, human_issue_file(&check, &requirements, &added))
                .with_context(|| format!("writing {}", path.display()))?;
            left -= 1;
            out.push_str(&format!("wrote {rel}: "));
        }
        out.push_str(&human_issue_line(&check, &requirements));
        out.push('\n');
    }
    Ok(out)
}

/// Deletes the setting's human task file, if there is one. Returns whether
/// it existed.
pub fn close_task(root: &Path, setting: &str) -> Result<bool> {
    let path = root.join(HUMAN_ISSUES).join(human_issue_file_name(setting));
    match fs::remove_file(&path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e).with_context(|| format!("deleting {}", path.display())),
    }
}

/// Builds the answer line for `setting`, checking it is in the manifest.
pub fn answer(
    root: &Path,
    setting: &str,
    status: ResultStatus,
    note: Option<String>,
    at: DateTime<Utc>,
) -> Result<SettingCheck> {
    let settings = read_settings(root)?;
    if !settings.iter().any(|s| s.id == setting) {
        bail!("setting `{setting}` is not in {SETTINGS}");
    }
    if status == ResultStatus::Fail && note.as_deref().is_none_or(|n| n.trim().is_empty()) {
        bail!("a not-ok answer needs Jim's words on what is wrong");
    }
    Ok(SettingCheck {
        setting: setting.to_string(),
        status,
        note,
        at,
        by: By::Jim,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"
[[setting]]
id = "auth0-cookbook-app"
vendor = "Auth0"
where = "Applications > Cookbook"
expect = "The production callback URL is allowed."
why = "Sign-in breaks."
every = "90d"

[[setting]]
id = "claude-connectors"
vendor = "Claude"
where = "Settings > Connectors"
expect = "GitHub is connected."
why = "Agents stop."
every = "30d"
"#;

    fn time(text: &str) -> DateTime<Utc> {
        match text.parse() {
            Ok(t) => t,
            Err(e) => panic!("{e}"),
        }
    }

    fn repo() -> Result<tempfile::TempDir> {
        let dir = tempfile::tempdir()?;
        fs::create_dir_all(dir.path().join("standards"))?;
        fs::write(dir.path().join(SETTINGS), MANIFEST)?;
        fs::create_dir_all(dir.path().join("cookbook"))?;
        fs::write(
            dir.path().join("cookbook").join(EVALS),
            "prefix = \"COOK\"\n[[eval]]\nid = \"callbacks\"\nrequirement = \"COOK-004\"\nstages = [\"human\"]\nkind = \"human-check\"\nsetting = \"auth0-cookbook-app\"\n",
        )?;
        Ok(dir)
    }

    #[test]
    fn lists_due_checks_with_the_requirements_they_cover() -> Result<()> {
        let dir = repo()?;
        let root = dir.path();
        let out = due(root, time("2026-12-01T00:00:00Z"), None)?;
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("Check auth0-cookbook-app: "));
        assert!(lines[0].ends_with("(Periodic, every 90d. Never checked. Covers COOK-004.)"));
        assert!(lines[1].ends_with("(Periodic, every 30d. Never checked.)"));

        let ok = answer(
            root,
            "claude-connectors",
            ResultStatus::Pass,
            Some("ok".into()),
            time("2026-12-01T00:00:00Z"),
        )?;
        fs::write(
            root.join(SETTING_CHECKS),
            format!("{}\n", serde_json::to_string(&ok)?),
        )?;
        let out = due(root, time("2026-12-15T00:00:00Z"), None)?;
        assert_eq!(out.lines().count(), 1, "{out}");
        let out = due(root, time("2026-12-31T00:00:00Z"), None)?;
        assert_eq!(out.lines().count(), 2, "{out}");
        assert!(out.contains("Last checked 2026-12-01"));
        Ok(())
    }

    #[test]
    fn writes_task_files_up_to_the_limit_and_closes_them() -> Result<()> {
        let dir = repo()?;
        let root = dir.path();
        let now = time("2026-12-01T00:00:00Z");
        let out = due(root, now, Some(1))?;
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with(
            "wrote standards/docs/issues/human/settings-check-auth0-cookbook-app.md: Check"
        ));
        assert!(lines[1].starts_with("Check claude-connectors:"), "{out}");
        let file = fs::read_to_string(
            root.join("standards/docs/issues/human/settings-check-auth0-cookbook-app.md"),
        )?;
        assert!(file.starts_with("---\nadded: 2026-12-01\n"), "{file}");

        // Next run: the first is open, the second gets written.
        let out = due(root, now, Some(5))?;
        assert!(out.starts_with(
            "open standards/docs/issues/human/settings-check-auth0-cookbook-app.md: "
        ));
        assert!(
            out.contains("wrote standards/docs/issues/human/settings-check-claude-connectors.md: ")
        );

        assert!(close_task(root, "claude-connectors")?);
        assert!(!close_task(root, "claude-connectors")?);
        Ok(())
    }

    #[test]
    fn answers_must_name_a_setting_and_say_what_is_wrong() -> Result<()> {
        let dir = repo()?;
        let root = dir.path();
        let at = time("2026-12-01T00:00:00Z");
        assert!(answer(root, "nope", ResultStatus::Pass, None, at).is_err());
        assert!(answer(root, "claude-connectors", ResultStatus::Fail, None, at).is_err());
        let fail = answer(
            root,
            "claude-connectors",
            ResultStatus::Fail,
            Some("GitHub needs reconnecting".into()),
            at,
        )?;
        assert_eq!(fail.by, By::Jim);
        assert_eq!(fail.note.as_deref(), Some("GitHub needs reconnecting"));
        Ok(())
    }

    #[test]
    fn the_real_manifest_parses() -> Result<()> {
        let root = crate::check::find_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .ok_or_else(|| anyhow!("no repo root"))?;
        assert!(!read_settings(&root)?.is_empty());
        read_checks(&root)?;
        Ok(())
    }
}
