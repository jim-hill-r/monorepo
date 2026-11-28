//! The weekly live check: compares what is deployed on Cloudflare with what
//! the threat models claim, and what each declared URL actually answers
//! with what its surface says. No network here: `standards/scripts/threat-live.sh`
//! lists the account and makes the requests, and passes the results in.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::Deserialize;

use crate::{Access, Kind, REQUIRED_HEADERS, Surface, ThreatFile};

/// What the Cloudflare account has, as `threat-live.sh` lists it.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    #[serde(default)]
    pub pages: Vec<PagesProject>,
    /// Worker script names. `None` when the token can't list Workers.
    #[serde(default)]
    pub workers: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PagesProject {
    pub name: String,
    /// `<name>.pages.dev` and any custom domains.
    #[serde(default)]
    pub domains: Vec<String>,
    /// The newest preview deployment's URL, if it has one.
    #[serde(default)]
    pub preview_url: Option<String>,
}

/// One HTTP response, fetched without following redirects.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub url: String,
    /// `0` when the request failed (DNS, TLS, timeout).
    pub status: u16,
    #[serde(default)]
    pub location: Option<String>,
    /// Response headers, names lowercase.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
}

/// One project's parsed threat model.
pub struct Model<'a> {
    pub project: &'a str,
    pub file: &'a ThreatFile,
}

/// A URL to request, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub url: String,
    pub project: String,
    pub surface: String,
    /// What the surface says the URL should answer like.
    pub access: Access,
    /// Whether the required security headers are checked.
    pub headers: bool,
    /// A Pages preview deployment rather than production.
    pub preview: bool,
}

/// Something the live check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveFinding {
    /// The project whose threat model it is about, or `None` for something
    /// deployed that no threat model claims.
    pub project: Option<String>,
    pub message: String,
}

/// An issue file to add for something deployed that nobody claims.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Orphan {
    /// `pages` or `worker`.
    pub kind: &'static str,
    pub name: String,
}

impl Orphan {
    /// Its path from the repo root, in the threat project's `docs/issues/`.
    pub fn issue_path(&self) -> String {
        format!(
            "threat/docs/issues/claim-or-delete-{}-{}.md",
            self.kind,
            slug(&self.name)
        )
    }

    /// The issue file, in the format of `standards/docs/issue-management.md`.
    pub fn issue_text(&self, today: &str) -> String {
        let what = match self.kind {
            "pages" => format!("Cloudflare Pages project `{}`", self.name),
            _ => format!("Cloudflare Worker `{}`", self.name),
        };
        format!(
            "---\nkind: fix\npriority: priority\ntags: [agent-generated]\nepic: Threat check\nadded: {today}\n---\n\nThe weekly live threat check found the {what} deployed, but no project's `docs/threat.toml` has a surface on it, so nothing checks its access or headers. Claim it (add the surface to the owning project's threat model) or, if nothing uses it, delete it; deleting a Cloudflare project is a human task in the owning project's `docs/issues/human/` with a runbook.\n"
        )
    }
}

fn slug(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

/// The lowercase host of `https://host[:port]/...`, or `None`.
pub fn host_of(url: &str) -> Option<String> {
    let rest = url.split_once("://")?.1;
    let host = rest.split(['/', ':', '?', '#']).next()?;
    (!host.is_empty()).then(|| host.to_ascii_lowercase())
}

fn on_host(host: &str, domain: &str) -> bool {
    let domain = domain.to_ascii_lowercase();
    host == domain || host.ends_with(&format!(".{domain}"))
}

/// Whether a surface's URL is on this Pages project.
fn claims_pages(surface: &Surface, project: &PagesProject) -> bool {
    let Some(host) = surface.url.as_deref().and_then(host_of) else {
        return false;
    };
    let default = format!("{}.pages.dev", project.name);
    std::iter::once(default.as_str())
        .chain(project.domains.iter().map(String::as_str))
        .any(|d| on_host(&host, d))
}

/// Whether a surface's URL is on this Worker's `workers.dev` address.
fn claims_worker(surface: &Surface, worker: &str) -> bool {
    surface
        .url
        .as_deref()
        .and_then(host_of)
        .is_some_and(|h| h.ends_with(".workers.dev") && h.split('.').next() == Some(worker))
}

fn probed(surface: &Surface) -> bool {
    surface.kind.is_web()
        && surface.access != Access::None
        && surface
            .url
            .as_deref()
            .is_some_and(|u| u.starts_with("https://") && !u.contains('*'))
}

/// Every URL to request: each reachable web surface's `url`, and the
/// newest preview of each Pages project a surface claims.
pub fn targets(models: &[Model], account: &Account) -> Vec<Target> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    for m in models {
        for s in m.file.surface.iter().filter(|s| probed(s)) {
            let Some(url) = s.url.clone() else { continue };
            if seen.insert(url.clone()) {
                out.push(Target {
                    url,
                    project: m.project.to_string(),
                    surface: s.name.clone(),
                    access: s.access,
                    headers: s.access == Access::Public,
                    preview: false,
                });
            }
            if s.kind != Kind::Pages {
                continue;
            }
            for p in account.pages.iter().filter(|p| claims_pages(s, p)) {
                let Some(url) = p.preview_url.clone() else {
                    continue;
                };
                if seen.insert(url.clone()) {
                    let access = s.previews.unwrap_or(s.access);
                    out.push(Target {
                        url,
                        project: m.project.to_string(),
                        surface: s.name.clone(),
                        access,
                        headers: access == Access::Public,
                        preview: true,
                    });
                }
            }
        }
    }
    out
}

/// Compares the account and the responses with the threat models.
pub fn check_live(
    models: &[Model],
    account: &Account,
    probes: &[Probe],
) -> (Vec<LiveFinding>, Vec<Orphan>) {
    let mut findings = Vec::new();
    let mut orphans = Vec::new();
    let surfaces = || models.iter().flat_map(|m| m.file.surface.iter());

    for p in &account.pages {
        if !surfaces().any(|s| claims_pages(s, p)) {
            findings.push(LiveFinding {
                project: None,
                message: format!(
                    "Pages project `{}` is deployed, but no docs/threat.toml has a surface on it",
                    p.name
                ),
            });
            orphans.push(Orphan {
                kind: "pages",
                name: p.name.clone(),
            });
        }
    }
    for w in account.workers.iter().flatten() {
        if !surfaces().any(|s| claims_worker(s, w)) {
            findings.push(LiveFinding {
                project: None,
                message: format!(
                    "Worker `{w}` is deployed, but no docs/threat.toml has a surface on its workers.dev address"
                ),
            });
            orphans.push(Orphan {
                kind: "worker",
                name: w.clone(),
            });
        }
    }

    let by_url: BTreeMap<&str, &Probe> = probes.iter().map(|p| (p.url.as_str(), p)).collect();
    for t in targets(models, account) {
        let what = if t.preview {
            format!("surface `{}` preview {}", t.surface, t.url)
        } else {
            format!("surface `{}` at {}", t.surface, t.url)
        };
        let mut add = |message: String| {
            findings.push(LiveFinding {
                project: Some(t.project.clone()),
                message,
            });
        };
        let Some(probe) = by_url.get(t.url.as_str()) else {
            add(format!("{what} was not requested"));
            continue;
        };
        if probe.status == 0 {
            add(format!("{what} didn't answer"));
            continue;
        }
        let to_access = probe
            .location
            .as_deref()
            .and_then(host_of)
            .is_some_and(|h| h.ends_with(".cloudflareaccess.com"));
        match t.access {
            Access::CloudflareAccess => {
                if !((300..400).contains(&probe.status) && to_access) {
                    add(format!(
                        "{what} should redirect to Cloudflare Access, but answered {}{}",
                        probe.status,
                        probe
                            .location
                            .as_deref()
                            .map(|l| format!(" to {l}"))
                            .unwrap_or_default()
                    ));
                }
            }
            // A Pages site behind in-app sign-in still serves its static
            // files; only an API must refuse.
            Access::Auth0 | Access::Token => {
                if !is_pages(models, &t) && !matches!(probe.status, 401 | 403) {
                    add(format!(
                        "{what} should refuse a request without a token (401 or 403), but answered {}",
                        probe.status
                    ));
                }
            }
            Access::Public | Access::None => {
                if to_access {
                    add(format!(
                        "{what} says access = \"public\" but redirects to Cloudflare Access; update the threat model"
                    ));
                } else if probe.status >= 400 {
                    add(format!("{what} answered {}", probe.status));
                }
            }
        }
        if t.headers && (200..300).contains(&probe.status) {
            let text: String = probe
                .headers
                .iter()
                .map(|(k, v)| format!("{}: {}\n", k.to_ascii_lowercase(), v.to_ascii_lowercase()))
                .collect();
            let missing: Vec<&str> = REQUIRED_HEADERS
                .iter()
                .filter(|(_, spellings)| !spellings.iter().any(|h| text.contains(h)))
                .map(|(name, _)| *name)
                .collect();
            if !missing.is_empty() {
                add(format!(
                    "{what} is public but doesn't send {}",
                    missing.join(", ")
                ));
            }
        }
    }
    (findings, orphans)
}

fn is_pages(models: &[Model], t: &Target) -> bool {
    models
        .iter()
        .filter(|m| m.project == t.project)
        .flat_map(|m| m.file.surface.iter())
        .any(|s| s.name == t.surface && s.kind == Kind::Pages)
}

/// The Markdown report for the job summary.
pub fn render_live(
    account: &Account,
    targets: usize,
    findings: &[LiveFinding],
    orphans: &[Orphan],
    today: &str,
) -> String {
    let mut out = String::from("## Live threat check\n\n");
    let workers = account.workers.as_ref().map_or(
        "not listed (the token can't read Workers)".to_string(),
        |w| w.len().to_string(),
    );
    let _ = writeln!(
        out,
        "{} Pages project(s), {workers} Worker(s), {targets} URL(s) requested.\n",
        account.pages.len()
    );
    if findings.is_empty() {
        out.push_str("Everything deployed matches its threat model.\n");
        return out;
    }
    out.push_str("| Project | Finding |\n| --- | --- |\n");
    for f in findings {
        let _ = writeln!(
            out,
            "| {} | {} |",
            f.project.as_deref().unwrap_or("(none)"),
            f.message.replace('|', "\\|")
        );
    }
    if !orphans.is_empty() {
        out.push_str("\nPR triage adds these issue files (standards/docs/repo-health.md):\n");
        for o in orphans {
            let _ = write!(
                out,
                "\n`{}`\n\n```markdown\n{}```\n",
                o.issue_path(),
                o.issue_text(today)
            );
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_threat_file;

    const SITE: &str = r#"
reviewed = "2026-11-27"
summary = "x"

[[surface]]
name = "site"
kind = "pages"
url = "https://jimhillr-app.pages.dev"
access = "public"
data = "public"
previews = "public"

[[surface]]
name = "api"
kind = "worker"
url = "https://jimhillr-app-api.jim.workers.dev"
access = "token"
data = "private"

[[surface]]
name = "glob"
kind = "worker"
url = "https://jimhillr-app.pages.dev/api/*"
access = "auth0"
data = "personal"
"#;

    const PRIVATE: &str = r#"
reviewed = "2026-11-27"
summary = "x"

[[surface]]
name = "dash"
kind = "pages"
url = "https://jimhillr-dash.pages.dev"
access = "cloudflare-access"
data = "private"
previews = "cloudflare-access"
"#;

    fn files() -> Vec<(&'static str, ThreatFile)> {
        vec![
            ("app", parse_threat_file(SITE).expect("site")),
            ("dash", parse_threat_file(PRIVATE).expect("dash")),
        ]
    }

    fn account() -> Account {
        Account {
            pages: vec![
                PagesProject {
                    name: "jimhillr-app".into(),
                    domains: vec!["jimhillr-app.pages.dev".into()],
                    preview_url: Some("https://abc.jimhillr-app.pages.dev".into()),
                },
                PagesProject {
                    name: "jimhillr-dash".into(),
                    domains: vec![],
                    preview_url: Some("https://def.jimhillr-dash.pages.dev".into()),
                },
                PagesProject {
                    name: "pane-dbc".into(),
                    domains: vec![],
                    preview_url: None,
                },
            ],
            workers: Some(vec!["jimhillr-app-api".into(), "stray".into()]),
        }
    }

    fn headers() -> BTreeMap<String, String> {
        [
            (
                "content-security-policy",
                "default-src 'self'; frame-ancestors 'none'",
            ),
            ("x-content-type-options", "nosniff"),
            ("referrer-policy", "no-referrer"),
            ("strict-transport-security", "max-age=31536000"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
    }

    fn probe(
        url: &str,
        status: u16,
        location: Option<&str>,
        headers: BTreeMap<String, String>,
    ) -> Probe {
        Probe {
            url: url.into(),
            status,
            location: location.map(str::to_string),
            headers,
        }
    }

    fn good() -> Vec<Probe> {
        let access = Some("https://team.cloudflareaccess.com/cdn-cgi/access/login/x");
        vec![
            probe("https://jimhillr-app.pages.dev", 200, None, headers()),
            probe("https://abc.jimhillr-app.pages.dev", 200, None, headers()),
            probe(
                "https://jimhillr-app-api.jim.workers.dev",
                401,
                None,
                BTreeMap::new(),
            ),
            probe(
                "https://jimhillr-dash.pages.dev",
                302,
                access,
                BTreeMap::new(),
            ),
            probe(
                "https://def.jimhillr-dash.pages.dev",
                302,
                access,
                BTreeMap::new(),
            ),
        ]
    }

    fn run(probes: &[Probe]) -> (Vec<String>, Vec<Orphan>) {
        let files = files();
        let models: Vec<Model> = files
            .iter()
            .map(|(p, f)| Model {
                project: p,
                file: f,
            })
            .collect();
        let (findings, orphans) = check_live(&models, &account(), probes);
        (findings.into_iter().map(|f| f.message).collect(), orphans)
    }

    #[test]
    fn test_targets_skip_globs_and_add_previews() {
        let files = files();
        let models: Vec<Model> = files
            .iter()
            .map(|(p, f)| Model {
                project: p,
                file: f,
            })
            .collect();
        let urls: Vec<(String, Access, bool, bool)> = targets(&models, &account())
            .into_iter()
            .map(|t| (t.url, t.access, t.headers, t.preview))
            .collect();
        assert_eq!(
            urls,
            [
                (
                    "https://jimhillr-app.pages.dev".into(),
                    Access::Public,
                    true,
                    false
                ),
                (
                    "https://abc.jimhillr-app.pages.dev".into(),
                    Access::Public,
                    true,
                    true
                ),
                (
                    "https://jimhillr-app-api.jim.workers.dev".into(),
                    Access::Token,
                    false,
                    false
                ),
                (
                    "https://jimhillr-dash.pages.dev".into(),
                    Access::CloudflareAccess,
                    false,
                    false
                ),
                (
                    "https://def.jimhillr-dash.pages.dev".into(),
                    Access::CloudflareAccess,
                    false,
                    true
                ),
            ]
        );
    }

    #[test]
    fn test_only_unclaimed_deployments_are_orphans() {
        let (findings, orphans) = run(&good());
        assert_eq!(
            findings,
            [
                "Pages project `pane-dbc` is deployed, but no docs/threat.toml has a surface on it",
                "Worker `stray` is deployed, but no docs/threat.toml has a surface on its workers.dev address",
            ]
        );
        assert_eq!(
            orphans.iter().map(Orphan::issue_path).collect::<Vec<_>>(),
            [
                "threat/docs/issues/claim-or-delete-pages-pane-dbc.md",
                "threat/docs/issues/claim-or-delete-worker-stray.md"
            ]
        );
        let text = orphans[0].issue_text("2026-11-30");
        assert!(
            text.starts_with("---\nkind: fix\npriority: priority\n"),
            "{text}"
        );
        assert!(text.contains("added: 2026-11-30\n"), "{text}");
    }

    #[test]
    fn test_wrong_answers_are_findings() {
        let mut probes = good();
        probes[0].headers.remove("strict-transport-security");
        probes[1] = probe(
            "https://abc.jimhillr-app.pages.dev",
            302,
            Some("https://t.cloudflareaccess.com/x"),
            BTreeMap::new(),
        );
        probes[2].status = 200;
        probes[3] = probe("https://jimhillr-dash.pages.dev", 200, None, headers());
        probes.remove(4);
        let (findings, _) = run(&probes);
        assert_eq!(
            &findings[2..],
            [
                "surface `site` at https://jimhillr-app.pages.dev is public but doesn't send Strict-Transport-Security",
                "surface `site` preview https://abc.jimhillr-app.pages.dev says access = \"public\" but redirects to Cloudflare Access; update the threat model",
                "surface `api` at https://jimhillr-app-api.jim.workers.dev should refuse a request without a token (401 or 403), but answered 200",
                "surface `dash` at https://jimhillr-dash.pages.dev should redirect to Cloudflare Access, but answered 200",
                "surface `dash` preview https://def.jimhillr-dash.pages.dev was not requested",
            ]
        );
    }

    #[test]
    fn test_failed_requests_and_errors() {
        let mut probes = good();
        probes[0].status = 0;
        probes[2].status = 500;
        let (findings, _) = run(&probes);
        assert_eq!(
            &findings[2..],
            [
                "surface `site` at https://jimhillr-app.pages.dev didn't answer",
                "surface `api` at https://jimhillr-app-api.jim.workers.dev should refuse a request without a token (401 or 403), but answered 500",
            ]
        );
    }

    #[test]
    fn test_unlisted_workers_are_skipped() {
        let files = files();
        let models: Vec<Model> = files
            .iter()
            .map(|(p, f)| Model {
                project: p,
                file: f,
            })
            .collect();
        let mut account = account();
        account.workers = None;
        let (findings, _) = check_live(&models, &account, &good());
        assert_eq!(findings.len(), 1, "{findings:?}");
        let report = render_live(&account, 5, &findings, &[], "2026-11-30");
        assert!(report.contains("not listed"), "{report}");
    }

    #[test]
    fn test_render() {
        let (_, orphans) = run(&good());
        let findings = vec![LiveFinding {
            project: None,
            message: "a | b".into(),
        }];
        let report = render_live(&account(), 5, &findings, &orphans, "2026-11-30");
        assert!(report.contains("| (none) | a \\| b |"), "{report}");
        assert!(
            report.contains("`threat/docs/issues/claim-or-delete-pages-pane-dbc.md`"),
            "{report}"
        );
        let clean = render_live(&account(), 5, &[], &[], "2026-11-30");
        assert!(clean.contains("Everything deployed matches"), "{clean}");
    }
}
