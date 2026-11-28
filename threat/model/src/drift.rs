//! THR-004: is the threat model true to the code? The cost check's shared
//! detectors (`standards/radar/detectors.toml`) find what a project's files
//! show; each detector's `threat` field says what that implies for its
//! `docs/threat.toml`. One detector engine reads the files for both checks.

use std::collections::{BTreeMap, BTreeSet};

use cost_model::{Detection, Detectors, Hit, ThreatHint};

use crate::{Access, Code, Data, Finding, Kind, THREAT_FILE, ThreatFile, parse_threat_file};

/// THR-004 for one project. `text` is its `docs/threat.toml`; a missing or
/// broken file gives nothing here, because THR-001 or THR-002 already
/// reports it. `detection` is what the detectors found in the project's
/// tracked files.
pub fn check_drift(
    name: &str,
    text: Option<&str>,
    detectors: &Detectors,
    detection: &Detection,
) -> Vec<Finding> {
    let Some(Ok(file)) = text.map(parse_threat_file) else {
        return Vec::new();
    };
    let path = format!("{name}/{THREAT_FILE}");
    let mut out = Vec::new();
    let mut add = |message: String| {
        out.push(Finding {
            code: Code::Thr004,
            path: path.clone(),
            message,
        });
    };

    let ids: BTreeSet<&str> = detectors
        .file
        .detector
        .iter()
        .map(|d| d.id.as_str())
        .collect();
    for t in &file.threat {
        for c in t.covers.iter().filter(|c| !ids.contains(c.as_str())) {
            add(format!(
                "threat `{}` covers `{c}`, which isn't a detector in standards/radar/detectors.toml",
                t.id
            ));
        }
    }

    let mut by_hint: BTreeMap<ThreatHint, Vec<&Hit>> = BTreeMap::new();
    for hit in &detection.hits {
        if let Some(hint) = hit.threat {
            by_hint.entry(hint).or_default().push(hit);
        }
    }
    for (hint, hits) in &by_hint {
        match hint {
            ThreatHint::Pages => {
                if !has_kind(&file, &[Kind::Pages]) {
                    add(format!(
                        "the code deploys a Pages site ({}), but no [[surface]] has kind = \"pages\"",
                        locations(hits)
                    ));
                }
            }
            ThreatHint::Worker => {
                if !has_kind(&file, &[Kind::Worker, Kind::Api]) {
                    add(format!(
                        "the code deploys a Worker ({}), but no [[surface]] has kind = \"worker\" or \"api\"",
                        locations(hits)
                    ));
                }
            }
            ThreatHint::Url => {
                for (host, hits) in named(hits, |n| host_of(n).map(str::to_string)) {
                    let listed = file
                        .surface
                        .iter()
                        .filter_map(|s| s.url.as_deref().and_then(host_of))
                        .any(|h| h == host);
                    if !listed {
                        add(format!(
                            "the code deploys to {host} ({}), but no [[surface]] has a url on that host",
                            locations(&hits)
                        ));
                    }
                }
            }
            ThreatHint::Storage => {
                let stored = file
                    .surface
                    .iter()
                    .any(|s| s.access != Access::None && s.data != Data::Public);
                for (detector, hits) in by_detector(hits) {
                    if !stored && !covered(&file, detector) {
                        add(format!(
                            "the code stores data ({detector} at {}), but every reachable surface says data = \"public\"; set the surface's data, or add a [[threat]] with covers = [\"{detector}\"] saying why it is public",
                            locations(&hits)
                        ));
                    }
                }
            }
            ThreatHint::Secret => {
                let listed: BTreeSet<&str> = file.secret.iter().map(|s| s.name.as_str()).collect();
                for (secret, hits) in named(hits, |n| Some(n.to_string())) {
                    if !listed.contains(secret.as_str()) {
                        add(format!(
                            "the code sets the secret {secret} ({}), but there is no [[secret]] named {secret}",
                            locations(&hits)
                        ));
                    }
                }
            }
            ThreatHint::Auth => {
                if file.auth.is_none() {
                    add(format!(
                        "the code uses a sign-in library ({}), but there is no [auth] table",
                        locations(hits)
                    ));
                }
            }
            ThreatHint::Risk => {
                for (detector, hits) in by_detector(hits) {
                    if !covered(&file, detector) {
                        let why = detectors
                            .file
                            .detector
                            .iter()
                            .find(|d| d.id == detector)
                            .map_or("", |d| d.why.as_str());
                        add(format!(
                            "`{detector}` at {}: {why}; add a [[threat]] with covers = [\"{detector}\"] and its mitigation",
                            locations(&hits)
                        ));
                    }
                }
            }
        }
    }
    out
}

fn has_kind(file: &ThreatFile, kinds: &[Kind]) -> bool {
    file.surface.iter().any(|s| kinds.contains(&s.kind))
}

fn covered(file: &ThreatFile, detector: &str) -> bool {
    file.threat
        .iter()
        .any(|t| t.covers.iter().any(|c| c == detector))
}

fn by_detector<'a>(hits: &[&'a Hit]) -> BTreeMap<&'a str, Vec<&'a Hit>> {
    let mut map: BTreeMap<&str, Vec<&Hit>> = BTreeMap::new();
    for hit in hits {
        map.entry(hit.detector.as_str()).or_default().push(hit);
    }
    map
}

/// Groups hits by the key `key` makes of their captured name.
fn named<'a>(
    hits: &[&'a Hit],
    key: impl Fn(&str) -> Option<String>,
) -> BTreeMap<String, Vec<&'a Hit>> {
    let mut map: BTreeMap<String, Vec<&Hit>> = BTreeMap::new();
    for hit in hits {
        if let Some(k) = hit.name.as_deref().and_then(&key) {
            map.entry(k).or_default().push(hit);
        }
    }
    map
}

/// The lowercase host of `https://host[:port]/...`, or `None`.
fn host_of(url: &str) -> Option<&str> {
    let rest = url.split_once("://")?.1;
    let host = rest.split(['/', ':', '?', '#']).next()?;
    (!host.is_empty()).then_some(host)
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
    use cost_model::{SourceFile, parse_detectors};

    const DETECTORS: &str = r#"
[scan]
extensions = ["rs", "toml", "sh", "ts"]
skip_dirs = ["docs"]
skip_files = []

[[detector]]
id = "pages"
blip = "Cloudflare Pages"
why = "Pages"
wrangler = "pages_build_output_dir"
threat = "pages"

[[detector]]
id = "worker"
blip = "Cloudflare Workers"
why = "Worker"
wrangler = "main"
threat = "worker"

[[detector]]
id = "d1"
blip = "Cloudflare D1"
why = "D1"
wrangler = "d1_databases"
threat = "storage"

[[detector]]
id = "auth0-package"
blip = "Auth0"
why = "Auth0 SDK"
files = ["Cargo.toml"]
patterns = ['^\s*auth0[a-z0-9_-]*\s*=']
threat = "auth"

[[detector]]
id = "deploy-url"
threat = "url"
why = "deploy URL"
files = ["mise.toml"]
patterns = ['^\s*DEPLOY_URL\s*=\s*"(https?://[^"/]+)']

[[detector]]
id = "wrangler-secret"
threat = "secret"
why = "wrangler secret"
patterns = ['wrangler\s+(?:pages\s+)?secret\s+put\s+([A-Z][A-Z0-9_]*)']

[[detector]]
id = "raw-html"
threat = "risk"
why = "raw HTML can carry XSS"
patterns = ['dangerouslySetInnerHTML']
"#;

    fn src(path: &str, text: &str) -> SourceFile {
        SourceFile {
            path: path.to_string(),
            text: text.to_string(),
        }
    }

    fn files() -> Vec<SourceFile> {
        vec![
            src(
                "app/cloudflare/wrangler.toml",
                "pages_build_output_dir = \"dist\"\nmain = \"src/index.ts\"\n[[d1_databases]]\nbinding = \"DB\"\n",
            ),
            src(
                "app/cloudflare/mise.toml",
                "[env]\nDEPLOY_URL = \"https://jimhillr-app.pages.dev\"\n",
            ),
            src("app/Cargo.toml", "[dependencies]\nauth0_sdk = \"1\"\n"),
            src(
                "app/scripts/setup.sh",
                "npx wrangler pages secret put API_KEY\n# wrangler secret put COMMENTED\n",
            ),
            src("app/web/a.ts", "<div dangerouslySetInnerHTML={x} />\n"),
        ]
    }

    fn run(threat: &str) -> Vec<String> {
        let detectors = parse_detectors(DETECTORS).expect("detectors");
        let detection = detectors.detect("app", &files(), &[]);
        let text = format!("reviewed = \"2026-11-26\"\nsummary = \"x\"\n{threat}");
        check_drift("app", Some(&text), &detectors, &detection)
            .into_iter()
            .map(|f| {
                assert_eq!(f.code, Code::Thr004);
                assert_eq!(f.path, "app/docs/threat.toml");
                f.message
            })
            .collect()
    }

    const COMPLETE: &str = r#"
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
url = "https://jimhillr-app.pages.dev/api"
access = "auth0"
data = "personal"

[[secret]]
name = "API_KEY"
where = "worker"

[auth]
provider = "Auth0"
flow = "authorization-code-pkce"

[[threat]]
id = "T1"
stride = "tampering"
what = "XSS through rendered HTML"
mitigation = "Sanitised first"
covers = ["raw-html"]
"#;

    #[test]
    fn test_no_drift_when_the_file_lists_everything() {
        assert_eq!(run(COMPLETE), Vec::<String>::new());
    }

    #[test]
    fn test_reports_everything_an_empty_file_misses() {
        let found = run("");
        let starts: Vec<&str> = found
            .iter()
            .map(|m| m.split(" (").next().unwrap_or_default())
            .collect();
        assert_eq!(
            starts,
            [
                "the code deploys a Pages site",
                "the code deploys a Worker",
                "the code deploys to jimhillr-app.pages.dev",
                "the code stores data",
                "the code sets the secret API_KEY",
                "the code uses a sign-in library",
                "`raw-html` at app/web/a.ts:1: raw HTML can carry XSS; add a [[threat]] with covers = [\"raw-html\"] and its mitigation",
            ]
        );
        assert!(
            found[0].contains("app/cloudflare/wrangler.toml"),
            "{}",
            found[0]
        );
        assert!(found[4].contains("app/scripts/setup.sh:1"), "{}", found[4]);
    }

    #[test]
    fn test_storage_with_only_public_data_needs_a_covering_threat() {
        let public = COMPLETE.replace("data = \"personal\"", "data = \"public\"");
        let found = run(&public);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].contains("d1 at app/cloudflare/wrangler.toml"),
            "{}",
            found[0]
        );
        let covered = public.replace("covers = [\"raw-html\"]", "covers = [\"raw-html\", \"d1\"]");
        assert_eq!(run(&covered), Vec::<String>::new());
    }

    #[test]
    fn test_url_matches_on_host_only() {
        let other = COMPLETE.replace(
            "url = \"https://jimhillr-app.pages.dev\"",
            "url = \"https://other.pages.dev\"",
        );
        // The api surface's url is still on the deploy host.
        assert_eq!(run(&other), Vec::<String>::new());
        let both = other.replace(
            "url = \"https://jimhillr-app.pages.dev/api\"",
            "url = \"https://other.pages.dev/api\"",
        );
        let found = run(&both);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].starts_with("the code deploys to jimhillr-app.pages.dev"));
    }

    #[test]
    fn test_unknown_covers_are_reported() {
        let found = run(&COMPLETE.replace(
            "covers = [\"raw-html\"]",
            "covers = [\"raw-html\", \"nope\"]",
        ));
        assert_eq!(
            found,
            ["threat `T1` covers `nope`, which isn't a detector in standards/radar/detectors.toml"]
        );
    }

    #[test]
    fn test_missing_or_broken_files_are_left_to_thr_001_and_002() {
        let detectors = parse_detectors(DETECTORS).expect("detectors");
        let detection = detectors.detect("app", &files(), &[]);
        assert!(check_drift("app", None, &detectors, &detection).is_empty());
        assert!(check_drift("app", Some("nope ="), &detectors, &detection).is_empty());
    }

    #[test]
    fn test_host_of() {
        assert_eq!(host_of("https://a.b.dev/x?y"), Some("a.b.dev"));
        assert_eq!(host_of("http://localhost:8080/cb"), Some("localhost"));
        assert_eq!(host_of("jimhillr.dev"), None);
    }
}
