//! Project isolation: every top-level project could be moved to another
//! repo or sold and still mostly work, and anything live it shares with
//! other projects is written down in its own `docs/isolation.md`, with a
//! migration guide for separating it out (decision 0026, "Project
//! isolation" in standards/docs/project-isolation.md).
//!
//!   ISO-001  The project has no `docs/isolation.md`.
//!   ISO-002  The guide is missing `## Moving out` or `## Shared resources`,
//!            or one of them has no text.
//!   ISO-003  The project's files use a shared resource (from [`SHARED`])
//!            that the guide has no `### ...` section for.
//!   ISO-004  A shared resource's section has no `#### Why it is shared` or
//!            no `#### Migration guide`, or one of them has no text.
//!   ISO-005  A `###` section under `## Shared resources` names no known
//!            resource id: add the resource to [`SHARED`] and the standard.
//!
//! Every finding is a warning until each project's guide is written (one
//! issue file per project, epic "Project isolation"); `--strict` fails on
//! any finding, and the audit switches to it once the epic is done. There is
//! no central baseline: what a project shares is found in its own files.
//!
//! isolation-check: names only (this file lists every resource's patterns).

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::user_docs::projects;

/// Where each project's isolation guide lives, from the project folder.
pub const GUIDE: &str = "docs/isolation.md";

/// The `##` sections every guide has, matched by the start of the heading,
/// ignoring case.
pub const SECTIONS: [&str; 2] = ["Moving out", "Shared resources"];

/// The `####` sections under each shared resource's `###` section.
pub const RESOURCE_SECTIONS: [&str; 2] = ["Why it is shared", "Migration guide"];

/// One live resource that more than one project uses.
#[derive(Debug, Clone, Copy)]
pub struct Resource {
    /// The id a guide's `###` heading names in backticks.
    pub id: &'static str,
    /// What it is, for messages.
    pub name: &'static str,
    /// The project that runs it, which is not asked to document it as
    /// shared (it documents its users instead). `None` for a vendor account.
    pub owner: Option<&'static str>,
    /// Text in a project's files that means it uses the resource.
    pub patterns: &'static [&'static str],
    /// File names whose presence means it uses the resource.
    pub files: &'static [&'static str],
}

/// The known shared live resources. Keep this in step with the table in
/// standards/docs/project-isolation.md (a test checks every id is there).
pub const SHARED: [Resource; 6] = [
    Resource {
        id: "cloudflare-account",
        name: "the one Cloudflare account, its workers.dev and pages.dev names and the deploy token",
        owner: Some("infra"),
        patterns: &[
            "wrangler deploy",
            "wrangler pages deploy",
            "standards/tasks/cloudflare-",
            "standards/tasks/bevy-game.toml",
        ],
        files: &["wrangler.toml"],
    },
    Resource {
        id: "auth0-tenant",
        name: "the one Auth0 tenant and its users",
        owner: Some("infra"),
        patterns: &["dev-jdadpn4pckxevrv5"],
        files: &[],
    },
    Resource {
        id: "telemetry-worker",
        name: "the jimhillr-telemetry Worker and its Analytics Engine dataset",
        owner: Some("telemetry"),
        patterns: &["jimhillr-telemetry"],
        files: &[],
    },
    Resource {
        id: "posthog-project",
        name: "the one PostHog project and its free recording quota",
        owner: None,
        patterns: &["phc_"],
        files: &[],
    },
    Resource {
        id: "flags-service",
        name: "the jimhillr-flags Worker and its D1 database (decision 0018)",
        owner: None,
        patterns: &["jimhillr-flags"],
        files: &[],
    },
    Resource {
        id: "products-d1",
        name: "the shared jimhillr-products D1 database (decision 0021)",
        owner: None,
        patterns: &["jimhillr-products", "tenant_store"],
        files: &[],
    },
];

/// Folders the scan skips: docs, issues and tests talk about resources
/// without using them, and the rest are build output.
const SKIP_DIRS: [&str; 11] = [
    "docs",
    "issues",
    "tests",
    "fixtures",
    "target",
    "node_modules",
    "dist",
    ".wrangler",
    "test-results",
    "playwright-report",
    "out",
];

/// File extensions the scan reads, the same as the cost check's detectors.
const EXTENSIONS: [&str; 17] = [
    "rs", "ts", "tsx", "js", "jsx", "mjs", "cjs", "html", "css", "toml", "json", "yml", "yaml",
    "sh", "py", "vue", "svelte",
];

/// A file that holds this text only names resources (this check, a
/// detector list, a test fixture) and is not scanned.
pub const NAMES_ONLY: &str = "isolation-check: names only";

/// Files larger than this are not read (generated bindings, lock data).
const MAX_FILE: u64 = 1_000_000;

/// One problem found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub rule: &'static str,
    pub project: String,
    pub message: String,
}

/// The result of a check over the repo.
#[derive(Debug, Default, Serialize)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// Each project and the shared resource ids its files use.
    pub uses: Vec<(String, Vec<&'static str>)>,
}

/// The ids of the shared resources a project's files use, apart from the
/// ones it owns.
pub fn uses(root: &Path, project: &str) -> Result<Vec<&'static str>> {
    let mut found = Vec::new();
    scan(&root.join(project), &mut found)?;
    Ok(SHARED
        .iter()
        .filter(|r| r.owner != Some(project) && found.contains(&r.id))
        .map(|r| r.id)
        .collect())
}

fn scan(dir: &Path, found: &mut Vec<&'static str>) -> Result<()> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(());
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            if !name.starts_with('.') && !SKIP_DIRS.contains(&name.as_str()) {
                scan(&path, found)?;
            }
            continue;
        }
        for r in &SHARED {
            if r.files.contains(&name.as_str()) && !found.contains(&r.id) {
                found.push(r.id);
            }
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !EXTENSIONS.contains(&ext) || entry.metadata()?.len() > MAX_FILE {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if text.contains(NAMES_ONLY) {
            continue;
        }
        for r in &SHARED {
            if !found.contains(&r.id) && r.patterns.iter().any(|p| text.contains(p)) {
                found.push(r.id);
            }
        }
    }
    Ok(())
}

/// A heading and whether any text follows it before the next heading of
/// the same or a higher level.
#[derive(Debug, Default)]
struct Section {
    title: String,
    text: bool,
    children: Vec<Section>,
}

/// The guide's `##` sections, each with its `###` sections, each with its
/// `####` sections. Text under a deeper heading counts for every heading
/// above it. Lines in fenced code are text, HTML comments are not.
fn outline(text: &str) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    let (mut in_fence, mut in_comment) = (false, false);
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            mark_text(&mut out);
            continue;
        }
        if in_fence {
            mark_text(&mut out);
            continue;
        }
        if in_comment || t.starts_with("<!--") {
            in_comment = !t.contains("-->");
            continue;
        }
        let level = t.chars().take_while(|c| *c == '#').count();
        let title = t.get(level..).unwrap_or("");
        if (1..=4).contains(&level) && title.starts_with(' ') {
            let section = Section {
                title: title.trim().to_string(),
                ..Section::default()
            };
            match level {
                1 => out.push(Section::default()),
                2 => out.push(section),
                3 => {
                    if let Some(h2) = out.last_mut() {
                        h2.children.push(section);
                    }
                }
                _ => {
                    if let Some(h3) = out.last_mut().and_then(|h2| h2.children.last_mut()) {
                        h3.children.push(section);
                    }
                }
            }
        } else if !t.is_empty() {
            mark_text(&mut out);
        }
    }
    out
}

fn mark_text(out: &mut [Section]) {
    if let Some(h2) = out.last_mut() {
        h2.text = true;
        if let Some(h3) = h2.children.last_mut() {
            h3.text = true;
            if let Some(h4) = h3.children.last_mut() {
                h4.text = true;
            }
        }
    }
}

fn starts(title: &str, prefix: &str) -> bool {
    let norm = |s: &str| s.trim().replace('\u{2019}', "'").to_lowercase();
    norm(title).starts_with(&norm(prefix))
}

/// The resource id a `###` heading names in backticks, if any.
fn heading_id(title: &str) -> Option<&str> {
    let start = title.find('`')? + 1;
    let len = title.get(start..)?.find('`')?;
    title.get(start..start + len)
}

/// Check one guide's text against the resources its project uses.
pub fn check_guide(project: &str, text: &str, used: &[&str]) -> Vec<Finding> {
    let finding = |rule, message: String| Finding {
        rule,
        project: project.to_string(),
        message,
    };
    let mut out = Vec::new();
    let sections = outline(text);
    for want in SECTIONS {
        match sections.iter().find(|s| starts(&s.title, want)) {
            None => out.push(finding(
                "ISO-002",
                format!("{project}/{GUIDE} has no `## {want}` section"),
            )),
            Some(s) if !s.text => out.push(finding(
                "ISO-002",
                format!("{project}/{GUIDE}: `## {want}` has no text"),
            )),
            Some(_) => {}
        }
    }
    let documented: Vec<&Section> = sections
        .iter()
        .filter(|s| starts(&s.title, "Shared resources"))
        .flat_map(|s| s.children.iter())
        .collect();
    for id in used {
        if !documented.iter().any(|s| heading_id(&s.title) == Some(id)) {
            let name = SHARED.iter().find(|r| r.id == *id).map_or("", |r| r.name);
            out.push(finding(
                "ISO-003",
                format!(
                    "{project} uses `{id}` ({name}) but {GUIDE} has no `### ... (`{id}`)` section under `## Shared resources`"
                ),
            ));
        }
    }
    for s in documented {
        let Some(id) = heading_id(&s.title).filter(|id| SHARED.iter().any(|r| r.id == *id)) else {
            out.push(finding(
                "ISO-005",
                format!(
                    "{project}/{GUIDE}: `### {}` names no known shared resource id; add it to chores/src/isolation.rs and standards/docs/project-isolation.md",
                    s.title
                ),
            ));
            continue;
        };
        for want in RESOURCE_SECTIONS {
            if !s.children.iter().any(|c| starts(&c.title, want) && c.text) {
                out.push(finding(
                    "ISO-004",
                    format!("{project}/{GUIDE}: `{id}` has no `#### {want}` with text"),
                ));
            }
        }
    }
    out
}

/// Check every top-level project.
pub fn check(root: &Path) -> Result<Report> {
    let mut report = Report::default();
    for project in projects(root)? {
        let used = uses(root, &project)?;
        match fs::read_to_string(root.join(&project).join(GUIDE)) {
            Ok(text) => report.findings.extend(check_guide(&project, &text, &used)),
            Err(_) => report.findings.push(Finding {
                rule: "ISO-001",
                project: project.clone(),
                message: format!(
                    "{project} has no {GUIDE} (shares: {})",
                    if used.is_empty() {
                        "nothing found".to_string()
                    } else {
                        used.join(", ")
                    }
                ),
            }),
        }
        report.uses.push((project, used));
    }
    Ok(report)
}

/// The report as text; `strict` labels findings as errors.
pub fn render(report: &Report, strict: bool) -> String {
    let label = if strict { "error" } else { "warning" };
    let mut out = String::new();
    for f in &report.findings {
        let _ = writeln!(out, "{label}: {} {}", f.rule, f.message);
    }
    if report.findings.is_empty() {
        out.push_str("Project isolation: every project's guide is complete.\n");
    } else {
        let _ = writeln!(
            out,
            "Project isolation: {} finding(s); see standards/docs/project-isolation.md.",
            report.findings.len()
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str = "# telemetry isolation

## Moving out

Copy the folder and the workflow.

## Shared resources

### Cloudflare account (`cloudflare-account`)

#### Why it is shared

Free plan, one account.

#### Migration guide

1. Make an account.
";

    fn rules(findings: &[Finding]) -> Vec<&str> {
        findings.iter().map(|f| f.rule).collect()
    }

    #[test]
    fn a_full_guide_passes() {
        assert!(check_guide("p", FULL, &["cloudflare-account"]).is_empty());
    }

    #[test]
    fn a_guide_that_shares_nothing_passes() {
        let text = "## Moving out\n\nCopy it.\n\n## Shared resources\n\nNone.\n";
        assert!(check_guide("p", text, &[]).is_empty());
    }

    #[test]
    fn missing_and_empty_sections() {
        let text = "## Moving out\n\n## Other\n\ntext\n";
        assert_eq!(rules(&check_guide("p", text, &[])), ["ISO-002", "ISO-002"]);
    }

    #[test]
    fn an_undocumented_resource() {
        let f = check_guide("p", FULL, &["cloudflare-account", "auth0-tenant"]);
        assert_eq!(rules(&f), ["ISO-003"]);
        assert!(f[0].message.contains("auth0-tenant"));
    }

    #[test]
    fn a_resource_without_a_migration_guide() {
        let text = FULL.replace("1. Make an account.\n", "");
        assert_eq!(
            rules(&check_guide("p", &text, &["cloudflare-account"])),
            ["ISO-004"]
        );
    }

    #[test]
    fn an_unknown_resource() {
        let text = FULL.replace("cloudflare-account", "mystery");
        assert_eq!(rules(&check_guide("p", &text, &[])), ["ISO-005"]);
    }

    #[test]
    fn headings_in_code_blocks_do_not_count() {
        let text = "## Moving out\n\n```\n## Shared resources\n```\n";
        assert_eq!(rules(&check_guide("p", text, &[])), ["ISO-002"]);
    }

    #[test]
    fn scanning_finds_patterns_and_files_but_not_docs_or_owned() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        for p in ["app", "telemetry"] {
            fs::create_dir_all(root.join(p).join("docs")).unwrap();
            fs::write(root.join(p).join("mise.toml"), "").unwrap();
        }
        fs::write(root.join("app/wrangler.toml"), "name = \"x\"").unwrap();
        fs::write(
            root.join("app/main.js"),
            "fetch('https://jimhillr-telemetry.x')",
        )
        .unwrap();
        fs::write(root.join("app/docs/notes.md"), "phc_ in docs").unwrap();
        fs::write(root.join("app/docs/a.toml"), "phc_ in docs").unwrap();
        fs::write(
            root.join("telemetry/wrangler.toml"),
            "name = \"jimhillr-telemetry\"",
        )
        .unwrap();
        assert_eq!(
            uses(root, "app").unwrap(),
            ["cloudflare-account", "telemetry-worker"]
        );
        assert_eq!(uses(root, "telemetry").unwrap(), ["cloudflare-account"]);
        let report = check(root).unwrap();
        assert_eq!(rules(&report.findings), ["ISO-001", "ISO-001"]);
    }

    #[test]
    fn the_standard_lists_every_resource() {
        let standard = include_str!("../../standards/docs/project-isolation.md");
        for r in &SHARED {
            assert!(
                standard.contains(&format!("`{}`", r.id)),
                "standards/docs/project-isolation.md lists `{}`",
                r.id
            );
        }
    }
}
