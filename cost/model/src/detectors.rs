//! `standards/radar/detectors.toml`: which patterns in a project's files
//! mean it uses an outside service, and the rules that compare that with
//! its `docs/cost.toml` (COST-007, and COST-009 for unknown hosts). The
//! threat check reuses the same matches through each detector's `threat`
//! field (THR-004), so a project's files are read by one engine.
//!
//! The detectors run over the project's whole tree, not a diff, so a hook,
//! PR CI and `main` all get the same answer.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::LazyLock;

use regex::Regex;
use serde::Deserialize;

use crate::{COST_FILE, Code, CostFile, Finding};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DetectorsFile {
    pub scan: Scan,
    #[serde(default)]
    pub detector: Vec<Detector>,
    /// Outside hosts that cost nothing (docs sites, registries, CDNs).
    #[serde(default)]
    pub allow: Vec<Allow>,
    /// A file a detector must not count, such as a guard that searches for
    /// the very pattern it matches.
    #[serde(default)]
    pub exempt: Vec<Exempt>,
}

/// Which of a project's files the line detectors read.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scan {
    /// File extensions, without the dot. `wrangler.toml` is always read.
    pub extensions: Vec<String>,
    /// Directory names skipped at any depth (`docs`, `target`, ...).
    pub skip_dirs: Vec<String>,
    /// File names never read (`Cargo.lock`, ...).
    pub skip_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Detector {
    /// Short name, used in findings and in `[[exempt]]`.
    pub id: String,
    /// The vendor blip in `radar.json` a match implies. A detector with
    /// only a `threat` has none, and the cost check ignores it.
    #[serde(default)]
    pub blip: Option<String>,
    /// What a match implies for the project's `docs/threat.toml` (THR-004).
    #[serde(default)]
    pub threat: Option<ThreatHint>,
    pub why: String,
    /// URL hosts (and their subdomains) in source or config.
    #[serde(default)]
    pub hosts: Vec<String>,
    /// Regexes matched against each non-comment line.
    #[serde(default)]
    pub patterns: Vec<String>,
    /// Only match `patterns` in files with these names (`Cargo.toml`).
    /// Empty means every scanned file.
    #[serde(default)]
    pub files: Vec<String>,
    /// A dotted key in a `wrangler.toml`. An array counts its entries,
    /// anything else counts 1.
    #[serde(default)]
    pub wrangler: Option<String>,
    /// A workflow in `.github/workflows/` named after the project
    /// (`<project>.yml` or `<project>-*.yml`) with a `schedule:`.
    #[serde(default)]
    pub workflow_schedule: bool,
    /// A `limits.toml` key that each match adds its count to.
    #[serde(default)]
    pub usage: Option<String>,
}

/// What a detector match tells the threat check (THR-004 in
/// `standards/docs/security.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThreatHint {
    /// A Pages site: the file needs a `kind = "pages"` surface.
    Pages,
    /// A Worker: the file needs a `worker` or `api` surface.
    Worker,
    /// A deployed URL, the pattern's first capture group: some surface's
    /// `url` must be on the same host.
    Url,
    /// Stored data (D1, KV, R2): some surface's `data` isn't `public`, or a
    /// `[[threat]]` covers the detector.
    Storage,
    /// A secret, named by the pattern's first capture group: the file needs
    /// a `[[secret]]` with that name.
    Secret,
    /// A sign-in library or tenant: the file needs an `[auth]` table.
    Auth,
    /// A risky construct (CORS open to any origin, raw HTML, `eval`): a
    /// `[[threat]]` must cover the detector by id.
    Risk,
}

impl ThreatHint {
    pub fn as_str(self) -> &'static str {
        match self {
            ThreatHint::Pages => "pages",
            ThreatHint::Worker => "worker",
            ThreatHint::Url => "url",
            ThreatHint::Storage => "storage",
            ThreatHint::Secret => "secret",
            ThreatHint::Auth => "auth",
            ThreatHint::Risk => "risk",
        }
    }

    /// Whether the hint needs the name a pattern captures.
    pub fn captures(self) -> bool {
        matches!(self, ThreatHint::Url | ThreatHint::Secret)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Allow {
    /// A host; its subdomains are allowed too.
    pub host: String,
    pub why: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exempt {
    /// A repo-relative file, or a directory ending in `/`.
    pub path: String,
    /// A detector id, or `unknown-host`.
    pub detector: String,
    pub why: String,
}

/// The parsed file with its regexes compiled.
#[derive(Debug, Clone)]
pub struct Detectors {
    pub file: DetectorsFile,
    patterns: Vec<Vec<Regex>>,
}

/// The id `[[exempt]]` uses for COST-009.
pub const UNKNOWN_HOST: &str = "unknown-host";

/// Parses `detectors.toml`, compiles its regexes and checks that each
/// detector has exactly one kind of match and a unique id.
pub fn parse_detectors(text: &str) -> Result<Detectors, String> {
    let file: DetectorsFile = toml::from_str(text).map_err(|e| e.message().to_string())?;
    let mut ids = BTreeSet::new();
    let mut patterns = Vec::new();
    for d in &file.detector {
        if !ids.insert(d.id.as_str()) || d.id == UNKNOWN_HOST {
            return Err(format!("detector id `{}` is used twice or reserved", d.id));
        }
        let kinds = [
            !d.hosts.is_empty() || !d.patterns.is_empty(),
            d.wrangler.is_some(),
            d.workflow_schedule,
        ];
        if kinds.iter().filter(|k| **k).count() != 1 {
            return Err(format!(
                "detector `{}` needs exactly one of hosts/patterns, wrangler or workflow_schedule",
                d.id
            ));
        }
        if d.blip.is_none() && d.threat.is_none() {
            return Err(format!(
                "detector `{}` needs a blip, a threat or both",
                d.id
            ));
        }
        if d.blip.is_none() && d.usage.is_some() {
            return Err(format!("detector `{}` counts usage but has no blip", d.id));
        }
        let compiled = d
            .patterns
            .iter()
            .map(|p| Regex::new(p).map_err(|e| format!("detector `{}`: {e}", d.id)))
            .collect::<Result<Vec<_>, _>>()?;
        if d.threat.is_some_and(ThreatHint::captures)
            && (compiled.is_empty()
                || !d.hosts.is_empty()
                || compiled.iter().any(|r| r.captures_len() < 2))
        {
            return Err(format!(
                "detector `{}`: a `{}` threat needs patterns only, each with a capture group for the name",
                d.id,
                d.threat.map_or("", ThreatHint::as_str)
            ));
        }
        patterns.push(compiled);
    }
    Ok(Detectors { file, patterns })
}

/// One file's repo-relative path and text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: String,
    pub text: String,
}

/// One detector match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub detector: String,
    /// The vendor blip, for the cost check. `None` for a threat-only
    /// detector.
    pub blip: Option<String>,
    /// What the match implies for the threat model.
    pub threat: Option<ThreatHint>,
    /// The first capture group of the pattern that matched (a secret's
    /// name, a URL), when it has one.
    pub name: Option<String>,
    /// `path:line`, or just the path for wrangler and workflow matches.
    pub at: String,
    /// The limit key and how much this match adds to it.
    pub usage: Option<(String, u64)>,
}

/// An outside host no detector or allow entry knows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownHost {
    pub host: String,
    pub path: String,
    pub at: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Detection {
    pub hits: Vec<Hit>,
    pub unknown_hosts: Vec<UnknownHost>,
}

impl Detection {
    /// The blips the project's files show it uses.
    pub fn blips(&self) -> BTreeSet<String> {
        self.hits.iter().filter_map(|h| h.blip.clone()).collect()
    }
}

// Constant patterns, covered by the tests.
#[allow(clippy::expect_used)]
static URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:https?|wss?)://([a-z0-9](?:[a-z0-9-]*[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]*[a-z0-9])?)+)")
        .expect("valid regex")
});

/// Text just before a URL that makes it a link a person follows (an HTML
/// or rsx `href`, a Markdown link), not a service the code calls.
#[allow(clippy::expect_used)]
static LINK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)(href\s*[:=]\s*["']?|\]\()$"#).expect("valid regex"));

#[allow(clippy::expect_used)]
static SCHEDULE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\s*schedule:").expect("valid regex"));

fn host_matches(host: &str, entry: &str) -> bool {
    let entry = entry.to_ascii_lowercase();
    host == entry || host.ends_with(&format!(".{entry}"))
}

/// A line that is only a comment in the languages we scan, so links in doc
/// comments and commented-out code don't count.
fn is_comment(line: &str) -> bool {
    let t = line.trim_start();
    ["//", "#", "/*", "*", "<!--", "--"]
        .iter()
        .any(|p| t.starts_with(p))
        && !t.starts_with("#[")
        && !t.starts_with("#!")
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

impl Detectors {
    /// Whether the line detectors read this repo-relative path.
    pub fn scans(&self, path: &str) -> bool {
        let scan = &self.file.scan;
        let mut parts: Vec<&str> = path.split('/').collect();
        let name = parts.pop().unwrap_or_default();
        if parts.iter().any(|p| scan.skip_dirs.iter().any(|d| d == p)) {
            return false;
        }
        if name == "wrangler.toml" {
            return true;
        }
        if scan.skip_files.iter().any(|f| f == name) {
            return false;
        }
        name.rsplit_once('.')
            .is_some_and(|(_, ext)| scan.extensions.iter().any(|e| e == ext))
    }

    fn exempt(&self, path: &str, detector: &str) -> bool {
        self.file.exempt.iter().any(|e| {
            (e.detector == detector || e.detector == "*")
                && (e.path == path || (e.path.ends_with('/') && path.starts_with(&e.path)))
        })
    }

    fn known_host(&self, host: &str) -> bool {
        if host
            .rsplit('.')
            .next()
            .is_some_and(|tld| tld.chars().all(|c| c.is_ascii_digit()))
        {
            return true; // an IP address
        }
        self.file.allow.iter().any(|a| host_matches(host, &a.host))
            || self
                .file
                .detector
                .iter()
                .any(|d| d.hosts.iter().any(|h| host_matches(host, h)))
    }

    /// Whether any host or pattern detector matches `line` of `path`, for
    /// spotting a manifest line that names a vendor's SDK in a diff.
    pub fn line_matches(&self, path: &str, line: &str) -> bool {
        if is_comment(line) {
            return false;
        }
        let name = file_name(path);
        let hosts: Vec<String> = URL
            .captures_iter(line)
            .map(|c| c[1].to_ascii_lowercase())
            .collect();
        self.file
            .detector
            .iter()
            .zip(&self.patterns)
            .any(|(d, regexes)| {
                d.blip.is_some()
                    && !self.exempt(path, &d.id)
                    && (hosts
                        .iter()
                        .any(|h| d.hosts.iter().any(|e| host_matches(h, e)))
                        || ((d.files.is_empty() || d.files.iter().any(|f| f == name))
                            && regexes.iter().any(|r| r.is_match(line))))
            })
    }

    /// Runs every detector over a project's `files` (paths under the
    /// project) and the repo's `workflows` (paths under
    /// `.github/workflows/`).
    pub fn detect(
        &self,
        project: &str,
        files: &[SourceFile],
        workflows: &[SourceFile],
    ) -> Detection {
        let mut out = Detection::default();
        let mut seen_hosts = BTreeSet::new();
        for file in files.iter().filter(|f| self.scans(&f.path)) {
            let name = file_name(&file.path);
            if name == "wrangler.toml" {
                self.detect_wrangler(file, &mut out);
            }
            for (n, line) in file.text.lines().enumerate() {
                if is_comment(line) {
                    continue;
                }
                let at = format!("{}:{}", file.path, n + 1);
                let urls: Vec<(String, bool)> = URL
                    .captures_iter(line)
                    .map(|c| {
                        let start = c.get(0).map_or(0, |m| m.start());
                        (c[1].to_ascii_lowercase(), LINK.is_match(&line[..start]))
                    })
                    .collect();
                let hosts: Vec<String> = urls.iter().map(|(h, _)| h.clone()).collect();
                for (d, regexes) in self.file.detector.iter().zip(&self.patterns) {
                    if self.exempt(&file.path, &d.id) {
                        continue;
                    }
                    let by_host = hosts
                        .iter()
                        .any(|h| d.hosts.iter().any(|e| host_matches(h, e)));
                    let in_file = d.files.is_empty() || d.files.iter().any(|f| f == name);
                    let captured: Option<Option<String>> = if in_file {
                        regexes.iter().find_map(|r| {
                            r.captures(line)
                                .map(|c| c.get(1).map(|m| m.as_str().to_string()))
                        })
                    } else {
                        None
                    };
                    if by_host || captured.is_some() {
                        out.hits.push(Hit {
                            detector: d.id.clone(),
                            blip: d.blip.clone(),
                            threat: d.threat,
                            name: captured.flatten(),
                            at: at.clone(),
                            usage: d.usage.clone().map(|k| (k, 1)),
                        });
                    }
                }
                for (host, link) in urls {
                    if !link
                        && !self.known_host(&host)
                        && !self.exempt(&file.path, UNKNOWN_HOST)
                        && seen_hosts.insert(host.clone())
                    {
                        out.unknown_hosts.push(UnknownHost {
                            host,
                            path: file.path.clone(),
                            at: at.clone(),
                        });
                    }
                }
            }
        }
        for wf in workflows {
            let stem = file_name(&wf.path)
                .trim_end_matches(".yml")
                .trim_end_matches(".yaml");
            if stem != project && !stem.starts_with(&format!("{project}-")) {
                continue;
            }
            let scheduled = wf
                .text
                .lines()
                .any(|l| !is_comment(l) && SCHEDULE.is_match(l));
            if !scheduled {
                continue;
            }
            for d in self.file.detector.iter().filter(|d| d.workflow_schedule) {
                if !self.exempt(&wf.path, &d.id) {
                    out.hits.push(Hit {
                        detector: d.id.clone(),
                        blip: d.blip.clone(),
                        threat: d.threat,
                        name: None,
                        at: wf.path.clone(),
                        usage: d.usage.clone().map(|k| (k, 1)),
                    });
                }
            }
        }
        out
    }

    fn detect_wrangler(&self, file: &SourceFile, out: &mut Detection) {
        let Ok(value) = file.text.parse::<toml::Table>() else {
            return; // a broken wrangler.toml fails the project's own build
        };
        for d in &self.file.detector {
            let Some(key) = &d.wrangler else { continue };
            if self.exempt(&file.path, &d.id) {
                continue;
            }
            let parts: Vec<&str> = key.split('.').collect();
            let mut table = &value;
            let mut found: Option<&toml::Value> = None;
            for (i, part) in parts.iter().enumerate() {
                match table.get(*part) {
                    Some(v) if i + 1 == parts.len() => found = Some(v),
                    Some(toml::Value::Table(t)) => table = t,
                    _ => break,
                }
            }
            let count = match found {
                None => 0,
                Some(toml::Value::Array(items)) => items.len() as u64,
                Some(_) => 1,
            };
            if count > 0 {
                out.hits.push(Hit {
                    detector: d.id.clone(),
                    blip: d.blip.clone(),
                    threat: d.threat,
                    name: None,
                    at: file.path.clone(),
                    usage: d.usage.clone().map(|k| (k, count)),
                });
            }
        }
    }
}

/// COST-007 (a detected service or counted usage the file doesn't list) and
/// COST-009 (an unknown outside host) for one project.
pub fn check_drift(project: &str, file: &CostFile, detection: &Detection) -> Vec<Finding> {
    let path = format!("{project}/{COST_FILE}");
    let mut findings = Vec::new();
    let listed: BTreeSet<&str> = file.service.iter().map(|s| s.blip.as_str()).collect();

    let mut by_blip: BTreeMap<&str, Vec<&Hit>> = BTreeMap::new();
    for hit in &detection.hits {
        if let Some(blip) = &hit.blip {
            by_blip.entry(blip.as_str()).or_default().push(hit);
        }
    }
    for (blip, hits) in &by_blip {
        if listed.contains(blip) {
            continue;
        }
        findings.push(Finding {
            code: Code::Cost007,
            path: path.clone(),
            message: format!(
                "the code uses {blip} ({}), but there is no [[service]] for it; add one, or an [[exempt]] in standards/radar/detectors.toml if it is a false match",
                locations(hits)
            ),
        });
    }

    let mut counted: BTreeMap<&str, (u64, Vec<&Hit>)> = BTreeMap::new();
    for hit in &detection.hits {
        if let Some((key, n)) = &hit.usage {
            let entry = counted.entry(key.as_str()).or_default();
            entry.0 += n;
            entry.1.push(hit);
        }
    }
    for (key, (found, hits)) in counted {
        let Some(blip) = hits[0].blip.as_deref() else {
            continue; // parse_detectors rejects usage without a blip
        };
        if !listed.contains(blip) {
            continue; // already reported above
        }
        let expected: u64 = file.service.iter().filter_map(|s| s.usage.get(key)).sum();
        if expected < found {
            findings.push(Finding {
                code: Code::Cost007,
                path: path.clone(),
                message: format!(
                    "{key}: the code has {found} ({}), but [service.usage] expects {expected}",
                    locations(&hits)
                ),
            });
        }
    }

    for unknown in &detection.unknown_hosts {
        findings.push(Finding {
            code: Code::Cost009,
            path: unknown.path.clone(),
            message: format!(
                "unknown outside service `{}` ({}); list it in {path} if it is a vendor, or add it to [[allow]] in standards/radar/detectors.toml if it costs nothing",
                unknown.host, unknown.at
            ),
        });
    }
    findings
}

fn locations(hits: &[&Hit]) -> String {
    let mut at: Vec<&str> = hits.iter().map(|h| h.at.as_str()).collect();
    at.dedup();
    let more = at.len().saturating_sub(3);
    let mut text = at.into_iter().take(3).collect::<Vec<_>>().join(", ");
    if more > 0 {
        text.push_str(&format!(" and {more} more"));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_cost_file;

    const DETECTORS: &str = r#"
[scan]
extensions = ["rs", "toml", "json", "yml"]
skip_dirs = ["docs", "target"]
skip_files = ["Cargo.lock"]

[[detector]]
id = "auth0"
blip = "Auth0"
why = "Auth0 tenant"
hosts = ["auth0.com"]
patterns = ['\bAUTH0_[A-Z_]+']

[[detector]]
id = "posthog-package"
blip = "PostHog"
why = "PostHog SDK"
files = ["package.json"]
patterns = ['"posthog-js"\s*:']

[[detector]]
id = "pages"
blip = "Cloudflare Pages"
why = "Pages project"
wrangler = "pages_build_output_dir"
usage = "pages_projects"

[[detector]]
id = "crons"
blip = "Cloudflare Workers"
why = "Cron Triggers"
wrangler = "triggers.crons"
usage = "cron_triggers"

[[detector]]
id = "schedule"
blip = "GitHub (Actions, Dependabot, CodeQL)"
why = "scheduled workflow"
workflow_schedule = true

[[allow]]
host = "docs.rs"
why = "docs"

[[exempt]]
path = "app/src/guard.rs"
detector = "auth0"
why = "searches for the key"
"#;

    fn detectors() -> Detectors {
        parse_detectors(DETECTORS).expect("parses")
    }

    fn src(path: &str, text: &str) -> SourceFile {
        SourceFile {
            path: path.to_string(),
            text: text.to_string(),
        }
    }

    fn cost(text: &str) -> CostFile {
        parse_cost_file(&format!(
            "reviewed = \"2026-11-26\"\nsummary = \"x\"\n{text}"
        ))
        .expect("cost file")
    }

    fn codes(findings: &[Finding]) -> Vec<&'static str> {
        findings.iter().map(|f| f.code.as_str()).collect()
    }

    #[test]
    fn test_the_repo_detectors_parse() {
        let d = parse_detectors(include_str!("../../../standards/radar/detectors.toml"))
            .expect("detectors.toml parses");
        assert!(
            d.file
                .detector
                .iter()
                .any(|d| d.blip.as_deref() == Some("Auth0"))
        );
    }

    #[test]
    fn test_rejects_bad_detectors() {
        let base = "[scan]\nextensions = []\nskip_dirs = []\nskip_files = []\n";
        for (extra, want) in [
            (
                "[[detector]]\nid = \"x\"\nblip = \"y\"\nwhy = \"z\"\n",
                "exactly one",
            ),
            (
                "[[detector]]\nid = \"x\"\nblip = \"y\"\nwhy = \"z\"\npatterns = [\"(\"]\n",
                "regex",
            ),
            (
                "[[detector]]\nid = \"x\"\nblip = \"y\"\nwhy = \"z\"\nworkflow_schedule = true\n[[detector]]\nid = \"x\"\nblip = \"y\"\nwhy = \"z\"\nworkflow_schedule = true\n",
                "twice",
            ),
        ] {
            let err = parse_detectors(&format!("{base}{extra}")).expect_err(extra);
            assert!(err.contains(want), "{err}");
        }
    }

    #[test]
    fn test_threat_fields_are_checked() {
        let base = "[scan]\nextensions = []\nskip_dirs = []\nskip_files = []\n";
        for (extra, want) in [
            (
                "[[detector]]\nid = \"x\"\nwhy = \"z\"\npatterns = [\"a\"]\n",
                "a blip, a threat or both",
            ),
            (
                "[[detector]]\nid = \"x\"\nwhy = \"z\"\nthreat = \"secret\"\npatterns = [\"put [A-Z]+\"]\n",
                "capture group",
            ),
            (
                "[[detector]]\nid = \"x\"\nwhy = \"z\"\nthreat = \"url\"\nhosts = [\"a.dev\"]\n",
                "capture group",
            ),
            (
                "[[detector]]\nid = \"x\"\nwhy = \"z\"\nthreat = \"risk\"\nwrangler = \"main\"\nusage = \"workers\"\n",
                "no blip",
            ),
            (
                "[[detector]]\nid = \"x\"\nwhy = \"z\"\nthreat = \"magic\"\npatterns = [\"a\"]\n",
                "unknown variant",
            ),
        ] {
            let err = parse_detectors(&format!("{base}{extra}")).expect_err(extra);
            assert!(err.contains(want), "{extra}: {err}");
        }
    }

    #[test]
    fn test_threat_only_detectors_capture_names_and_cost_nothing() {
        let d = parse_detectors(&format!(
            "{DETECTORS}\n[[detector]]\nid = \"secret\"\nwhy = \"x\"\nthreat = \"secret\"\npatterns = ['secret put ([A-Z_]+)']\n"
        ))
        .expect("parses");
        let files = [src("app/x.rs", "wrangler secret put API_KEY\n")];
        let found = d.detect("app", &files, &[]);
        assert_eq!(found.hits.len(), 1);
        assert_eq!(found.hits[0].name.as_deref(), Some("API_KEY"));
        assert_eq!(found.hits[0].threat, Some(ThreatHint::Secret));
        assert!(found.blips().is_empty());
        assert!(!d.line_matches("app/x.rs", "wrangler secret put API_KEY"));
        assert!(check_drift("app", &cost(""), &found).is_empty());
    }

    #[test]
    fn test_scans_only_listed_extensions_outside_skipped_dirs() {
        let d = detectors();
        assert!(d.scans("app/src/main.rs"));
        assert!(d.scans("app/cloudflare/wrangler.toml"));
        assert!(!d.scans("app/docs/cost.toml"));
        assert!(!d.scans("app/target/x.rs"));
        assert!(!d.scans("app/Cargo.lock"));
        assert!(!d.scans("app/README.md"));
    }

    #[test]
    fn test_hosts_patterns_and_comments() {
        let d = detectors();
        let files = [
            src(
                "app/src/main.rs",
                "// see https://dev-1.us.auth0.com/ in a comment\nconst A: &str = \"https://dev-1.us.auth0.com/\";\nlet k = env(\"AUTH0_CLIENT\");\n",
            ),
            src("app/src/guard.rs", "grep AUTH0_KEY\n"),
            src("app/package.json", "{\"posthog-js\": \"1.0\"}\n"),
            src("app/src/other.json", "{\"posthog-js\": \"1.0\"}\n"),
        ];
        let found = d.detect("app", &files, &[]);
        let at: Vec<(&str, &str)> = found
            .hits
            .iter()
            .map(|h| (h.detector.as_str(), h.at.as_str()))
            .collect();
        assert_eq!(
            at,
            [
                ("auth0", "app/src/main.rs:2"),
                ("auth0", "app/src/main.rs:3"),
                ("posthog-package", "app/package.json:1"),
            ]
        );
        assert_eq!(
            found.blips().into_iter().collect::<Vec<_>>(),
            ["Auth0", "PostHog"]
        );
    }

    #[test]
    fn test_unknown_hosts_once_each_skipping_allowed_and_ips() {
        let d = detectors();
        let files = [src(
            "app/src/main.rs",
            "fetch(\"https://api.Example.org/v1\");\nfetch(\"https://api.example.org/v2\");\nlet d = \"https://docs.rs/x\";\nlet ip = \"http://127.0.0.1:8080\";\n<a href=\"https://link.example.net\">[x](https://md.example.net)</a>\n",
        )];
        let found = d.detect("app", &files, &[]);
        assert_eq!(found.unknown_hosts.len(), 1);
        assert_eq!(found.unknown_hosts[0].host, "api.example.org");
        assert_eq!(found.unknown_hosts[0].at, "app/src/main.rs:1");
    }

    #[test]
    fn test_wrangler_keys_count_entries() {
        let d = detectors();
        let files = [src(
            "app/cloudflare/wrangler.toml",
            "name = \"x\"\npages_build_output_dir = \"dist\"\n[triggers]\ncrons = [\"0 * * * *\", \"5 * * * *\"]\n",
        )];
        let found = d.detect("app", &files, &[]);
        let usage: Vec<_> = found.hits.iter().filter_map(|h| h.usage.clone()).collect();
        assert_eq!(
            usage,
            [
                ("pages_projects".to_string(), 1),
                ("cron_triggers".to_string(), 2)
            ]
        );
    }

    #[test]
    fn test_scheduled_workflows_named_after_the_project() {
        let d = detectors();
        let wf = [
            src(
                ".github/workflows/app-nightly.yml",
                "on:\n  schedule:\n    - cron: x\n",
            ),
            src(".github/workflows/apple.yml", "on:\n  schedule:\n"),
            src(".github/workflows/app.yml", "on:\n  # schedule:\n  push:\n"),
        ];
        let found = d.detect("app", &[], &wf);
        let at: Vec<&str> = found.hits.iter().map(|h| h.at.as_str()).collect();
        assert_eq!(at, [".github/workflows/app-nightly.yml"]);
    }

    #[test]
    fn test_drift_reports_unlisted_blips_and_undercounted_usage() {
        let d = detectors();
        let files = [
            src(
                "app/src/main.rs",
                "let k = \"AUTH0_ID\";\nlet u = \"https://api.nope.example.net\";\n",
            ),
            src(
                "app/wrangler.toml",
                "pages_build_output_dir = \"dist\"\n[triggers]\ncrons = [\"x\"]\n",
            ),
        ];
        let found = d.detect("app", &files, &[]);
        let file = cost(
            "[[service]]\nblip = \"Cloudflare Workers\"\nplan = \"free\"\nwhy = \"x\"\n[[service]]\nblip = \"Cloudflare Pages\"\nplan = \"free\"\nwhy = \"x\"\n[service.usage]\npages_projects = 1\n",
        );
        let findings = check_drift("app", &file, &found);
        assert_eq!(codes(&findings), ["COST-007", "COST-007", "COST-009"]);
        assert!(
            findings[0]
                .message
                .contains("uses Auth0 (app/src/main.rs:1)"),
            "{}",
            findings[0].message
        );
        assert!(
            findings[1]
                .message
                .starts_with("cron_triggers: the code has 1"),
            "{}",
            findings[1].message
        );
        assert_eq!(findings[2].path, "app/src/main.rs");
    }

    #[test]
    fn test_no_drift_when_the_file_lists_everything() {
        let d = detectors();
        let files = [src(
            "app/wrangler.toml",
            "pages_build_output_dir = \"dist\"\n",
        )];
        let found = d.detect("app", &files, &[]);
        let file = cost(
            "[[service]]\nblip = \"Cloudflare Pages\"\nplan = \"free\"\nwhy = \"x\"\n[service.usage]\npages_projects = 1\n",
        );
        assert!(check_drift("app", &file, &found).is_empty());
    }
}
