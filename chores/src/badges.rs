//! `chores badges`: the attention line at the top of a PR description
//! (Jim, 2026-11-27), telling a reviewer how closely to look:
//!
//! - **Requirements**: a project's `docs/requirements.md` changed. Red when
//!   a `must` requirement is added, removed or edited (or one stops or
//!   starts being a `must`), amber for `should` and `could` ones.
//! - **Threat/risk**: a file that sets what is exposed or trusted changed:
//!   the security files of the PR security summary (threat models,
//!   `_headers`, `wrangler` configs, workflows, gitleaks, cargo-deny and
//!   zizmor configs) plus sign-in, secrets and credential code and docs.
//!
//! Whether a risk change *weakens* security is the Security summary job's
//! call (`threat summary`, `needs-human`); this line only says where to
//! look. The rules are under "Attention badges" in
//! `standards/docs/pull-requests.md`.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Serialize;

/// Every attention line starts with this, so the PR-001 check skips it.
pub const PREFIX: &str = "> **Attention:**";

/// What the badges say.
#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct Attention {
    /// IDs of changed `must` requirements, with the file.
    pub must: Vec<String>,
    /// IDs of other changed requirements.
    pub other: Vec<String>,
    /// Changed files that affect a threat or risk.
    pub risk: Vec<String>,
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

fn list(items: &[String], max: usize) -> String {
    let shown: Vec<String> = items.iter().take(max).map(|i| format!("`{i}`")).collect();
    let more = items.len().saturating_sub(max);
    if more > 0 {
        format!("{} and {more} more", shown.join(", "))
    } else {
        shown.join(", ")
    }
}

/// The attention line, in Markdown: always both badges, so a green line
/// says "nothing fundamental here" too.
pub fn render(a: &Attention) -> String {
    let req = if !a.must.is_empty() {
        format!("🔴 must requirement changed: {}", list(&a.must, 5))
    } else if !a.other.is_empty() {
        format!("🟡 requirement changed: {}", list(&a.other, 5))
    } else {
        "🟢 requirements unchanged".to_string()
    };
    let risk = if a.risk.is_empty() {
        "🟢 threat/risk unchanged".to_string()
    } else {
        format!(
            "🟠 threat/risk files changed: {} (the Security summary check says whether it weakens anything)",
            list(&a.risk, 4)
        )
    };
    format!("{PREFIX} {req} · {risk}")
}

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))
}

/// The attention line for the working tree against the merge base with
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
            "> **Attention:** 🟢 requirements unchanged · 🟢 threat/risk unchanged"
        );
        let changed = vec![
            "cookbook/docs/requirements.md".to_string(),
            "cookbook/docs/threat.toml".to_string(),
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
        assert!(line.starts_with(PREFIX));
        assert!(line.contains("🔴 must requirement changed: `COOK-001`"));
        assert!(line.contains("🟠 threat/risk files changed: `cookbook/docs/threat.toml`"));
        let intro_only = attention(&changed[..1], |_, after| {
            Some(if after {
                format!("{REQS}x")
            } else {
                REQS.to_string()
            })
        });
        assert!(render(&intro_only).contains("🟡 requirement changed"));
    }

    #[test]
    fn lists_are_capped() {
        let items: Vec<String> = (1..=7).map(|i| format!("f{i}")).collect();
        assert_eq!(list(&items, 2), "`f1`, `f2` and 5 more");
    }
}
