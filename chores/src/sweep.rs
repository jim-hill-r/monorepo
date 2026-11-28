//! The weekly resource sweep ("Part 3: the weekly sweep" in
//! standards/docs/plans/2026-11-resource-cleanup.md, "Checks" in
//! standards/docs/resources.md).
//!
//! `resources.sh sweep` lists what is live on Cloudflare and GitHub into a
//! JSON file and the claims into a TSV file; this sorts every live resource
//! into one status and writes the report. It reads only files, so the rules
//! are tested without an account.
//!
//! | Status | Meaning | Fails the sweep |
//! | --- | --- | --- |
//! | `claimed` | A file on `main` names it, or it is `main` or an open PR's branch or cache | no |
//! | `kept` | A `[[keep]]` entry whose review date hasn't passed | no |
//! | `keep-overdue` | A `[[keep]]` entry past its `review` date | yes |
//! | `retired-pending` | A `[[retired]]` entry before its `delete_after` date | no |
//! | `retired-still-live` | A `[[retired]]` entry past `delete_after` and still live: the deletion failed | yes |
//! | `gone` | A `[[retired]]` entry with no `deleted` date that is no longer live: set `deleted` | no |
//! | `temporary` | A branch of a closed PR, or a cache of a closed PR or a deleted branch (phase 2 deletes them) | no |
//! | `no-pr` | A branch with no PR, last changed more than 14 days ago: the digest asks Jim | no |
//! | `orphan` | Anything else live | yes |

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// The file with the `[[keep]]` and `[[retired]]` entries, from the root.
pub const RESOURCES: &str = "standards/resources.toml";

/// Days a branch with no PR may sit before the sweep lists it for Jim.
pub const NO_PR_DAYS: i64 = 14;

/// What `resources.sh sweep` read from the live accounts.
#[derive(Debug, Default, Deserialize)]
pub struct Live {
    /// Kinds the tokens couldn't list, with why. They are reported as
    /// warnings, and retired entries of those kinds are left alone.
    #[serde(default)]
    pub unlisted: Vec<Unlisted>,
    #[serde(default)]
    pub resources: Vec<LiveResource>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Unlisted {
    pub kind: String,
    pub why: String,
}

/// One live resource.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LiveResource {
    pub kind: String,
    pub name: String,
    /// Branches and caches: the PR behind it, `open`, `closed` or `none`.
    #[serde(default)]
    pub pr: Option<String>,
    /// Branches: the date of the newest commit, YYYY-MM-DD.
    #[serde(default)]
    pub updated: Option<String>,
    /// Anything worth showing in the report, such as a deployment count.
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Entries {
    #[serde(default)]
    keep: Vec<Entry>,
    #[serde(default)]
    retired: Vec<Entry>,
}

#[derive(Debug, Default, Deserialize)]
struct Entry {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    review: String,
    #[serde(default)]
    delete_after: String,
    #[serde(default)]
    deleted: String,
}

/// What the sweep compares the live resources with.
#[derive(Debug, Default)]
pub struct Claims {
    /// `(kind, name)` from `resources.sh claims`.
    pub claimed: BTreeSet<(String, String)>,
    /// Host names of every `[[surface]] url` in a `docs/threat.toml`, which
    /// claim DNS records.
    pub hosts: BTreeSet<String>,
    keep: Vec<Entry>,
    retired: Vec<Entry>,
}

/// One row of the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Row {
    pub kind: String,
    pub name: String,
    pub status: &'static str,
    pub note: String,
}

/// The sweep's result, written as the JSON report.
#[derive(Debug, Default, Serialize)]
pub struct Report {
    pub today: String,
    pub rows: Vec<Row>,
    pub unlisted: Vec<Unlisted>,
}

impl Report {
    /// Rows that fail the sweep: orphans, overdue keeps and failed deletions.
    pub fn failures(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(|r| fails(r.status))
    }
}

fn fails(status: &str) -> bool {
    matches!(status, "orphan" | "keep-overdue" | "retired-still-live")
}

/// Reads the claims TSV, the entries and the threat models' hosts.
pub fn load_claims(root: &Path, claims_tsv: &str) -> Result<Claims> {
    let mut claimed = BTreeSet::new();
    for line in claims_tsv.lines().filter(|l| !l.trim().is_empty()) {
        let Some((kind, name)) = line.split_once('\t') else {
            bail!("claims line {line:?} is not `kind<TAB>name`");
        };
        claimed.insert((kind.trim().to_string(), name.trim().to_string()));
    }
    let path = root.join(RESOURCES);
    let entries: Entries = if path.is_file() {
        toml::from_str(&std::fs::read_to_string(&path)?)
            .with_context(|| format!("reading {RESOURCES}"))?
    } else {
        Entries::default()
    };
    let mut hosts = BTreeSet::new();
    for file in threat_models(root)? {
        // A threat model that doesn't parse claims nothing; the threat
        // check fails it, so the sweep only warns rather than stopping.
        let text = std::fs::read_to_string(root.join(&file))?;
        match surface_hosts(&text) {
            Ok(h) => hosts.extend(h),
            Err(e) => {
                eprintln!("warning: {file} is not valid TOML, so it claims no DNS records: {e}")
            }
        }
    }
    Ok(Claims {
        claimed,
        hosts,
        keep: entries.keep,
        retired: entries.retired,
    })
}

fn threat_models(root: &Path) -> Result<Vec<String>> {
    let out = Command::new("git")
        .args(["ls-files", "--", "*docs/threat.toml"])
        .current_dir(root)
        .output()
        .context("running git ls-files")?;
    if !out.status.success() {
        bail!("git ls-files failed in {}", root.display());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|p| root.join(p).is_file())
        .map(str::to_string)
        .collect())
}

/// The host of each `[[surface]] url` in a threat model.
pub fn surface_hosts(threat_toml: &str) -> Result<Vec<String>> {
    #[derive(Deserialize)]
    struct Threat {
        #[serde(default)]
        surface: Vec<Surface>,
    }
    #[derive(Deserialize)]
    struct Surface {
        #[serde(default)]
        url: String,
    }
    let threat: Threat = toml::from_str(threat_toml)?;
    Ok(threat.surface.iter().filter_map(|s| host(&s.url)).collect())
}

fn host(url: &str) -> Option<String> {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    let host = rest.split(['/', '?', '#']).next()?;
    let host = host.rsplit_once('@').map_or(host, |(_, h)| h);
    let host = host.split(':').next()?.trim().to_lowercase();
    (!host.is_empty()).then_some(host)
}

/// Sorts every live resource into a status, and adds a `gone` row for each
/// retired entry of a listed kind that is no longer live.
pub fn sweep(live: &Live, claims: &Claims, today: &str) -> Report {
    let unlisted: BTreeSet<&str> = live.unlisted.iter().map(|u| u.kind.as_str()).collect();
    let mut rows: Vec<Row> = live
        .resources
        .iter()
        .map(|r| classify(r, claims, today))
        .collect();
    let seen: BTreeSet<(&str, &str)> = live
        .resources
        .iter()
        .map(|r| (r.kind.as_str(), r.name.as_str()))
        .collect();
    for e in &claims.retired {
        if e.deleted.is_empty()
            && is_sweep_kind(&e.kind)
            && !unlisted.contains(e.kind.as_str())
            && !seen.contains(&(e.kind.as_str(), e.name.as_str()))
        {
            rows.push(Row {
                kind: e.kind.clone(),
                name: e.name.clone(),
                status: "gone",
                note: format!("no longer live: set `deleted = \"{today}\"` in {RESOURCES}"),
            });
        }
    }
    rows.sort_by(|a, b| {
        (!fails(a.status), a.kind.as_str(), a.name.as_str()).cmp(&(
            !fails(b.status),
            b.kind.as_str(),
            b.name.as_str(),
        ))
    });
    Report {
        today: today.to_string(),
        rows,
        unlisted: live.unlisted.clone(),
    }
}

/// The kinds `resources.sh sweep` lists. A retired entry of any other kind
/// (an Auth0 URL, R2) is never reported `gone`, because nothing looked.
pub const SWEEP_KINDS: [&str; 10] = [
    "pages", "worker", "d1", "kv", "queue", "cron", "dns", "branch", "release", "cache",
];

fn is_sweep_kind(kind: &str) -> bool {
    SWEEP_KINDS.contains(&kind)
}

fn row(r: &LiveResource, status: &'static str, note: impl Into<String>) -> Row {
    let mut note = note.into();
    if let Some(d) = r.detail.as_deref().filter(|d| !d.is_empty()) {
        if note.is_empty() {
            note = d.to_string();
        } else {
            let _ = write!(note, "; {d}");
        }
    }
    Row {
        kind: r.kind.clone(),
        name: r.name.clone(),
        status,
        note,
    }
}

fn classify(r: &LiveResource, claims: &Claims, today: &str) -> Row {
    if let Some(claim) = claimed_by(r, claims) {
        return row(r, "claimed", claim);
    }
    if let Some(e) = claims
        .keep
        .iter()
        .find(|e| e.kind == r.kind && e.name == r.name)
    {
        return if e.review.as_str() < today {
            row(
                r,
                "keep-overdue",
                format!(
                    "review date {} has passed: look at it again and move the date, or retire it",
                    e.review
                ),
            )
        } else {
            row(r, "kept", format!("review {}", e.review))
        };
    }
    if let Some(e) = claims
        .retired
        .iter()
        .find(|e| e.kind == r.kind && e.name == r.name)
    {
        return if e.delete_after.as_str() < today {
            row(
                r,
                "retired-still-live",
                format!(
                    "delete_after {} has passed and it is still live: the deletion failed",
                    e.delete_after
                ),
            )
        } else {
            row(
                r,
                "retired-pending",
                format!("deleted after {}", e.delete_after),
            )
        };
    }
    match r.kind.as_str() {
        "branch" => match r.pr.as_deref() {
            Some("closed") => row(
                r,
                "temporary",
                "its PR is closed; deleted 7 days after it closed",
            ),
            _ => match r.updated.as_deref().and_then(|u| days_between(u, today)) {
                Some(age) if age <= NO_PR_DAYS => row(
                    r,
                    "claimed",
                    format!("no PR yet, last changed {age} days ago"),
                ),
                Some(age) => row(
                    r,
                    "no-pr",
                    format!("no PR, last changed {age} days ago: the digest asks Jim"),
                ),
                None => row(r, "no-pr", "no PR: the digest asks Jim"),
            },
        },
        "cache" => row(
            r,
            "temporary",
            "its PR or branch is gone; deleted when the PR closes",
        ),
        _ => row(
            r,
            "orphan",
            "no file claims it: open a retirement PR (or a keep entry) for Jim",
        ),
    }
}

fn claimed_by(r: &LiveResource, claims: &Claims) -> Option<String> {
    if claims.claimed.contains(&(r.kind.clone(), r.name.clone())) {
        return Some(String::new());
    }
    match r.kind.as_str() {
        "branch" | "cache" if r.pr.as_deref() == Some("open") => Some("open PR".to_string()),
        "branch" if r.name == "main" => Some("default branch".to_string()),
        "cache" if r.pr.as_deref() == Some("main") => Some("main".to_string()),
        "dns" if claims.hosts.contains(&r.name.to_lowercase()) => {
            Some("a threat.toml surface url".to_string())
        }
        // wrangler names a KV namespace `<worker>-<binding>`; the claim is
        // `<worker>/<binding>`.
        "kv" => claims
            .claimed
            .iter()
            .any(|(k, n)| k == "kv" && n.replace('/', "-") == r.name)
            .then(String::new),
        _ => None,
    }
}

/// Whole days from `from` to `to`, both YYYY-MM-DD.
pub fn days_between(from: &str, to: &str) -> Option<i64> {
    Some(days(to)? - days(from)?)
}

/// Days since 1970-01-01 (Howard Hinnant's days-from-civil).
fn days(date: &str) -> Option<i64> {
    let mut parts = date.get(..10)?.split('-');
    let y: i64 = parts.next()?.parse().ok()?;
    let m: i64 = parts.next()?.parse().ok()?;
    let d: i64 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// The report as Markdown, for the run summary: failures first, then a
/// count per status and the full table.
pub fn render(report: &Report) -> String {
    let mut out = String::from("## Resource sweep\n\n");
    let failures: Vec<&Row> = report.failures().collect();
    if failures.is_empty() {
        out.push_str("No orphans, overdue keeps or failed deletions.\n\n");
    } else {
        let _ = writeln!(
            out,
            "{} finding(s) to act on (standards/docs/resources.md):\n",
            failures.len()
        );
        for r in &failures {
            let _ = writeln!(
                out,
                "- **{}** {} `{}`: {}",
                r.status, r.kind, r.name, r.note
            );
        }
        out.push('\n');
    }
    for u in &report.unlisted {
        let _ = writeln!(out, "- warning: {} not listed: {}", u.kind, u.why);
    }
    if !report.unlisted.is_empty() {
        out.push('\n');
    }
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for r in &report.rows {
        *counts.entry(r.status).or_default() += 1;
    }
    let summary: Vec<String> = counts.iter().map(|(s, n)| format!("{n} {s}")).collect();
    if summary.is_empty() {
        out.push_str("Nothing listed.\n");
        return out;
    }
    let _ = writeln!(
        out,
        "{} live or retired: {}.\n",
        report.rows.len(),
        summary.join(", ")
    );
    out.push_str("| Status | Kind | Name | Note |\n| --- | --- | --- | --- |\n");
    for r in &report.rows {
        let _ = writeln!(
            out,
            "| {} | {} | `{}` | {} |",
            r.status,
            r.kind,
            r.name,
            r.note.replace('|', "\\|")
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live(kind: &str, name: &str) -> LiveResource {
        LiveResource {
            kind: kind.into(),
            name: name.into(),
            ..LiveResource::default()
        }
    }

    fn claims(tsv: &str, entries: &str, hosts: &[&str]) -> Claims {
        let e: Entries = toml::from_str(entries).unwrap();
        Claims {
            claimed: tsv
                .lines()
                .filter_map(|l| l.split_once('\t'))
                .map(|(k, n)| (k.to_string(), n.to_string()))
                .collect(),
            hosts: hosts.iter().map(|h| (*h).to_string()).collect(),
            keep: e.keep,
            retired: e.retired,
        }
    }

    fn status(report: &Report, kind: &str, name: &str) -> &'static str {
        report
            .rows
            .iter()
            .find(|r| r.kind == kind && r.name == name)
            .map_or("missing", |r| r.status)
    }

    const ENTRIES: &str = r#"
[[keep]]
kind = "pages"
name = "landing"
why = "Jim's"
review = "2027-01-01"

[[keep]]
kind = "worker"
name = "old-keep"
why = "Jim's"
review = "2026-11-01"

[[retired]]
kind = "pages"
name = "cahokia-web"
why = "renamed"
retired = "2026-11-27"
delete_after = "2026-12-04"
deleted = ""

[[retired]]
kind = "pages"
name = "pane"
why = "renamed"
retired = "2026-11-01"
delete_after = "2026-11-08"
deleted = ""

[[retired]]
kind = "pages"
name = "controlled-chaos"
why = "renamed"
retired = "2026-11-01"
delete_after = "2026-11-08"
deleted = ""

[[retired]]
kind = "pages"
name = "long-gone"
why = "renamed"
retired = "2026-10-01"
delete_after = "2026-10-08"
deleted = "2026-10-10"

[[retired]]
kind = "auth0-url"
name = "https://old.example/callback"
why = "renamed"
retired = "2026-11-01"
delete_after = "2026-11-08"
deleted = ""
"#;

    #[test]
    fn sorts_every_kind_of_status() {
        let live = Live {
            unlisted: vec![],
            resources: vec![
                live("pages", "jimhillr-cookbook"),
                live("pages", "landing"),
                live("worker", "old-keep"),
                live("pages", "cahokia-web"),
                live("pages", "pane"),
                live("pages", "stray"),
            ],
        };
        let c = claims("pages\tjimhillr-cookbook\n", ENTRIES, &[]);
        let report = sweep(&live, &c, "2026-11-30");
        assert_eq!(status(&report, "pages", "jimhillr-cookbook"), "claimed");
        assert_eq!(status(&report, "pages", "landing"), "kept");
        assert_eq!(status(&report, "worker", "old-keep"), "keep-overdue");
        assert_eq!(status(&report, "pages", "cahokia-web"), "retired-pending");
        assert_eq!(status(&report, "pages", "pane"), "retired-still-live");
        assert_eq!(status(&report, "pages", "stray"), "orphan");
        // Retired, not live and not yet marked deleted.
        assert_eq!(status(&report, "pages", "controlled-chaos"), "gone");
        // Already marked deleted: not reported again.
        assert_eq!(status(&report, "pages", "long-gone"), "missing");
        // Nothing lists Auth0 URLs yet, so it is never called gone.
        assert_eq!(
            status(&report, "auth0-url", "https://old.example/callback"),
            "missing"
        );
        let failing: Vec<&str> = report.failures().map(|r| r.name.as_str()).collect();
        assert_eq!(failing, ["pane", "stray", "old-keep"]);
    }

    #[test]
    fn a_claim_wins_over_an_entry() {
        let live = Live {
            resources: vec![live("pages", "pane")],
            ..Live::default()
        };
        let c = claims("pages\tpane\n", ENTRIES, &[]);
        assert_eq!(
            status(&sweep(&live, &c, "2026-11-30"), "pages", "pane"),
            "claimed"
        );
    }

    #[test]
    fn the_review_and_delete_after_days_themselves_pass() {
        let live = Live {
            resources: vec![live("worker", "old-keep"), live("pages", "cahokia-web")],
            ..Live::default()
        };
        let c = claims("", ENTRIES, &[]);
        let r = sweep(&live, &c, "2026-11-01");
        assert_eq!(status(&r, "worker", "old-keep"), "kept");
        let r = sweep(&live, &c, "2026-12-04");
        assert_eq!(status(&r, "pages", "cahokia-web"), "retired-pending");
        let r = sweep(&live, &c, "2026-12-05");
        assert_eq!(status(&r, "pages", "cahokia-web"), "retired-still-live");
    }

    #[test]
    fn an_unlisted_kind_is_never_gone() {
        let live = Live {
            unlisted: vec![Unlisted {
                kind: "pages".into(),
                why: "token can't read Pages".into(),
            }],
            resources: vec![],
        };
        let r = sweep(&live, &claims("", ENTRIES, &[]), "2026-11-30");
        assert!(r.rows.is_empty(), "{:?}", r.rows);
        assert!(render(&r).contains("warning: pages not listed: token can't read Pages"));
        assert!(render(&r).contains("Nothing listed."));
    }

    #[test]
    fn branches_follow_their_pr() {
        let branch = |name: &str, pr: Option<&str>, updated: Option<&str>| LiveResource {
            kind: "branch".into(),
            name: name.into(),
            pr: pr.map(str::to_string),
            updated: updated.map(str::to_string),
            detail: None,
        };
        let live = Live {
            resources: vec![
                branch("main", Some("none"), None),
                branch("claude/open", Some("open"), None),
                branch("claude/merged", Some("closed"), None),
                branch("claude/new", Some("none"), Some("2026-11-20")),
                branch("claude/stale", Some("none"), Some("2026-11-01")),
            ],
            ..Live::default()
        };
        let r = sweep(&live, &claims("", "", &[]), "2026-11-30");
        assert_eq!(status(&r, "branch", "main"), "claimed");
        assert_eq!(status(&r, "branch", "claude/open"), "claimed");
        assert_eq!(status(&r, "branch", "claude/merged"), "temporary");
        assert_eq!(status(&r, "branch", "claude/new"), "claimed");
        assert_eq!(status(&r, "branch", "claude/stale"), "no-pr");
        assert_eq!(r.failures().count(), 0, "branches never fail the sweep");
    }

    #[test]
    fn caches_follow_their_ref() {
        let cache = |name: &str, pr: &str| LiveResource {
            kind: "cache".into(),
            name: name.into(),
            pr: Some(pr.into()),
            ..LiveResource::default()
        };
        let live = Live {
            resources: vec![
                cache("main-key", "main"),
                cache("open-key", "open"),
                cache("closed-key", "closed"),
            ],
            ..Live::default()
        };
        let r = sweep(&live, &claims("", "", &[]), "2026-11-30");
        assert_eq!(status(&r, "cache", "main-key"), "claimed");
        assert_eq!(status(&r, "cache", "open-key"), "claimed");
        assert_eq!(status(&r, "cache", "closed-key"), "temporary");
        assert_eq!(r.failures().count(), 0);
    }

    #[test]
    fn dns_records_are_claimed_by_threat_model_hosts() {
        let live = Live {
            resources: vec![
                live("dns", "Cook.Example.com"),
                live("dns", "stray.example.com"),
            ],
            ..Live::default()
        };
        let r = sweep(&live, &claims("", "", &["cook.example.com"]), "2026-11-30");
        assert_eq!(status(&r, "dns", "Cook.Example.com"), "claimed");
        assert_eq!(status(&r, "dns", "stray.example.com"), "orphan");
    }

    #[test]
    fn kv_namespaces_match_the_wrangler_title() {
        let live = Live {
            resources: vec![
                live("kv", "jimhillr-pane-FLAGS"),
                live("kv", "jimhillr-pane/FLAGS"),
            ],
            ..Live::default()
        };
        let r = sweep(
            &live,
            &claims("kv\tjimhillr-pane/FLAGS\n", "", &[]),
            "2026-11-30",
        );
        assert_eq!(status(&r, "kv", "jimhillr-pane-FLAGS"), "claimed");
        assert_eq!(status(&r, "kv", "jimhillr-pane/FLAGS"), "claimed");
    }

    #[test]
    fn surface_hosts_reads_each_url() {
        let hosts = surface_hosts(
            r#"
[[surface]]
name = "site"
url = "https://jimhillr-cookbook.pages.dev/recipes?x=1"

[[surface]]
name = "api"
url = "https://user@API.example.com:8443"

[[surface]]
name = "cli"
"#,
        )
        .unwrap();
        assert_eq!(hosts, ["jimhillr-cookbook.pages.dev", "api.example.com"]);
    }

    #[test]
    fn days_between_counts_across_months_and_years() {
        assert_eq!(days_between("2026-11-30", "2026-12-01"), Some(1));
        assert_eq!(days_between("2026-12-31", "2027-01-01"), Some(1));
        assert_eq!(days_between("2028-02-28", "2028-03-01"), Some(2));
        assert_eq!(days_between("2026-11-14T10:00:00Z", "2026-11-28"), Some(14));
        assert_eq!(days_between("soon", "2026-11-28"), None);
    }

    #[test]
    fn render_puts_findings_first_and_escapes_pipes() {
        let live = Live {
            resources: vec![
                live("pages", "jimhillr-cookbook"),
                LiveResource {
                    detail: Some("2 | deployments".into()),
                    ..live("pages", "stray")
                },
            ],
            ..Live::default()
        };
        let r = sweep(
            &live,
            &claims("pages\tjimhillr-cookbook\n", "", &[]),
            "2026-11-30",
        );
        let md = render(&r);
        assert!(md.contains("1 finding(s) to act on"), "{md}");
        assert!(md.contains("- **orphan** pages `stray`"), "{md}");
        assert!(
            md.contains("2 live or retired: 1 claimed, 1 orphan."),
            "{md}"
        );
        assert!(md.contains("2 \\| deployments"), "{md}");
        let first = md.find("| orphan |").unwrap();
        assert!(first < md.find("| claimed |").unwrap(), "{md}");
    }

    #[test]
    fn load_claims_reads_the_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("standards")).unwrap();
        std::fs::create_dir_all(root.join("cookbook/docs")).unwrap();
        std::fs::write(root.join(RESOURCES), ENTRIES).unwrap();
        std::fs::write(
            root.join("cookbook/docs/threat.toml"),
            "[[surface]]\nname = \"site\"\nurl = \"https://jimhillr-cookbook.pages.dev\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("broken/docs")).unwrap();
        std::fs::write(root.join("broken/docs/threat.toml"), "<<<<<<< HEAD\n").unwrap();
        let git = |args: &[&str]| {
            let ok = Command::new("git")
                .args(args)
                .current_dir(root)
                .env_remove("GIT_DIR")
                .env_remove("GIT_INDEX_FILE")
                .output()
                .unwrap()
                .status
                .success();
            assert!(ok, "git {args:?}");
        };
        git(&["init", "-q"]);
        git(&["add", "-A"]);
        let c = load_claims(root, "pages\tjimhillr-cookbook\n\n").unwrap();
        assert!(
            c.claimed
                .contains(&("pages".into(), "jimhillr-cookbook".into()))
        );
        assert!(c.hosts.contains("jimhillr-cookbook.pages.dev"));
        assert_eq!(c.keep.len(), 2);
        assert_eq!(c.retired.len(), 5);
        assert!(load_claims(root, "no tab here").is_err());
    }
}
