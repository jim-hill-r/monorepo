//! `chores badges`: the Attention section at the top of a PR description
//! (Jim, 2026-11-27), telling a reviewer how closely to look:
//!
//! - **Requirements**: a project's `docs/requirements.md` changed. Red when
//!   a `must` requirement is added, removed or edited (or one stops or
//!   starts being a `must`), amber for `should` and `could` ones.
//! - **Cost**: a file that sets what the repo uses or costs changed (the
//!   cost files of the PR cost summary).
//! - **Threat/risk**: a file that sets what is exposed or trusted changed:
//!   the security files of the PR security summary (threat models,
//!   `_headers`, `wrangler` configs, workflows, gitleaks, cargo-deny and
//!   zizmor configs) plus sign-in, secrets and credential code and docs.
//! - **Docs**: whether the PR changes only docs (Jim, 2026-11-28: "i want
//!   the attention section to state if it is only docs being updated which
//!   includes plans"). Docs are Markdown anywhere (plans, decisions, retros,
//!   user guides, READMEs, issue files, skills) and images in a `docs/`
//!   folder. A docs-only PR is the `docs-reviewer`'s (decision 0019).
//!
//! Whether a risk change *weakens* security is the Security summary job's
//! call (`threat summary`, `needs-human`); this line only says where to
//! look. The rules are under "Attention tags" in
//! `standards/docs/pull-requests.md`.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Serialize;

/// The heading of the Attention section, the first section of every PR
/// description (Jim, 2026-11-27).
pub const HEADING: &str = "## Attention";

/// The attention tags: a closed set, so the PR-001 check can reject any
/// other. Each Attention section has exactly one requirements tag, one cost
/// tag, one threat/risk tag and one docs tag. Keep in step with `standards/scripts/pr-summary-check.sh`
/// and "Attention tags" in `standards/docs/pull-requests.md`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tag {
    /// A `must` requirement was added, removed or edited.
    MustRequirementChanged,
    /// A `should` or `could` requirement, or text outside any, changed.
    RequirementChanged,
    /// No `docs/requirements.md` changed.
    RequirementsUnchanged,
    /// A cost file changed.
    CostChanged,
    /// No cost file changed.
    CostUnchanged,
    /// A threat or risk file changed.
    ThreatRiskChanged,
    /// No threat or risk file changed.
    ThreatRiskUnchanged,
    /// Every changed file is docs.
    DocsOnly,
    /// Docs changed alongside other files.
    DocsChanged,
    /// No docs changed.
    DocsUnchanged,
}

/// Which question a tag answers; a section has one tag per group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Requirements,
    Cost,
    ThreatRisk,
    Docs,
}

impl Tag {
    pub const ALL: [Tag; 10] = [
        Tag::MustRequirementChanged,
        Tag::RequirementChanged,
        Tag::RequirementsUnchanged,
        Tag::CostChanged,
        Tag::CostUnchanged,
        Tag::ThreatRiskChanged,
        Tag::ThreatRiskUnchanged,
        Tag::DocsOnly,
        Tag::DocsChanged,
        Tag::DocsUnchanged,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Tag::MustRequirementChanged => "must-requirement-changed",
            Tag::RequirementChanged => "requirement-changed",
            Tag::RequirementsUnchanged => "requirements-unchanged",
            Tag::CostChanged => "cost-changed",
            Tag::CostUnchanged => "cost-unchanged",
            Tag::ThreatRiskChanged => "threat-risk-changed",
            Tag::ThreatRiskUnchanged => "threat-risk-unchanged",
            Tag::DocsOnly => "docs-only",
            Tag::DocsChanged => "docs-changed",
            Tag::DocsUnchanged => "docs-unchanged",
        }
    }

    pub fn emoji(self) -> &'static str {
        match self {
            Tag::MustRequirementChanged => "🔴",
            Tag::RequirementChanged => "🟡",
            Tag::CostChanged | Tag::ThreatRiskChanged => "🟠",
            Tag::DocsOnly => "🔵",
            Tag::RequirementsUnchanged
            | Tag::CostUnchanged
            | Tag::ThreatRiskUnchanged
            | Tag::DocsChanged
            | Tag::DocsUnchanged => "🟢",
        }
    }

    pub fn group(self) -> Group {
        match self {
            Tag::MustRequirementChanged | Tag::RequirementChanged | Tag::RequirementsUnchanged => {
                Group::Requirements
            }
            Tag::CostChanged | Tag::CostUnchanged => Group::Cost,
            Tag::ThreatRiskChanged | Tag::ThreatRiskUnchanged => Group::ThreatRisk,
            Tag::DocsOnly | Tag::DocsChanged | Tag::DocsUnchanged => Group::Docs,
        }
    }

    pub fn parse(s: &str) -> Option<Tag> {
        Tag::ALL.into_iter().find(|t| t.as_str() == s)
    }
}

/// The tag name on one bullet of an Attention section
/// (`- 🔴 must-requirement-changed: ...`): the first word after the dash
/// that isn't an emoji, without a trailing colon or backticks.
pub fn bullet_tag(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("- ")?;
    rest.split_whitespace()
        .find(|w| w.chars().any(|c| c.is_ascii_alphanumeric()))
        .map(|w| w.trim_end_matches(':').trim_matches('`'))
}

/// What the badges say.
#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct Attention {
    /// IDs of changed `must` requirements, with the file.
    pub must: Vec<String>,
    /// IDs of other changed requirements.
    pub other: Vec<String>,
    /// Changed files that affect a threat or risk.
    pub risk: Vec<String>,
    /// Changed files that set what the repo uses or costs.
    #[serde(default)]
    pub cost: Vec<String>,
    /// Changed docs files.
    #[serde(default)]
    pub docs: Vec<String>,
    /// How many changed files are not docs.
    #[serde(default)]
    pub non_docs: usize,
}

/// Files whose change affects a threat or risk. Keep the first group in
/// step with `is_security_file` in `threat/model/src/summary.rs`.
pub fn is_risk_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let security_file = is_threat_file(path)
        || (path.starts_with(".github/workflows/")
            && (name.ends_with(".yml") || name.ends_with(".yaml")))
        || path == ".github/zizmor.yml"
        || matches!(
            name,
            "_headers"
                | "wrangler.toml"
                | "wrangler.json"
                | "wrangler.jsonc"
                | ".gitleaksignore"
                | ".gitleaks.toml"
                | "deny.toml"
        );
    // Sign-in, secrets, credentials and the rules about them.
    let auth_or_secrets = path.starts_with("auth_sdk/src/")
        || path.starts_with("infra/scripts/")
        || path.starts_with("infra/src/")
        || matches!(
            path,
            "standards/settings.toml"
                | "standards/docs/security.md"
                | "standards/docs/secret-rotation.md"
                | "standards/scripts/secrets-check.sh"
                | "standards/scripts/security-check.sh"
                | "SECURITY.md"
                | ".github/CODEOWNERS"
        );
    security_file || auth_or_secrets
}

/// Files whose change can move money or free-tier usage: the cost files of
/// the PR cost summary (`cost summary` in `standards/docs/cost.md`). That
/// job also reads workflow schedules and dependency lines; this tag only
/// says where to look, and the job's `cost` label stays the authority.
pub fn is_cost_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    path.ends_with("/docs/cost.toml")
        || matches!(
            path,
            "standards/radar/limits.toml" | "standards/radar/radar.json"
        )
        || matches!(
            name,
            "detectors.toml" | "wrangler.toml" | "wrangler.json" | "wrangler.jsonc"
        )
}

/// Docs: Markdown anywhere (plans, decisions, retros, user guides, READMEs,
/// issue files, skills, review decks) and images in a `docs/` folder. A
/// file's other tags still apply, so a changed `docs/requirements.md` or
/// `standards/docs/security.md` is docs and also flags its own group.
pub fn is_docs_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    let ext = name.rsplit_once('.').map_or("", |(_, e)| e);
    let in_docs = path.starts_with("docs/") || path.contains("/docs/");
    matches!(ext, "md" | "markdown" | "mdx")
        || (in_docs && matches!(ext, "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp"))
}

fn is_threat_file(path: &str) -> bool {
    path.strip_suffix("/docs/threat.toml")
        .is_some_and(|project| !project.is_empty() && !project.contains('/'))
}

/// A project's requirements file: `<project>/docs/requirements.md`.
pub fn is_requirements_file(path: &str) -> bool {
    path.strip_suffix("/docs/requirements.md")
        .is_some_and(|project| !project.is_empty() && !project.contains('/'))
}

/// Requirements by ID: (priority, the section's full text).
fn requirements(text: &str) -> BTreeMap<String, (String, String)> {
    let mut out = BTreeMap::new();
    let mut current: Option<(String, Vec<&str>)> = None;
    let finish = |cur: Option<(String, Vec<&str>)>, out: &mut BTreeMap<_, _>| {
        if let Some((id, lines)) = cur {
            let priority = lines
                .iter()
                .find_map(|l| l.trim().strip_prefix("- Priority:"))
                .map(|p| p.trim().to_lowercase())
                .unwrap_or_default();
            let body = lines.join("\n").trim().to_string();
            out.insert(id, (priority, body));
        }
    };
    for line in text.lines() {
        if line.starts_with("## ") || line.starts_with("# ") {
            finish(current.take(), &mut out);
            if let Some(rest) = line.strip_prefix("## ") {
                let id = rest.split(':').next().unwrap_or("").trim();
                let is_id = id.contains('-')
                    && id
                        .chars()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-');
                if is_id {
                    current = Some((id.to_string(), vec![line]));
                }
            }
        } else if let Some((_, lines)) = current.as_mut() {
            lines.push(line);
        }
    }
    finish(current.take(), &mut out);
    out
}

/// The requirement IDs that changed between two versions of a file, split
/// into (must, other). A requirement counts as `must` when it is a `must`
/// on either side.
pub fn changed_requirements(before: &str, after: &str) -> (Vec<String>, Vec<String>) {
    let (b, a) = (requirements(before), requirements(after));
    let mut must = Vec::new();
    let mut other = Vec::new();
    let ids: std::collections::BTreeSet<&String> = b.keys().chain(a.keys()).collect();
    for id in ids {
        let (old, new) = (b.get(id), a.get(id));
        if old == new {
            continue;
        }
        let is_must = [old, new]
            .iter()
            .any(|r| r.is_some_and(|(p, _)| p == "must"));
        if is_must {
            must.push(id.clone());
        } else {
            other.push(id.clone());
        }
    }
    (must, other)
}

/// Works out the badges from the changed files and a way to read a file
/// before and after the change.
pub fn attention(changed: &[String], read: impl Fn(&str, bool) -> Option<String>) -> Attention {
    let mut out = Attention::default();
    for path in changed {
        if is_risk_file(path) {
            out.risk.push(path.clone());
        }
        if is_cost_file(path) {
            out.cost.push(path.clone());
        }
        if is_docs_file(path) {
            out.docs.push(path.clone());
        } else {
            out.non_docs += 1;
        }
        if is_requirements_file(path) {
            let before = read(path, false).unwrap_or_default();
            let after = read(path, true).unwrap_or_default();
            let (must, other) = changed_requirements(&before, &after);
            if must.is_empty() && other.is_empty() && before != after {
                // Prose outside any requirement, such as the intro.
                out.other
                    .push(format!("{path} (text outside requirements)"));
            }
            out.must.extend(must);
            out.other.extend(other);
        }
    }
    out
}

pub fn list(items: &[String], max: usize) -> String {
    let shown: Vec<String> = items.iter().take(max).map(|i| format!("`{i}`")).collect();
    let more = items.len().saturating_sub(max);
    if more > 0 {
        format!("{} and {more} more", shown.join(", "))
    } else {
        shown.join(", ")
    }
}

impl Attention {
    /// Whether the PR changes only docs (and changes something).
    pub fn docs_only(&self) -> bool {
        !self.docs.is_empty() && self.non_docs == 0
    }

    /// The four tags: requirements, cost, threat/risk, docs.
    pub fn tags(&self) -> [Tag; 4] {
        let req = if !self.must.is_empty() {
            Tag::MustRequirementChanged
        } else if !self.other.is_empty() {
            Tag::RequirementChanged
        } else {
            Tag::RequirementsUnchanged
        };
        let cost = if self.cost.is_empty() {
            Tag::CostUnchanged
        } else {
            Tag::CostChanged
        };
        let risk = if self.risk.is_empty() {
            Tag::ThreatRiskUnchanged
        } else {
            Tag::ThreatRiskChanged
        };
        let docs = if self.docs_only() {
            Tag::DocsOnly
        } else if self.docs.is_empty() {
            Tag::DocsUnchanged
        } else {
            Tag::DocsChanged
        };
        [req, cost, risk, docs]
    }
}

/// The Attention section, in Markdown: always every group's tag, so a green
/// section says "nothing fundamental here" too.
pub fn render(a: &Attention) -> String {
    let mut s = format!("{HEADING}\n\n");
    for tag in a.tags() {
        let what = match tag {
            Tag::MustRequirementChanged => format!(": {}", list(&a.must, 5)),
            Tag::RequirementChanged => format!(": {}", list(&a.other, 5)),
            Tag::CostChanged => format!(
                ": {} (the Cost summary check says what moved)",
                list(&a.cost, 4)
            ),
            Tag::ThreatRiskChanged => format!(
                ": {} (the Security summary check says whether it weakens anything)",
                list(&a.risk, 4)
            ),
            Tag::DocsOnly => format!(
                ": only docs change, {} (a docs-reviewer's call)",
                list(&a.docs, 4)
            ),
            Tag::DocsChanged => format!(": {} alongside other files", list(&a.docs, 4)),
            Tag::RequirementsUnchanged
            | Tag::CostUnchanged
            | Tag::ThreatRiskUnchanged
            | Tag::DocsUnchanged => String::new(),
        };
        s.push_str(&format!("- {} {}{what}\n", tag.emoji(), tag.as_str()));
    }
    s
}

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))
}

/// The attention tags for the working tree against the merge base with
/// `base` (such as `origin/main`), counting uncommitted changes too.
pub fn for_repo(root: &Path, base: &str) -> Result<Attention> {
    let mb = git(root, &["merge-base", base, "HEAD"])?;
    if !mb.status.success() {
        bail!(
            "no merge base with {base}: {}",
            String::from_utf8_lossy(&mb.stderr).trim()
        );
    }
    let mb = String::from_utf8_lossy(&mb.stdout).trim().to_string();
    let mut changed: Vec<String> = Vec::new();
    for args in [
        vec!["diff", "--name-only", mb.as_str()],
        vec!["ls-files", "--others", "--exclude-standard"],
    ] {
        let out = git(root, &args)?;
        if !out.status.success() {
            bail!("git {} failed", args.join(" "));
        }
        changed.extend(
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(String::from),
        );
    }
    changed.sort();
    changed.dedup();
    Ok(attention(&changed, |path, after| {
        if after {
            std::fs::read_to_string(root.join(path)).ok()
        } else {
            git(root, &["show", &format!("{mb}:{path}")])
                .ok()
                .filter(|o| o.status.success())
                .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQS: &str = "# Cookbook requirements\n\nIntro.\n\n## COOK-001: A recipe every day\n\n- Priority: must\n- Accept: days 1 to 365.\n\n## COOK-003: The list adds up\n\n- Priority: should\n- Accept: summed.\n";

    #[test]
    fn risk_files() {
        for p in [
            "cookbook/docs/threat.toml",
            "cookbook/web/public/_headers",
            "pane/cloudflare/wrangler.toml",
            ".github/workflows/pull-request-ci.yml",
            "deny.toml",
            "standards/deny.toml",
            "auth_sdk/src/lib.rs",
            "infra/scripts/mint.sh",
            "standards/settings.toml",
        ] {
            assert!(is_risk_file(p), "{p}");
        }
        for p in [
            "cookbook/src/main.rs",
            "cookbook/web/docs/threat.toml",
            "README.md",
            ".github/dependabot.yml",
        ] {
            assert!(!is_risk_file(p), "{p}");
        }
    }

    #[test]
    fn cost_files() {
        for p in [
            "cookbook/docs/cost.toml",
            "standards/radar/limits.toml",
            "standards/radar/radar.json",
            "pane/cloudflare/wrangler.toml",
            "cost/detectors.toml",
        ] {
            assert!(is_cost_file(p), "{p}");
        }
        for p in ["cookbook/src/main.rs", "standards/docs/cost.md"] {
            assert!(!is_cost_file(p), "{p}");
        }
    }

    #[test]
    fn docs_files() {
        for p in [
            "README.md",
            "cookbook/docs/plans/2026-11-x.md",
            "standards/docs/decisions/0019-review-roles.md",
            "pane/docs/issues/fix-x.md",
            ".agents/skills/code-review/SKILL.md",
            "cookbook/docs/user/shot.png",
            "docs/reviews/x/diagram.svg",
        ] {
            assert!(is_docs_file(p), "{p}");
        }
        for p in [
            "cookbook/docs/cost.toml",
            "cookbook/src/main.rs",
            "cookbook/web/public/logo.png",
            "AGENTS",
        ] {
            assert!(!is_docs_file(p), "{p}");
        }
    }

    #[test]
    fn docs_only_tag() {
        let tag = |files: &[&str]| {
            let changed: Vec<String> = files.iter().map(|f| (*f).to_string()).collect();
            attention(&changed, |_, _| None).tags()[3]
        };
        let plan = "pane/docs/plans/2026-11-x.md";
        assert_eq!(tag(&[plan, "README.md"]), Tag::DocsOnly);
        assert_eq!(tag(&[plan, "pane/src/lib.rs"]), Tag::DocsChanged);
        assert_eq!(tag(&["pane/src/lib.rs"]), Tag::DocsUnchanged);
        assert_eq!(tag(&[]), Tag::DocsUnchanged);
        let only = attention(&[plan.to_string()], |_, _| None);
        assert!(render(&only).contains(
            "- 🔵 docs-only: only docs change, `pane/docs/plans/2026-11-x.md` (a docs-reviewer's call)\n"
        ));
    }

    #[test]
    fn must_and_other_changes() {
        let edited_must = REQS.replace("days 1 to 365.", "days 1 to 366.");
        assert_eq!(
            changed_requirements(REQS, &edited_must),
            (vec!["COOK-001".to_string()], vec![])
        );
        let edited_should = REQS.replace("summed.", "summed per unit.");
        assert_eq!(
            changed_requirements(REQS, &edited_should),
            (vec![], vec!["COOK-003".to_string()])
        );
        let promoted = REQS.replace("Priority: should", "Priority: must");
        assert_eq!(changed_requirements(REQS, &promoted).0, vec!["COOK-003"]);
        let added = format!("{REQS}\n## COOK-009: New\n\n- Priority: could\n");
        assert_eq!(changed_requirements(REQS, &added).1, vec!["COOK-009"]);
        assert_eq!(changed_requirements("", REQS).0, vec!["COOK-001"]);
        let intro = REQS.replace("Intro.", "New intro.");
        assert_eq!(changed_requirements(REQS, &intro), (vec![], vec![]));
    }

    #[test]
    fn renders_the_line() {
        assert_eq!(
            render(&Attention::default()),
            "## Attention\n\n- 🟢 requirements-unchanged\n- 🟢 cost-unchanged\n- 🟢 threat-risk-unchanged\n- 🟢 docs-unchanged\n"
        );
        let changed = vec![
            "cookbook/docs/requirements.md".to_string(),
            "cookbook/docs/threat.toml".to_string(),
            "cookbook/docs/cost.toml".to_string(),
            "cookbook/src/lib.rs".to_string(),
        ];
        let a = attention(&changed, |_, after| {
            Some(if after {
                REQS.replace("days 1 to 365.", "most days.")
            } else {
                REQS.to_string()
            })
        });
        assert_eq!(a.must, vec!["COOK-001"]);
        assert_eq!(a.risk, vec!["cookbook/docs/threat.toml"]);
        let line = render(&a);
        assert!(line.starts_with(HEADING));
        assert!(line.contains("- 🔴 must-requirement-changed: `COOK-001`\n"));
        assert!(line.contains("- 🟠 threat-risk-changed: `cookbook/docs/threat.toml`"));
        assert!(line.contains("- 🟠 cost-changed: `cookbook/docs/cost.toml`"));
        assert!(line.contains(
            "- 🟢 docs-changed: `cookbook/docs/requirements.md` alongside other files\n"
        ));
        let intro_only = attention(&changed[..1], |_, after| {
            Some(if after {
                format!("{REQS}x")
            } else {
                REQS.to_string()
            })
        });
        assert!(render(&intro_only).contains("- 🟡 requirement-changed: "));
    }

    #[test]
    fn tags_round_trip_and_bullets_parse() {
        for tag in Tag::ALL {
            assert_eq!(Tag::parse(tag.as_str()), Some(tag));
            let bullet = format!("- {} {}: `x`", tag.emoji(), tag.as_str());
            assert_eq!(bullet_tag(&bullet), Some(tag.as_str()));
        }
        assert_eq!(Tag::parse("looks-fine"), None);
        assert_eq!(
            bullet_tag("- `threat-risk-unchanged`"),
            Some("threat-risk-unchanged")
        );
        assert_eq!(bullet_tag("not a bullet"), None);
    }

    #[test]
    fn lists_are_capped() {
        let items: Vec<String> = (1..=7).map(|i| format!("f{i}")).collect();
        assert_eq!(list(&items, 2), "`f1`, `f2` and 5 more");
    }
}
