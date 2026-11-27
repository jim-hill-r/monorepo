//! The PR security summary (`threat summary --base <ref>`): what a change
//! does to the repo's security files, and whether it weakens anything so a
//! person has to sign off. The labels and what counts as weakening are in
//! `standards/docs/security.md` ("PR security summary").
//!
//! No I/O: the caller passes each changed security file's text before and
//! after the change.

use std::collections::BTreeMap;

use crate::{Access, Auth, Surface, THREAT_FILE, ThreatFile, parse_threat_file};

/// The label every PR that changes a security file gets.
pub const SECURITY_LABEL: &str = "security";
/// The label a PR that weakens security gets, so PR triage waits for Jim.
pub const NEEDS_HUMAN_LABEL: &str = "needs-human";

const WORKFLOWS: &str = ".github/workflows/";
const ZIZMOR_CONFIG: &str = ".github/zizmor.yml";

/// One changed file, as text before and after (`None` when it didn't
/// exist on that side).
#[derive(Debug, Clone, PartialEq)]
pub struct FilePair {
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// One line of the summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// The file it is about.
    pub path: String,
    pub what: String,
    /// It makes something more exposed or adds an exception, so a person
    /// signs off.
    pub weakens: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Summary {
    /// The changed files that are security files.
    pub files: Vec<String>,
    pub changes: Vec<Change>,
    /// The checks that run for this change, beyond the always-on ones.
    pub guards: Vec<&'static str>,
}

impl Summary {
    pub fn weakens(&self) -> bool {
        self.changes.iter().any(|c| c.weakens)
    }

    /// `security` when a security file changed, and `needs-human` too when
    /// the change weakens security.
    pub fn labels(&self) -> Vec<&'static str> {
        let mut labels = Vec::new();
        if !self.files.is_empty() {
            labels.push(SECURITY_LABEL);
        }
        if self.weakens() {
            labels.push(NEEDS_HUMAN_LABEL);
        }
        labels
    }

    /// The job summary, in Markdown.
    pub fn render(&self) -> String {
        let mut out = String::from("## Security summary\n\n");
        if self.files.is_empty() {
            out.push_str("No security files changed.\n");
        } else {
            out.push_str("Security files changed:\n\n");
            for f in &self.files {
                out.push_str(&format!("- `{f}`\n"));
            }
            let weak: Vec<&Change> = self.changes.iter().filter(|c| c.weakens).collect();
            let other: Vec<&Change> = self.changes.iter().filter(|c| !c.weakens).collect();
            if !weak.is_empty() {
                out.push_str("\n**Weakens security, so Jim signs off (`needs-human`):**\n\n");
                for c in weak {
                    out.push_str(&format!("- `{}`: {}\n", c.path, c.what));
                }
            }
            if !other.is_empty() {
                out.push_str("\nOther changes:\n\n");
                for c in other {
                    out.push_str(&format!("- `{}`: {}\n", c.path, c.what));
                }
            }
        }
        out.push_str(
            "\nChecks: SEC-001 (Secret scan) and the threat check in `//:audit` run on every PR",
        );
        if !self.guards.is_empty() {
            out.push_str(&format!(
                ", plus {} for these files",
                self.guards.join(", ")
            ));
        }
        out.push_str(". CodeQL's result is on the PR's checks.\n");
        let labels = self.labels();
        if !labels.is_empty() {
            let labels: Vec<String> = labels.iter().map(|l| format!("`{l}`")).collect();
            out.push_str(&format!("\nLabels: {}.\n", labels.join(", ")));
        }
        out
    }
}

/// Whether a repo-relative path is a security file: a threat model, a
/// `_headers` or `wrangler` config, a workflow, or a gitleaks, cargo-deny
/// or zizmor config (where exceptions live).
pub fn is_security_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    is_threat_file(path)
        || path.starts_with(WORKFLOWS) && (name.ends_with(".yml") || name.ends_with(".yaml"))
        || path == ZIZMOR_CONFIG
        || matches!(
            name,
            "_headers"
                | "wrangler.toml"
                | "wrangler.json"
                | "wrangler.jsonc"
                | ".gitleaksignore"
                | ".gitleaks.toml"
                | "deny.toml"
        )
}

fn is_threat_file(path: &str) -> bool {
    path.strip_suffix(THREAT_FILE)
        .and_then(|p| p.strip_suffix('/'))
        .is_some_and(|project| !project.is_empty() && !project.contains('/'))
}

/// The checks that run only when their input files change.
fn guards(changed: &[String]) -> Vec<&'static str> {
    let any = |f: &dyn Fn(&str) -> bool| changed.iter().any(|p| f(p));
    let name = |p: &str| p.rsplit('/').next().unwrap_or(p).to_string();
    let mut out = Vec::new();
    if any(&|p| matches!(name(p).as_str(), "Cargo.lock" | "deny.toml")) {
        out.push("SEC-003 (cargo-deny)");
    }
    if any(&|p| name(p) == "package-lock.json") {
        out.push("SEC-004 (npm audit)");
    }
    if any(&|p| p.starts_with(WORKFLOWS) || p == ZIZMOR_CONFIG) {
        out.push("SEC-005 (zizmor)");
    }
    out
}

/// The summary for a change. `changed` is every changed path; `pairs` holds
/// the security files among them.
pub fn summarize(changed: &[String], pairs: &[FilePair]) -> Summary {
    let mut pairs: Vec<&FilePair> = pairs.iter().filter(|p| is_security_file(&p.path)).collect();
    pairs.sort_by(|a, b| a.path.cmp(&b.path));
    let mut changes = Vec::new();
    for pair in &pairs {
        let before = pair.before.as_deref().unwrap_or("");
        let after = pair.after.as_deref().unwrap_or("");
        let name = pair.path.rsplit('/').next().unwrap_or(&pair.path);
        let mut push = |what: String, weakens: bool| {
            changes.push(Change {
                path: pair.path.clone(),
                what,
                weakens,
            });
        };
        if is_threat_file(&pair.path) {
            threat_changes(pair, &mut push);
        } else if pair.path.starts_with(WORKFLOWS) {
            workflow_changes(before, after, &mut push);
        } else if name == ".gitleaksignore" {
            let added = entries(after).saturating_sub(entries(before));
            if added > 0 {
                push(format!("{added} new gitleaks exception(s)"), true);
            }
        } else if name == "deny.toml" {
            let added = advisory_ids(after).saturating_sub(advisory_ids(before));
            if added > 0 {
                push(format!("{added} new advisory exception(s)"), true);
            }
        } else if pair.path == ZIZMOR_CONFIG {
            let added = entries(after).saturating_sub(entries(before));
            if added > 0 {
                push(format!("{added} new zizmor exception line(s)"), true);
            }
        }
    }
    Summary {
        files: pairs.iter().map(|p| p.path.clone()).collect(),
        changes,
        guards: guards(changed),
    }
}

/// Non-blank lines that aren't comments.
fn entries(text: &str) -> usize {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .count()
}

/// RustSec and GitHub advisory ids, outside comments.
fn advisory_ids(text: &str) -> usize {
    text.lines()
        .map(|l| l.split('#').next().unwrap_or(""))
        .map(|l| l.matches("RUSTSEC-").count() + l.matches("GHSA-").count())
        .sum()
}

/// How exposed a surface with this access is: higher is more public.
fn exposure(access: Access) -> u8 {
    match access {
        Access::None => 0,
        Access::CloudflareAccess | Access::Token => 1,
        Access::Auth0 => 2,
        Access::Public => 3,
    }
}

fn parse(text: Option<&str>) -> Result<Option<ThreatFile>, ()> {
    match text {
        None => Ok(None),
        Some(t) => parse_threat_file(t).map(Some).map_err(|_| ()),
    }
}

fn threat_changes(pair: &FilePair, push: &mut dyn FnMut(String, bool)) {
    let (Ok(before), Ok(after)) = (parse(pair.before.as_deref()), parse(pair.after.as_deref()))
    else {
        push(
            "doesn't parse on one side, so it can't be compared (the threat check reports it)"
                .into(),
            true,
        );
        return;
    };
    let Some(after) = after.as_ref() else {
        push("threat model deleted".into(), true);
        return;
    };
    let no_surfaces = Vec::new();
    let no_secrets = Vec::new();
    let no_threats = Vec::new();
    let (b_surface, b_secret, b_auth, b_threat) = match before.as_ref() {
        Some(b) => (&b.surface, &b.secret, b.auth.as_ref(), &b.threat),
        None => (&no_surfaces, &no_secrets, None, &no_threats),
    };

    let old: BTreeMap<&str, &Surface> = b_surface.iter().map(|s| (s.name.as_str(), s)).collect();
    for s in &after.surface {
        match old.get(s.name.as_str()) {
            None => push(
                format!(
                    "new surface `{}` ({}, data {})",
                    s.name,
                    s.access.as_str(),
                    s.data.as_str()
                ),
                s.access != Access::None,
            ),
            Some(o) => surface_changes(o, s, push),
        }
    }
    for s in b_surface {
        if !after.surface.iter().any(|a| a.name == s.name) {
            push(format!("surface `{}` removed", s.name), false);
        }
    }

    for s in &after.secret {
        match b_secret.iter().find(|b| b.name == s.name) {
            None => push(format!("new secret `{}`", s.name), true),
            Some(b) if b.scope != s.scope => push(
                format!(
                    "secret `{}` scope: {} -> {}",
                    s.name,
                    b.scope.as_deref().unwrap_or("unset"),
                    s.scope.as_deref().unwrap_or("unset")
                ),
                false,
            ),
            Some(_) => {}
        }
    }
    for s in b_secret {
        if !after.secret.iter().any(|a| a.name == s.name) {
            push(format!("secret `{}` removed", s.name), false);
        }
    }

    if b_auth != after.auth.as_ref() {
        push(
            format!("`[auth]` {}", auth_change(b_auth, after.auth.as_ref())),
            true,
        );
    }

    for t in &after.threat {
        if !b_threat.iter().any(|b| b.id == t.id) {
            push(format!("new threat `{}`", t.id), false);
        }
    }
    for t in b_threat {
        if !after.threat.iter().any(|a| a.id == t.id) {
            push(format!("threat `{}` removed", t.id), false);
        }
    }
}

fn auth_change(before: Option<&Auth>, after: Option<&Auth>) -> String {
    match (before, after) {
        (None, Some(a)) => format!("added ({}, {})", a.provider, a.flow.as_str()),
        (Some(_), None) => "removed".into(),
        (Some(b), Some(a)) if b.provider != a.provider || b.flow != a.flow => format!(
            "changed: {} {} -> {} {}",
            b.provider,
            b.flow.as_str(),
            a.provider,
            a.flow.as_str()
        ),
        _ => "changed (audience, callbacks or setting)".into(),
    }
}

fn surface_changes(old: &Surface, new: &Surface, push: &mut dyn FnMut(String, bool)) {
    if old.access != new.access {
        push(
            format!(
                "surface `{}` access: {} -> {}",
                new.name,
                old.access.as_str(),
                new.access.as_str()
            ),
            exposure(new.access) > exposure(old.access),
        );
    }
    if old.data != new.data {
        push(
            format!(
                "surface `{}` data: {} -> {}",
                new.name,
                old.data.as_str(),
                new.data.as_str()
            ),
            new.data > old.data,
        );
    }
    if old.previews != new.previews {
        let show = |a: Option<Access>| a.map_or("unset", Access::as_str);
        let wider = match (old.previews, new.previews) {
            (Some(o), Some(n)) => exposure(n) > exposure(o),
            (None, Some(n)) => n != Access::None,
            _ => false,
        };
        push(
            format!(
                "surface `{}` previews: {} -> {}",
                new.name,
                show(old.previews),
                show(new.previews)
            ),
            wider,
        );
    }
    if old.headers.is_some() && new.headers.is_none() {
        push(
            format!("surface `{}` no longer names its headers", new.name),
            true,
        );
    }
}

/// `permissions` lines granting write (by scope, `write-all` counts as
/// `*`), and whether it uses `pull_request_target` or `workflow_run`.
struct WorkflowFacts {
    writes: BTreeMap<String, usize>,
    target: bool,
    workflow_run: bool,
}

fn workflow_facts(text: &str) -> WorkflowFacts {
    let mut facts = WorkflowFacts {
        writes: BTreeMap::new(),
        target: false,
        workflow_run: false,
    };
    for line in text.lines() {
        let code = line.split(" #").next().unwrap_or("");
        let t = code.trim().trim_start_matches("- ").trim();
        if t.starts_with('#') || t.is_empty() {
            continue;
        }
        if t.contains("write-all") {
            *facts.writes.entry("*".into()).or_default() += 1;
        } else if let Some((key, value)) = t.split_once(':')
            && value.trim() == "write"
            && key.chars().all(|c| c.is_ascii_lowercase() || c == '-')
        {
            *facts.writes.entry(key.to_string()).or_default() += 1;
        }
        let word = |w: &str| {
            t.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .any(|x| x == w)
        };
        if word("pull_request_target") {
            facts.target = true;
        }
        if t.starts_with("workflow_run:") || t == "workflow_run" {
            facts.workflow_run = true;
        }
    }
    facts
}

fn workflow_changes(before: &str, after: &str, push: &mut dyn FnMut(String, bool)) {
    let old = workflow_facts(before);
    let new = workflow_facts(after);
    let wider: Vec<String> = new
        .writes
        .iter()
        .filter(|(k, n)| old.writes.get(*k).copied().unwrap_or(0) < **n)
        .map(|(k, _)| {
            if k == "*" {
                "write-all".into()
            } else {
                format!("{k}: write")
            }
        })
        .collect();
    if !wider.is_empty() {
        push(format!("permissions widen ({})", wider.join(", ")), true);
    }
    if new.target && !old.target {
        push("starts using `pull_request_target`".into(), true);
    }
    if new.workflow_run && !old.workflow_run {
        push("starts using `workflow_run`".into(), true);
    }
    if wider.is_empty() && !(new.target && !old.target) && !(new.workflow_run && !old.workflow_run)
    {
        push(
            "workflow changed; permissions and triggers didn't widen".into(),
            false,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(path: &str, before: Option<&str>, after: Option<&str>) -> FilePair {
        FilePair {
            path: path.into(),
            before: before.map(str::to_string),
            after: after.map(str::to_string),
        }
    }

    fn run(pairs: &[FilePair]) -> Summary {
        let changed: Vec<String> = pairs.iter().map(|p| p.path.clone()).collect();
        summarize(&changed, pairs)
    }

    fn weak(s: &Summary) -> Vec<&str> {
        s.changes
            .iter()
            .filter(|c| c.weakens)
            .map(|c| c.what.as_str())
            .collect()
    }

    const BASE: &str = r#"
reviewed = "2026-11-27"
summary = "x"

[[surface]]
name = "site"
kind = "pages"
access = "cloudflare-access"
previews = "cloudflare-access"
data = "private"
headers = "x/_headers"

[[secret]]
name = "TOKEN"
where = "github"
scope = "read"
"#;

    #[test]
    fn test_security_files() {
        for p in [
            "pane/docs/threat.toml",
            "pane/_headers",
            "pane/web/wrangler.toml",
            ".github/workflows/x.yml",
            ".gitleaksignore",
            "standards/deny.toml",
            ".github/zizmor.yml",
        ] {
            assert!(is_security_file(p), "{p}");
        }
        for p in [
            "pane/src/lib.rs",
            "pane/docs/cost.toml",
            "a/b/docs/threat.toml",
            "README.md",
        ] {
            assert!(!is_security_file(p), "{p}");
        }
    }

    #[test]
    fn test_unrelated_change_has_no_labels() {
        let s = summarize(&["pane/src/lib.rs".into()], &[]);
        assert!(s.labels().is_empty());
        assert!(s.render().contains("No security files changed."));
    }

    #[test]
    fn test_access_more_public_needs_human() {
        let after = BASE.replacen("access = \"cloudflare-access\"", "access = \"public\"", 1);
        let s = run(&[pair("pane/docs/threat.toml", Some(BASE), Some(&after))]);
        assert_eq!(
            weak(&s),
            ["surface `site` access: cloudflare-access -> public"]
        );
        assert_eq!(s.labels(), ["security", "needs-human"]);
    }

    #[test]
    fn test_access_more_private_is_only_security() {
        let after = BASE.replacen("access = \"cloudflare-access\"", "access = \"none\"", 1);
        let s = run(&[pair("pane/docs/threat.toml", Some(BASE), Some(&after))]);
        assert!(weak(&s).is_empty());
        assert_eq!(s.labels(), ["security"]);
        assert_eq!(s.changes.len(), 1);
    }

    #[test]
    fn test_data_previews_secret_and_auth() {
        let after = BASE
            .replace("data = \"private\"", "data = \"secret\"")
            .replace("previews = \"cloudflare-access\"", "previews = \"public\"")
            + "\n[[secret]]\nname = \"NEW\"\nwhere = \"worker\"\n\n[auth]\nprovider = \"auth0\"\nflow = \"authorization-code-pkce\"\n";
        let s = run(&[pair("pane/docs/threat.toml", Some(BASE), Some(&after))]);
        assert_eq!(
            weak(&s),
            [
                "surface `site` data: private -> secret",
                "surface `site` previews: cloudflare-access -> public",
                "new secret `NEW`",
                "`[auth]` added (auth0, authorization-code-pkce)",
            ]
        );
    }

    #[test]
    fn test_new_and_removed_files() {
        let s = run(&[pair("pane/docs/threat.toml", None, Some(BASE))]);
        assert_eq!(
            weak(&s),
            [
                "new surface `site` (cloudflare-access, data private)",
                "new secret `TOKEN`"
            ]
        );
        let s = run(&[pair("pane/docs/threat.toml", Some(BASE), None)]);
        assert_eq!(weak(&s), ["threat model deleted"]);
        let s = run(&[pair("pane/docs/threat.toml", Some(BASE), Some("nope ="))]);
        assert_eq!(s.labels(), ["security", "needs-human"]);
    }

    #[test]
    fn test_secret_removed_is_not_weakening() {
        let after = BASE
            .split("[[secret]]")
            .next()
            .unwrap_or_default()
            .to_string();
        let s = run(&[pair("pane/docs/threat.toml", Some(BASE), Some(&after))]);
        assert!(weak(&s).is_empty());
        assert_eq!(s.changes[0].what, "secret `TOKEN` removed");
    }

    #[test]
    fn test_workflow_permissions_and_triggers() {
        let before = "on:\n  pull_request:\npermissions:\n  contents: read\njobs:\n  a:\n    permissions:\n      issues: write  # labels\n";
        let s = run(&[pair(".github/workflows/x.yml", Some(before), Some(before))]);
        assert!(weak(&s).is_empty());
        assert_eq!(s.guards, ["SEC-005 (zizmor)"]);

        let after = before.replace("contents: read", "contents: write")
            + "  b:\n    permissions: write-all\n";
        let after = after.replace("pull_request:", "pull_request_target:");
        let s = run(&[pair(".github/workflows/x.yml", Some(before), Some(&after))]);
        assert_eq!(
            weak(&s),
            [
                "permissions widen (write-all, contents: write)",
                "starts using `pull_request_target`",
            ]
        );
        // A comment that mentions it doesn't count.
        let commented = format!("{before}# never pull_request_target\n");
        let s = run(&[pair(
            ".github/workflows/x.yml",
            Some(before),
            Some(&commented),
        )]);
        assert!(weak(&s).is_empty());
    }

    #[test]
    fn test_new_exceptions_need_human() {
        let s = run(&[
            pair(
                ".gitleaksignore",
                Some("# why\nabc:1\n"),
                Some("# why\nabc:1\n# fixture\ndef:2\n"),
            ),
            pair(
                "standards/deny.toml",
                Some("[advisories]\nignore = []\n"),
                Some(
                    "[advisories]\nignore = [\n  { id = \"RUSTSEC-2024-0001\", reason = \"x\" },\n]\n",
                ),
            ),
            pair(
                ".github/zizmor.yml",
                None,
                Some("rules:\n  artipacked:\n    ignore:\n      - x.yml\n"),
            ),
        ]);
        assert_eq!(
            weak(&s),
            [
                "4 new zizmor exception line(s)",
                "1 new gitleaks exception(s)",
                "1 new advisory exception(s)",
            ]
        );
        assert!(s.guards.contains(&"SEC-003 (cargo-deny)"));
        // Taking one away is fine.
        let s = run(&[pair(".gitleaksignore", Some("abc:1\n"), Some(""))]);
        assert!(weak(&s).is_empty());
        assert_eq!(s.labels(), ["security"]);
    }

    #[test]
    fn test_render_lists_weakening_first() {
        let after = BASE.replacen("access = \"cloudflare-access\"", "access = \"public\"", 1);
        let text = run(&[pair("pane/docs/threat.toml", Some(BASE), Some(&after))]).render();
        assert!(text.contains("**Weakens security, so Jim signs off (`needs-human`):**"));
        assert!(text.contains(
            "- `pane/docs/threat.toml`: surface `site` access: cloudflare-access -> public"
        ));
        assert!(text.contains("Labels: `security`, `needs-human`."));
    }
}
