//! Pure parsers over file contents. Nothing here touches the file system, so
//! each one is tested against strings and fixture files.

use pane_model::{ProjectStatus, WorkItem, WorkKind, WorkSource};

/// The project paths in the root `mise.toml`'s `[monorepo].config_roots`.
pub fn config_roots(root_mise: &str) -> Result<Vec<String>, String> {
    let value: toml::Table = toml::from_str(root_mise).map_err(|e| e.to_string())?;
    let roots = value
        .get("monorepo")
        .and_then(|m| m.get("config_roots"))
        .and_then(|r| r.as_array())
        .ok_or("root mise.toml has no [monorepo].config_roots")?;
    roots
        .iter()
        .map(|r| {
            r.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "config_roots holds a value that is not a string".to_owned())
        })
        .collect()
}

/// The project type: `<type>` from its `standards/tasks/<type>.toml` include.
pub fn project_kind(project_mise: &str) -> Option<String> {
    const MARKER: &str = "standards/tasks/";
    let start = project_mise.find(MARKER)? + MARKER.len();
    let rest = &project_mise[start..];
    let end = rest.find(".toml")?;
    let kind = &rest[..end];
    let valid = !kind.is_empty()
        && kind
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    valid.then(|| kind.to_owned())
}

/// `DEPLOY_URL` from a project's `[env]` table.
pub fn deploy_url(project_mise: &str) -> Option<String> {
    let value: toml::Table = toml::from_str(project_mise).ok()?;
    value
        .get("env")?
        .get("DEPLOY_URL")?
        .as_str()
        .map(str::to_owned)
}

/// The first paragraph of a README that is not a heading, badge, quote,
/// list, table or code block, joined onto one line.
pub fn readme_summary(readme: &str) -> Option<String> {
    let mut paragraph: Vec<&str> = Vec::new();
    let mut in_code = false;
    for line in readme.lines() {
        let t = line.trim();
        if t.starts_with("```") {
            in_code = !in_code;
            if !paragraph.is_empty() {
                break;
            }
            continue;
        }
        if in_code {
            continue;
        }
        let skip = t.starts_with('#')
            || t.starts_with('>')
            || t.starts_with("[![")
            || t.starts_with("![")
            || t.starts_with('<')
            || t.starts_with("- ")
            || t.starts_with("* ")
            || t.starts_with('|');
        if t.is_empty() || skip {
            if !paragraph.is_empty() {
                break;
            }
            continue;
        }
        paragraph.push(t);
    }
    (!paragraph.is_empty()).then(|| paragraph.join(" "))
}

/// Projects listed under `# Priority Projects` and `# On Hold Projects` in
/// the root `ISSUES.md`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct RootLists {
    pub priority: Vec<String>,
    pub on_hold: Vec<String>,
}

impl RootLists {
    /// A project's status. A sub-project (`pane/web`) takes the status of
    /// the top-level project it lives in unless it is listed itself. On hold
    /// wins over priority.
    pub fn status_of(&self, path: &str) -> ProjectStatus {
        let top = path.split('/').next().unwrap_or(path);
        let listed = |list: &[String]| list.iter().any(|p| p == path || p == top);
        if listed(&self.on_hold) {
            ProjectStatus::OnHold
        } else if listed(&self.priority) {
            ProjectStatus::Priority
        } else {
            ProjectStatus::Active
        }
    }
}

pub fn root_lists(root_issues: &str) -> RootLists {
    let mut lists = RootLists::default();
    let mut current: Option<&mut Vec<String>> = None;
    for line in root_issues.lines() {
        if let Some(h) = line.strip_prefix("# ") {
            current = match h.trim() {
                "Priority Projects" => Some(&mut lists.priority),
                "On Hold Projects" => Some(&mut lists.on_hold),
                _ => None,
            };
            continue;
        }
        if let (Some(list), Some(item)) = (current.as_deref_mut(), line.trim().strip_prefix("- ")) {
            let name = item
                .trim()
                .trim_matches(|c| c == '[' || c == ']' || c == '`');
            if !name.is_empty() {
                list.push(name.to_owned());
            }
        }
    }
    lists
}

/// A `TODO` or `FIX` marker at the start of `text`, with its tags and the
/// text after them: `TODO (agent-generated, agent-ignore): Do it` gives
/// `(Todo, ["agent-generated", "agent-ignore"], "Do it")`.
pub fn parse_marker(text: &str) -> Option<(WorkKind, Vec<String>, String)> {
    let (kind, rest) = if let Some(r) = text.strip_prefix("TODO") {
        (WorkKind::Todo, r)
    } else {
        let r = text.strip_prefix("FIX")?;
        (WorkKind::Fix, r)
    };
    // The marker must end there: `TODOS` or `FIXME` are other words.
    if rest.starts_with(|c: char| c.is_alphanumeric() || c == '_') {
        return None;
    }
    let mut rest = rest.trim_start();
    let mut tags = Vec::new();
    if let Some(inner) = rest.strip_prefix('(') {
        let close = inner.find(')')?;
        tags = inner[..close]
            .split(',')
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty())
            .collect();
        rest = inner[close + 1..].trim_start();
    }
    let rest = rest.strip_prefix(':').unwrap_or(rest).trim();
    Some((kind, tags, rest.to_owned()))
}

/// Every `TODO`/`FIX` list item in an `ISSUES.md`, with its section (the
/// last `#` heading) and epic (the last `##` or `###` heading inside that
/// section, joined as `Epic / Phase` when both are present).
pub fn issues_items(file: &str, project: Option<&str>, contents: &str) -> Vec<WorkItem> {
    let mut items = Vec::new();
    let mut section: Option<String> = None;
    let mut h2: Option<String> = None;
    let mut h3: Option<String> = None;
    let mut in_code = false;
    for (i, line) in contents.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
            continue;
        }
        if in_code {
            continue;
        }
        if let Some(h) = line.strip_prefix("### ") {
            h3 = Some(h.trim().to_owned());
            continue;
        }
        if let Some(h) = line.strip_prefix("## ") {
            h2 = Some(h.trim().to_owned());
            h3 = None;
            continue;
        }
        if let Some(h) = line.strip_prefix("# ") {
            section = Some(h.trim().to_owned());
            h2 = None;
            h3 = None;
            continue;
        }
        let Some(item) = line.trim_start().strip_prefix("- ") else {
            continue;
        };
        let Some((kind, tags, text)) = parse_marker(item.trim_start()) else {
            continue;
        };
        let epic = match (&h2, &h3) {
            (Some(a), Some(b)) => Some(format!("{a} / {b}")),
            (a, b) => a.clone().or_else(|| b.clone()),
        };
        items.push(WorkItem {
            kind,
            text,
            project: project.map(str::to_owned),
            file: file.to_owned(),
            line: u32::try_from(i + 1).unwrap_or(u32::MAX),
            source: WorkSource::IssuesFile,
            section: section.clone(),
            epic,
            tags,
        });
    }
    items
}

/// The project a repo path belongs to: the longest config root that
/// contains it, or `None` for files outside every project.
pub fn owning_project<'a>(path: &str, roots: &'a [String]) -> Option<&'a str> {
    roots
        .iter()
        .filter(|r| path == r.as_str() || path.starts_with(&format!("{r}/")))
        .max_by_key(|r| r.len())
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT_MISE: &str = include_str!("../tests/fixtures/repo/mise.toml.fixture");
    const ROOT_ISSUES: &str = include_str!("../tests/fixtures/repo/ISSUES.md.fixture");
    const APP_MISE: &str = include_str!("../tests/fixtures/repo/app/mise.toml.fixture");
    const APP_WEB_MISE: &str = include_str!("../tests/fixtures/repo/app/web/mise.toml.fixture");
    const APP_README: &str = include_str!("../tests/fixtures/repo/app/README.md.fixture");
    const APP_ISSUES: &str = include_str!("../tests/fixtures/repo/app/ISSUES.md.fixture");

    #[test]
    fn reads_config_roots() {
        assert_eq!(
            config_roots(ROOT_MISE),
            Ok(vec![
                "app".to_owned(),
                "app/web".to_owned(),
                "tool".to_owned()
            ])
        );
    }

    #[test]
    fn config_roots_errors_without_the_table() {
        assert!(config_roots("[tools]\nnode = \"24\"\n").is_err());
        assert!(config_roots("not = [toml").is_err());
        assert!(config_roots("[monorepo]\nconfig_roots = [1]\n").is_err());
    }

    #[test]
    fn reads_project_kind() {
        assert_eq!(project_kind(APP_MISE), Some("rust".to_owned()));
        assert_eq!(project_kind(APP_WEB_MISE), Some("dioxus-web".to_owned()));
        assert_eq!(project_kind("[tools]\n"), None);
        assert_eq!(project_kind("includes = [\"standards/tasks/.toml\"]"), None);
    }

    #[test]
    fn reads_deploy_url() {
        assert_eq!(
            deploy_url(APP_WEB_MISE),
            Some("https://app.pages.dev".to_owned())
        );
        assert_eq!(deploy_url(APP_MISE), None);
        assert_eq!(deploy_url("not toml ["), None);
    }

    #[test]
    fn readme_summary_skips_headings_badges_and_code() {
        assert_eq!(
            readme_summary(APP_README),
            Some("App does one thing well. It is named for what it is.".to_owned())
        );
        assert_eq!(readme_summary("# Title\n\n## Only headings\n"), None);
        assert_eq!(
            readme_summary("# T\n\n- a list\n- item\n\n| a | b |\n\nProse here.\n"),
            Some("Prose here.".to_owned())
        );
        assert_eq!(
            readme_summary("```\ncode\n```\n\nText.\n"),
            Some("Text.".to_owned())
        );
    }

    #[test]
    fn reads_root_lists_and_status() {
        let lists = root_lists(ROOT_ISSUES);
        assert_eq!(lists.priority, vec!["app".to_owned()]);
        assert_eq!(lists.on_hold, vec!["tool".to_owned()]);
        assert_eq!(lists.status_of("app"), ProjectStatus::Priority);
        assert_eq!(lists.status_of("app/web"), ProjectStatus::Priority);
        assert_eq!(lists.status_of("tool"), ProjectStatus::OnHold);
        assert_eq!(lists.status_of("other"), ProjectStatus::Active);
    }

    #[test]
    fn on_hold_wins_over_priority() {
        let lists = root_lists("# Priority Projects\n- a\n\n# On Hold Projects\n- a\n");
        assert_eq!(lists.status_of("a"), ProjectStatus::OnHold);
    }

    #[test]
    fn parses_markers() {
        assert_eq!(
            parse_marker("TODO (agent-generated, agent-ignore): Do it."),
            Some((
                WorkKind::Todo,
                vec!["agent-generated".to_owned(), "agent-ignore".to_owned()],
                "Do it.".to_owned()
            ))
        );
        assert_eq!(
            parse_marker("FIX: Broken."),
            Some((WorkKind::Fix, vec![], "Broken.".to_owned()))
        );
        assert_eq!(
            parse_marker("TODO Plain text"),
            Some((WorkKind::Todo, vec![], "Plain text".to_owned()))
        );
        assert_eq!(parse_marker("TODOS are fine"), None);
        assert_eq!(parse_marker("FIXME later"), None);
        assert_eq!(parse_marker("Not a marker"), None);
        assert_eq!(parse_marker("TODO (unclosed: text"), None);
    }

    #[test]
    fn reads_issues_items_with_sections_and_epics() {
        let items = issues_items("app/ISSUES.md", Some("app"), APP_ISSUES);
        let summary: Vec<_> = items
            .iter()
            .map(|i| {
                (
                    i.kind,
                    i.line,
                    i.section.as_deref(),
                    i.epic.as_deref(),
                    i.text.as_str(),
                )
            })
            .collect();
        assert_eq!(
            summary,
            vec![
                (
                    WorkKind::Todo,
                    3,
                    Some("Priority Issues"),
                    None,
                    "Flat item."
                ),
                (
                    WorkKind::Todo,
                    7,
                    Some("Priority Issues"),
                    Some("Big epic"),
                    "Epic item."
                ),
                (
                    WorkKind::Fix,
                    11,
                    Some("Priority Issues"),
                    Some("Big epic / Phase 1: start"),
                    "Phase item."
                ),
                (WorkKind::Todo, 20, Some("Backlog"), None, "Later."),
            ]
        );
        assert_eq!(
            items[0].tags,
            vec!["agent-generated".to_owned(), "agent-ignore".to_owned()]
        );
        assert_eq!(items[2].tags, vec!["agent-ignore".to_owned()]);
        assert!(items.iter().all(|i| i.project.as_deref() == Some("app")
            && i.file == "app/ISSUES.md"
            && i.source == WorkSource::IssuesFile));
    }

    #[test]
    fn owning_project_picks_the_longest_root() {
        let roots = vec!["app".to_owned(), "app/web".to_owned()];
        assert_eq!(owning_project("app/web/ISSUES.md", &roots), Some("app/web"));
        assert_eq!(owning_project("app/ISSUES.md", &roots), Some("app"));
        assert_eq!(owning_project("application/ISSUES.md", &roots), None);
        assert_eq!(owning_project("ISSUES.md", &roots), None);
    }
}
