//! `chores new-issue` and `chores new-human-task`: write issue files in the
//! format of `standards/docs/issue-management.md`, so `issues.sh --check`
//! (ISS-002 to ISS-004) passes the first time.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::issues::quote;

/// An agent issue for `<folder>/issues/<slug>.md`.
#[derive(Debug, Default)]
pub struct AgentIssue {
    /// The project or sub-project folder (`cookbook`, `cahokia/web`), or
    /// `.` for the root `issues/` folder.
    pub folder: String,
    pub slug: String,
    pub kind: String,
    pub priority: String,
    pub epic: Option<String>,
    pub order: Option<u64>,
    /// Add `agent-ignore` so agents leave it alone.
    pub ignore: bool,
    pub text: String,
    pub date: Option<String>,
}

/// A task only a person can do, for `issues/human/<slug>.md`, with its
/// runbook in `standards/docs/runbooks/<runbook>.md`.
#[derive(Debug, Default)]
pub struct HumanTask {
    pub slug: String,
    /// The runbook's title, in plain words ("Make the OpenTofu state
    /// passwords"); also the link text.
    pub title: String,
    pub requirement: String,
    pub why: String,
    pub unblocks: Option<String>,
    /// The runbook's file name without `.md`. Defaults to the slug. An
    /// existing runbook is linked, not rewritten.
    pub runbook: Option<String>,
    pub date: Option<String>,
}

fn check_slug(slug: &str) -> Result<()> {
    let ok = !slug.is_empty()
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--");
    if !ok {
        bail!("'{slug}' is not a kebab-case slug (lowercase letters, digits and single dashes)");
    }
    if slug == "readme" {
        bail!("'readme' can't be an issue slug");
    }
    Ok(())
}

fn check_date(date: &str) -> Result<()> {
    let b = date.as_bytes();
    let ok = b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit());
    if !ok {
        bail!("'{date}' is not a YYYY-MM-DD date");
    }
    Ok(())
}

/// Today's date in UTC as YYYY-MM-DD.
pub fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    civil_from_days(i64::try_from(secs / 86_400).unwrap_or(0))
}

/// Howard Hinnant's days-to-civil algorithm.
fn civil_from_days(z: i64) -> String {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn write_new(path: &Path, text: &str) -> Result<()> {
    if path.exists() {
        bail!(
            "{} already exists; pick another slug or edit that file",
            path.display()
        );
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    fs::write(path, text).with_context(|| format!("writing {}", path.display()))
}

/// Writes the issue file and returns its path from the repo root.
pub fn write_agent_issue(root: &Path, issue: &AgentIssue) -> Result<String> {
    check_slug(&issue.slug)?;
    if !matches!(issue.kind.as_str(), "todo" | "fix") {
        bail!("kind must be todo or fix, not '{}'", issue.kind);
    }
    if !matches!(issue.priority.as_str(), "priority" | "backlog") {
        bail!(
            "priority must be priority or backlog, not '{}'",
            issue.priority
        );
    }
    if issue.text.trim().is_empty() {
        bail!("the issue needs text: what to do and why");
    }
    let date = issue.date.clone().unwrap_or_else(today);
    check_date(&date)?;
    let folder = issue.folder.trim_end_matches('/');
    let folder = folder.strip_suffix("/issues").unwrap_or(folder);
    let base: PathBuf = if folder.is_empty() || folder == "." {
        root.to_path_buf()
    } else {
        root.join(folder)
    };
    if !base.is_dir() {
        bail!("{folder} is not a folder in the repo");
    }
    if folder.starts_with("issues") {
        bail!("human tasks go through new-human-task; use '.' for the root issues/ folder");
    }
    let path = base.join("issues").join(format!("{}.md", issue.slug));
    let tags = if issue.ignore {
        "[agent-generated, agent-ignore]"
    } else {
        "[agent-generated]"
    };
    let mut text = format!(
        "---\nkind: {}\npriority: {}\ntags: {tags}\n",
        issue.kind, issue.priority
    );
    if let Some(epic) = issue.epic.as_deref().filter(|e| !e.trim().is_empty()) {
        text.push_str(&format!("epic: {}\n", quote(epic.trim())));
    }
    if let Some(order) = issue.order {
        text.push_str(&format!("order: {order}\n"));
    }
    text.push_str(&format!("added: {date}\n---\n\n{}\n", issue.text.trim()));
    write_new(&path, &text)?;
    Ok(crate::issues::relative(root, &path))
}

/// Writes the human task, and its runbook from `TEMPLATE.md` when that
/// runbook doesn't exist yet. Returns (task path, runbook path, whether
/// the runbook is new).
pub fn write_human_task(root: &Path, task: &HumanTask) -> Result<(String, String, bool)> {
    check_slug(&task.slug)?;
    let runbook = task.runbook.clone().unwrap_or_else(|| task.slug.clone());
    let runbook = runbook.trim_end_matches(".md").to_string();
    check_slug(&runbook)?;
    for (name, value) in [
        ("title", &task.title),
        ("requirement", &task.requirement),
        ("why", &task.why),
    ] {
        if value.trim().is_empty() {
            bail!("a human task needs --{name}");
        }
    }
    let date = task.date.clone().unwrap_or_else(today);
    check_date(&date)?;

    let runbook_rel = format!("standards/docs/runbooks/{runbook}.md");
    let runbook_path = root.join(&runbook_rel);
    let task_path = root.join("issues/human").join(format!("{}.md", task.slug));
    if task_path.exists() {
        bail!("{} already exists", task_path.display());
    }
    let new_runbook = !runbook_path.exists();
    if new_runbook {
        let template_path = root.join("standards/docs/runbooks/TEMPLATE.md");
        let template = fs::read_to_string(&template_path)
            .with_context(|| format!("reading {}", template_path.display()))?;
        let body = runbook_from_template(&template, &task.title, &task.slug);
        write_new(&runbook_path, &body)?;
    }

    let mut text = format!("---\nadded: {date}\n");
    if let Some(u) = task.unblocks.as_deref().filter(|u| !u.trim().is_empty()) {
        text.push_str(&format!("unblocks: {}\n", quote(u.trim())));
    }
    text.push_str(&format!(
        "runbook: {runbook_rel}\n---\n\n## Requirement\n\n{}\n\n## Why it can't be automated\n\n{}\n\n## Runbook\n\n[{}](../../{runbook_rel})\n",
        task.requirement.trim(),
        task.why.trim(),
        task.title.trim()
    ));
    write_new(&task_path, &text)?;
    Ok((
        crate::issues::relative(root, &task_path),
        runbook_rel,
        new_runbook,
    ))
}

/// The template with its title and status filled in and its copy note
/// dropped; every other section is left for the agent to write.
fn runbook_from_template(template: &str, title: &str, slug: &str) -> String {
    let mut out = String::new();
    let mut in_comment = false;
    for line in template.lines() {
        if in_comment {
            in_comment = !line.contains("-->");
            continue;
        }
        if line.trim_start().starts_with("<!--") {
            in_comment = !line.contains("-->");
            continue;
        }
        if line.starts_with("# ") {
            out.push_str(&format!("# {}\n", title.trim()));
        } else if line.starts_with("- **Status:**") {
            out.push_str(&format!("- **Status:** open (issues/human/{slug}.md)\n"));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    // Collapse the blank lines left where the comment was.
    while out.contains("\n\n\n") {
        out = out.replace("\n\n\n", "\n\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        assert_eq!(civil_from_days(0), "1970-01-01");
        assert_eq!(civil_from_days(20_419), "2025-11-27");
        assert_eq!(civil_from_days(11_016), "2000-02-29");
        assert!(check_date(&today()).is_ok());
        assert!(check_date("2026-1-1").is_err());
    }

    #[test]
    fn slugs() {
        assert!(check_slug("split-web-main-rs").is_ok());
        for bad in ["", "Caps", "a_b", "-a", "a-", "a--b", "a.md", "readme"] {
            assert!(check_slug(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn writes_an_agent_issue_that_parses() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        fs::create_dir_all(root.join("cookbook/web"))?;
        let issue = AgentIssue {
            folder: "cookbook/web".into(),
            slug: "fix-week-header".into(),
            kind: "fix".into(),
            priority: "backlog".into(),
            epic: Some("Design system / Phase 5: link dist".into()),
            order: Some(20),
            text: "The week header overlaps on phones.".into(),
            date: Some("2026-11-27".into()),
            ..AgentIssue::default()
        };
        let rel = write_agent_issue(root, &issue)?;
        assert_eq!(rel, "cookbook/web/issues/fix-week-header.md");
        let text = fs::read_to_string(root.join(&rel))?;
        assert_eq!(
            text,
            "---\nkind: fix\npriority: backlog\ntags: [agent-generated]\nepic: \"Design system / Phase 5: link dist\"\norder: 20\nadded: 2026-11-27\n---\n\nThe week header overlaps on phones.\n"
        );
        let parsed = crate::issues::parse(&rel, &text).expect("parses");
        assert_eq!(parsed.phase(), Some(5));
        assert!(write_agent_issue(root, &issue).is_err(), "no overwrite");
        Ok(())
    }

    #[test]
    fn root_folder_and_bad_input() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        let mut issue = AgentIssue {
            folder: ".".into(),
            slug: "x".into(),
            kind: "todo".into(),
            priority: "priority".into(),
            ignore: true,
            text: "Do x.".into(),
            ..AgentIssue::default()
        };
        assert_eq!(write_agent_issue(root, &issue)?, "issues/x.md");
        let text = fs::read_to_string(root.join("issues/x.md"))?;
        assert!(text.contains("tags: [agent-generated, agent-ignore]"));
        issue.slug = "y".into();
        issue.kind = "chore".into();
        assert!(write_agent_issue(root, &issue).is_err());
        issue.kind = "todo".into();
        issue.folder = "nope".into();
        assert!(write_agent_issue(root, &issue).is_err());
        Ok(())
    }

    #[test]
    fn writes_a_human_task_and_its_runbook() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        fs::create_dir_all(root.join("standards/docs/runbooks"))?;
        fs::write(
            root.join("standards/docs/runbooks/TEMPLATE.md"),
            "# <Task in plain words>\n\n<!-- Copy this file\nand fill it in. -->\n\n- **Kind:** one-time\n- **Status:** open (issues/human/<file>.md) | done\n\n## Steps\n\n1. ...\n",
        )?;
        let task = HumanTask {
            slug: "make-state-passwords".into(),
            title: "Make the OpenTofu state passwords".into(),
            requirement: "Make two passwords.".into(),
            why: "No workflow can make the first secret.".into(),
            unblocks: Some("IaC applies on main (decision 0007)".into()),
            date: Some("2026-11-27".into()),
            ..HumanTask::default()
        };
        let (task_rel, runbook_rel, new) = write_human_task(root, &task)?;
        assert!(new);
        assert_eq!(task_rel, "issues/human/make-state-passwords.md");
        assert_eq!(
            fs::read_to_string(root.join(&task_rel))?,
            "---\nadded: 2026-11-27\nunblocks: IaC applies on main (decision 0007)\nrunbook: standards/docs/runbooks/make-state-passwords.md\n---\n\n## Requirement\n\nMake two passwords.\n\n## Why it can't be automated\n\nNo workflow can make the first secret.\n\n## Runbook\n\n[Make the OpenTofu state passwords](../../standards/docs/runbooks/make-state-passwords.md)\n"
        );
        assert_eq!(
            fs::read_to_string(root.join(&runbook_rel))?,
            "# Make the OpenTofu state passwords\n\n- **Kind:** one-time\n- **Status:** open (issues/human/make-state-passwords.md)\n\n## Steps\n\n1. ...\n"
        );
        // A second task can link the same, now existing, runbook.
        let again = HumanTask {
            slug: "rotate-state-passwords".into(),
            runbook: Some("make-state-passwords".into()),
            ..task
        };
        let (_, _, new) = write_human_task(root, &again)?;
        assert!(!new);
        Ok(())
    }
}
