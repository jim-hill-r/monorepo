//! Issue files: one Markdown file per issue in an `issues/` folder, with
//! `key: value` front matter (standards/docs/issue-management.md).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

/// One issue file, as read from disk.
#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    /// Path from the repo root, with `/` separators.
    pub path: String,
    pub kind: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub epic: Option<String>,
    pub order: Option<u64>,
    pub added: String,
    /// The text after the front matter, trimmed.
    pub body: String,
}

impl Issue {
    /// True for tasks in the root `issues/human/`.
    pub fn is_human(&self) -> bool {
        self.path.starts_with("issues/human/")
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    /// The top-level project the issue belongs to, or `None` for the root
    /// `issues/` folder (repo-wide work).
    pub fn project(&self) -> Option<&str> {
        if self.path.starts_with("issues/") {
            None
        } else {
            self.path.split('/').next()
        }
    }

    /// The folder holding the issue file, such as `cookbook/web/issues`.
    pub fn folder(&self) -> &str {
        self.path.rsplit_once('/').map_or("", |(d, _)| d)
    }

    /// The epic's name without its phase: `Eval engine` for
    /// `"Eval engine / Phase 2: triage"`.
    pub fn epic_name(&self) -> Option<&str> {
        self.epic
            .as_deref()
            .map(|e| e.split(" / ").next().unwrap_or(e).trim())
    }

    /// The phase number in the epic, such as 2 for `Phase 2: triage`.
    pub fn phase(&self) -> Option<u64> {
        let epic = self.epic.as_deref()?;
        let rest = &epic[epic.find("Phase ")? + "Phase ".len()..];
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        digits.parse().ok()
    }
}

/// Every issue file under `root`: `issues/*.md`, `issues/human/*.md` and
/// `<dir>/issues/*.md`, never `README.md`. Skips `target`, `node_modules`
/// and hidden folders. Sorted by path.
pub fn issue_paths(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk(root, root, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }
            walk(root, &path, out)?;
        } else if file_type.is_file() && name.ends_with(".md") && name != "README.md" {
            let parent = dir.file_name().map(|n| n.to_string_lossy().into_owned());
            let grandparent = dir
                .parent()
                .and_then(Path::file_name)
                .map(|n| n.to_string_lossy().into_owned());
            let in_issues = parent.as_deref() == Some("issues");
            let in_human = parent.as_deref() == Some("human")
                && grandparent.as_deref() == Some("issues")
                && dir.parent().and_then(Path::parent) == Some(root);
            if in_issues || in_human {
                out.push(path);
            }
        }
    }
    Ok(())
}

/// Reads every issue file under `root`. Files without front matter are
/// skipped: the audit (`issues.sh --check`) reports them.
pub fn load(root: &Path) -> Result<Vec<Issue>> {
    let mut issues = Vec::new();
    for path in issue_paths(root)? {
        let text =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        let rel = relative(root, &path);
        if let Some(issue) = parse(&rel, &text) {
            issues.push(issue);
        }
    }
    Ok(issues)
}

/// `path` from `root`, with `/` separators.
pub fn relative(root: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Parses one issue file. `None` when it has no front matter.
pub fn parse(path: &str, text: &str) -> Option<Issue> {
    let text = text.replace('\r', "");
    let mut lines = text.lines();
    if lines.next()? != "---" {
        return None;
    }
    let mut issue = Issue {
        path: path.to_string(),
        kind: String::new(),
        priority: String::new(),
        tags: Vec::new(),
        epic: None,
        order: None,
        added: String::new(),
        body: String::new(),
    };
    let mut closed = false;
    for line in lines.by_ref() {
        if line == "---" {
            closed = true;
            break;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = unquote(value.trim());
        match key.trim() {
            "kind" => issue.kind = value,
            "priority" => issue.priority = value,
            "added" => issue.added = value,
            "epic" => issue.epic = Some(value).filter(|v| !v.is_empty()),
            "order" => issue.order = value.parse().ok(),
            "tags" => {
                issue.tags = value
                    .trim_matches(|c| c == '[' || c == ']')
                    .split(',')
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
            }
            _ => {}
        }
    }
    if !closed {
        return None;
    }
    issue.body = lines.collect::<Vec<_>>().join("\n").trim().to_string();
    Some(issue)
}

fn unquote(value: &str) -> String {
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1].replace("\\\"", "\"")
    } else {
        value.to_string()
    }
}

/// Quotes a front matter value when it holds a `:`, `#` or quote, as the
/// format asks.
pub fn quote(value: &str) -> String {
    if value.contains([':', '#', '"']) {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

/// The `# Priority Projects` and `# On Hold Projects` lists in the root
/// `issues/README.md`: each bullet's first word.
#[derive(Debug, Default, Clone, Serialize)]
pub struct ProjectLists {
    pub priority: Vec<String>,
    pub on_hold: Vec<String>,
}

pub fn project_lists(root: &Path) -> Result<ProjectLists> {
    let path = root.join("issues/README.md");
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Ok(parse_project_lists(&text))
}

pub fn parse_project_lists(text: &str) -> ProjectLists {
    let mut lists = ProjectLists::default();
    let mut current: Option<&mut Vec<String>> = None;
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("# ") {
            current = match heading.trim() {
                "Priority Projects" => Some(&mut lists.priority),
                "On Hold Projects" => Some(&mut lists.on_hold),
                _ => None,
            };
            continue;
        }
        if let (Some(list), Some(item)) = (current.as_deref_mut(), line.strip_prefix("- "))
            && let Some(word) = item.split_whitespace().next()
        {
            list.push(
                word.trim_matches(|c: char| c == '`' || c == '*')
                    .to_string(),
            );
        }
    }
    lists
}

/// The section of a folder's `README.md` under a heading named after the
/// epic (any level), up to the next heading of the same or a higher level.
pub fn epic_notes(readme: &str, epic: &str) -> Option<String> {
    let mut level = 0;
    let mut out = Vec::new();
    for line in readme.lines() {
        let hashes = line.chars().take_while(|&c| c == '#').count();
        let is_heading = hashes > 0 && line[hashes..].starts_with(' ');
        if level > 0 {
            if is_heading && hashes <= level {
                break;
            }
            out.push(line);
        } else if is_heading && line[hashes..].trim().eq_ignore_ascii_case(epic) {
            level = hashes;
        }
    }
    (level > 0).then(|| out.join("\n").trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "---\nkind: todo\npriority: priority\ntags: [agent-generated]\nepic: \"Review dashboard epic / Phase 1: read-only from the repo\"\norder: 30\nadded: 2026-11-26\n---\n\nBuild the Work page.\n";

    #[test]
    fn parses_the_standards_example() {
        let issue = parse("pane/issues/work-page.md", EXAMPLE).expect("front matter");
        assert_eq!(issue.kind, "todo");
        assert_eq!(issue.priority, "priority");
        assert_eq!(issue.tags, vec!["agent-generated"]);
        assert_eq!(issue.epic_name(), Some("Review dashboard epic"));
        assert_eq!(issue.phase(), Some(1));
        assert_eq!(issue.order, Some(30));
        assert_eq!(issue.added, "2026-11-26");
        assert_eq!(issue.body, "Build the Work page.");
        assert_eq!(issue.project(), Some("pane"));
        assert_eq!(issue.folder(), "pane/issues");
    }

    #[test]
    fn root_issues_have_no_project() {
        let issue = parse("issues/x.md", EXAMPLE).expect("front matter");
        assert_eq!(issue.project(), None);
        assert!(!issue.is_human());
        let human = parse("issues/human/x.md", "---\nadded: 2026-11-27\n---\nDo it.")
            .expect("front matter");
        assert!(human.is_human());
    }

    #[test]
    fn no_front_matter_is_none() {
        assert!(parse("issues/x.md", "Just text").is_none());
        assert!(parse("issues/x.md", "---\nkind: todo\n").is_none());
    }

    #[test]
    fn quotes_only_when_needed() {
        assert_eq!(quote("Eval engine"), "Eval engine");
        assert_eq!(quote("Epic / Phase 1: x"), "\"Epic / Phase 1: x\"");
        assert_eq!(quote("a \"b\""), "\"a \\\"b\\\"\"");
    }

    #[test]
    fn reads_project_lists() {
        let lists = parse_project_lists(
            "# Issues\n\n# Priority Projects\n- sparkhill (Blue Eel first)\n- cookbook\n\n# On Hold Projects\n- luggage\n\n# Epic notes\n- not a project\n",
        );
        assert_eq!(lists.priority, vec!["sparkhill", "cookbook"]);
        assert_eq!(lists.on_hold, vec!["luggage"]);
    }

    #[test]
    fn finds_epic_notes() {
        let readme = "# Epic notes\n\n## Eval engine\n\nSee the plan.\n\n## Free tiers\n\nOther.\n";
        assert_eq!(
            epic_notes(readme, "Eval engine").as_deref(),
            Some("See the plan.")
        );
        assert_eq!(epic_notes(readme, "Missing"), None);
    }

    #[test]
    fn walks_only_issue_folders() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let root = dir.path();
        for p in [
            "issues/a.md",
            "issues/README.md",
            "issues/human/h.md",
            "cookbook/issues/c.md",
            "cookbook/web/issues/w.md",
            "cookbook/docs/not.md",
            "cookbook/target/issues/no.md",
            "cookbook/issues/human/no.md",
        ] {
            let path = root.join(p);
            fs::create_dir_all(path.parent().expect("parent"))?;
            fs::write(&path, "x")?;
        }
        let found: Vec<String> = issue_paths(root)?
            .iter()
            .map(|p| relative(root, p))
            .collect();
        assert_eq!(
            found,
            vec![
                "cookbook/issues/c.md",
                "cookbook/web/issues/w.md",
                "issues/a.md",
                "issues/human/h.md"
            ]
        );
        Ok(())
    }
}
