//! The rules. Each `check_*` function returns findings; none of them read
//! files.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::{Access, Code, Date, Finding, Flow, Kind, ThreatFile, parse_threat_file};

/// Where each project keeps its threat model.
pub const THREAT_FILE: &str = "docs/threat.toml";

/// THR-009 warns once `reviewed` is older than this.
pub const REVIEW_MAX_DAYS: i64 = 180;

/// What every public web surface must send (THR-007), as the lowercase
/// text the check looks for in its `headers` file. Each entry lists the
/// spellings that satisfy it.
pub const REQUIRED_HEADERS: &[(&str, &[&str])] = &[
    ("Content-Security-Policy", &["content-security-policy"]),
    ("X-Content-Type-Options: nosniff", &["nosniff"]),
    ("Referrer-Policy", &["referrer-policy"]),
    (
        "frame-ancestors (or X-Frame-Options)",
        &["frame-ancestors", "x-frame-options"],
    ),
    ("Strict-Transport-Security", &["strict-transport-security"]),
];

/// A workflow in `.github/workflows/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workflow {
    /// Repo-relative path.
    pub path: String,
    pub text: String,
}

/// What the rules need from outside the project's own file.
pub struct Context<'a> {
    /// The `id`s in `standards/settings.toml`.
    pub settings: &'a BTreeSet<String>,
    /// Reads a repo-relative file (a surface's `headers`), `None` when it
    /// doesn't exist.
    pub read: &'a dyn Fn(&str) -> Option<String>,
    /// The repo's workflows.
    pub workflows: &'a [Workflow],
    pub today: Date,
}

#[derive(Deserialize)]
struct SettingsFile {
    #[serde(default)]
    setting: Vec<SettingId>,
}

#[derive(Deserialize)]
struct SettingId {
    id: String,
}

/// The setting ids in `standards/settings.toml`.
pub fn parse_setting_ids(text: &str) -> Result<BTreeSet<String>, String> {
    let file: SettingsFile = toml::from_str(text).map_err(|e| e.message().trim().to_string())?;
    Ok(file.setting.into_iter().map(|s| s.id).collect())
}

/// Runs every rule on one project. `text` is its `docs/threat.toml`, or
/// `None` when it has none.
pub fn check_project(name: &str, text: Option<&str>, cx: &Context) -> Vec<Finding> {
    let path = format!("{name}/{THREAT_FILE}");
    let mut out = Findings {
        path: &path,
        list: Vec::new(),
    };
    let Some(text) = text else {
        out.add(
            Code::Thr001,
            "no threat model; write one (format in standards/docs/security.md)".into(),
        );
        return out.list;
    };
    let file = match parse_threat_file(text) {
        Ok(file) => file,
        Err(e) => {
            out.add(Code::Thr002, e);
            return out.list;
        }
    };
    check_names(&file, &mut out);
    check_surfaces(&file, &mut out);
    check_secrets(name, &file, cx, &mut out);
    check_auth(&file, cx, &mut out);
    check_headers(&file, cx, &mut out);
    check_threats(&file, &mut out);
    check_reviewed(&file, cx.today, &mut out);
    out.list
}

struct Findings<'a> {
    path: &'a str,
    list: Vec<Finding>,
}

impl Findings<'_> {
    fn add(&mut self, code: Code, message: String) {
        self.add_at(code, self.path.to_string(), message);
    }

    fn add_at(&mut self, code: Code, path: String, message: String) {
        self.list.push(Finding {
            code,
            path,
            message,
        });
    }
}

fn duplicates<'a>(names: impl Iterator<Item = &'a str>) -> BTreeSet<&'a str> {
    let mut seen = BTreeSet::new();
    names.filter(|n| !seen.insert(*n)).collect()
}

/// THR-002 beyond parsing: names that must be unique, and fields that only
/// mean something on some kinds.
fn check_names(file: &ThreatFile, out: &mut Findings) {
    for (what, dups) in [
        (
            "surface",
            duplicates(file.surface.iter().map(|s| s.name.as_str())),
        ),
        (
            "secret",
            duplicates(file.secret.iter().map(|s| s.name.as_str())),
        ),
        (
            "threat id",
            duplicates(file.threat.iter().map(|t| t.id.as_str())),
        ),
    ] {
        for dup in dups {
            out.add(Code::Thr002, format!("{what} `{dup}` is listed twice"));
        }
    }
    for s in &file.surface {
        if s.previews.is_some() && s.kind != Kind::Pages {
            out.add(
                Code::Thr002,
                format!(
                    "surface `{}`: `previews` is only for kind = \"pages\"",
                    s.name
                ),
            );
        }
    }
}

/// THR-003: nothing non-public on a public surface, previews included.
fn check_surfaces(file: &ThreatFile, out: &mut Findings) {
    for s in &file.surface {
        if s.access == Access::Public && s.data != crate::Data::Public {
            out.add(
                Code::Thr003,
                format!(
                    "surface `{}` serves {} data with access = \"public\"; put it behind auth0 or cloudflare-access",
                    s.name,
                    s.data.as_str()
                ),
            );
        }
        if s.kind != Kind::Pages || s.access == Access::None {
            continue;
        }
        match s.previews {
            None => out.add(
                Code::Thr003,
                format!(
                    "surface `{}` is on Pages but doesn't say what its preview URLs expose (`previews`)",
                    s.name
                ),
            ),
            Some(p) if p != s.access => out.add(
                Code::Thr003,
                format!(
                    "surface `{}` has access = \"{}\" but previews = \"{}\"; previews must be as closed as production",
                    s.name,
                    s.access.as_str(),
                    p.as_str()
                ),
            ),
            Some(_) => {}
        }
    }
}

/// The workflows that belong to a project: named after it, as
/// `<project>.yml` or `<project>-*.yml`, with `_` written `-`.
fn owns_workflow(project: &str, path: &str) -> bool {
    let file = path.rsplit('/').next().unwrap_or(path);
    let Some(stem) = file
        .strip_suffix(".yml")
        .or_else(|| file.strip_suffix(".yaml"))
    else {
        return false;
    };
    let dashed = project.replace('_', "-");
    [project, dashed.as_str()]
        .iter()
        .any(|p| stem == *p || stem.strip_prefix(p).is_some_and(|r| r.starts_with('-')))
}

/// Every `secrets.X` a workflow reads, with the first line it is on, except
/// `GITHUB_TOKEN` (GitHub makes it per run) and commented-out lines.
pub fn workflow_secrets(text: &str) -> BTreeMap<String, usize> {
    let mut found = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim_start().starts_with('#') {
            continue;
        }
        for (at, _) in line.match_indices("secrets.") {
            let before = line[..at].chars().next_back();
            if before.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
                continue;
            }
            let name: String = line[at + "secrets.".len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() && name != "GITHUB_TOKEN" {
                found.entry(name).or_insert(i + 1);
            }
        }
    }
    found
}

/// THR-005: every secret has a scope and a setting, and every secret the
/// project's workflows read is listed.
fn check_secrets(name: &str, file: &ThreatFile, cx: &Context, out: &mut Findings) {
    for s in &file.secret {
        if s.scope.as_deref().is_none_or(|v| v.trim().is_empty()) {
            out.add(
                Code::Thr005,
                format!("secret `{}` has no `scope` (what it can do)", s.name),
            );
        }
        match s.setting.as_deref() {
            None => out.add(
                Code::Thr005,
                format!(
                    "secret `{}` has no `setting` (its id in standards/settings.toml)",
                    s.name
                ),
            ),
            Some(id) if !cx.settings.contains(id) => out.add(
                Code::Thr005,
                format!(
                    "secret `{}` names setting `{id}`, which standards/settings.toml doesn't have",
                    s.name
                ),
            ),
            Some(_) => {}
        }
    }
    let listed: BTreeSet<&str> = file.secret.iter().map(|s| s.name.as_str()).collect();
    for wf in cx.workflows.iter().filter(|w| owns_workflow(name, &w.path)) {
        for (secret, line) in workflow_secrets(&wf.text) {
            if !listed.contains(secret.as_str()) {
                out.add_at(
                    Code::Thr005,
                    format!("{}:{line}", wf.path),
                    format!("passes secret `{secret}`, which {name}/{THREAT_FILE} doesn't list"),
                );
            }
        }
    }
}

/// The host of an `http(s)` URL, without port or user info.
fn http_host(url: &str) -> Option<(&str, &str)> {
    let (scheme, rest) = url.split_once("://")?;
    if scheme != "http" && scheme != "https" {
        return None;
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    let host = if authority.starts_with('[') {
        authority.split_inclusive(']').next().unwrap_or(authority)
    } else {
        authority.split(':').next().unwrap_or(authority)
    };
    Some((scheme, host))
}

fn is_localhost(host: &str) -> bool {
    matches!(host, "localhost" | "127.0.0.1" | "[::1]")
}

/// Our own hosts: the file's own surface URLs, and any `jimhillr-*` Pages
/// project (its preview subdomains included).
fn is_own_host(host: &str, surfaces: &BTreeSet<&str>) -> bool {
    if surfaces.contains(host) {
        return true;
    }
    host.strip_suffix(".pages.dev")
        .and_then(|h| h.rsplit('.').next())
        .is_some_and(|project| project.starts_with("jimhillr-"))
}

/// THR-006: the sign-in rules.
fn check_auth(file: &ThreatFile, cx: &Context, out: &mut Findings) {
    let Some(auth) = &file.auth else {
        return;
    };
    if !auth.provider.eq_ignore_ascii_case("auth0") {
        out.add(
            Code::Thr006,
            format!(
                "[auth] provider `{}` isn't Auth0, the approved sign-in provider",
                auth.provider
            ),
        );
    }
    let client = file.surface.iter().find(|s| s.kind.is_client());
    match (auth.flow, client) {
        (Flow::Implicit | Flow::Password, _) => out.add(
            Code::Thr006,
            format!(
                "[auth] flow `{}` is never allowed; use authorization-code-pkce",
                auth.flow.as_str()
            ),
        ),
        (Flow::AuthorizationCode | Flow::ClientCredentials, Some(s)) => out.add(
            Code::Thr006,
            format!(
                "[auth] flow `{}` needs a client secret, which surface `{}` ({:?}) can't keep; use authorization-code-pkce",
                auth.flow.as_str(),
                s.name,
                s.kind
            ),
        ),
        _ => {}
    }
    let surfaces: BTreeSet<&str> = file
        .surface
        .iter()
        .filter_map(|s| http_host(s.url.as_deref()?))
        .map(|(_, host)| host)
        .collect();
    for cb in &auth.callbacks {
        let Some((scheme, host)) = http_host(cb) else {
            if !cb.contains("://") {
                out.add(Code::Thr006, format!("[auth] callback `{cb}` is not a URL"));
            }
            // A native app's own scheme, such as com.example.app://callback.
            continue;
        };
        if is_localhost(host) {
            continue;
        }
        if scheme != "https" {
            out.add(
                Code::Thr006,
                format!("[auth] callback `{cb}` must use https (http only for localhost)"),
            );
        } else if !is_own_host(host, &surfaces) {
            out.add(
                Code::Thr006,
                format!(
                    "[auth] callback `{cb}` is on `{host}`, which isn't one of this project's surfaces or a jimhillr-* Pages project"
                ),
            );
        }
    }
    if let Some(id) = auth.setting.as_deref()
        && !cx.settings.contains(id)
    {
        out.add(
            Code::Thr006,
            format!("[auth] names setting `{id}`, which standards/settings.toml doesn't have"),
        );
    }
}

/// THR-007: every public web surface sends the security headers.
fn check_headers(file: &ThreatFile, cx: &Context, out: &mut Findings) {
    for s in &file.surface {
        if s.access != Access::Public || !s.kind.is_web() {
            continue;
        }
        let Some(headers) = s.headers.as_deref() else {
            out.add(
                Code::Thr007,
                format!(
                    "public surface `{}` has no `headers` file (a _headers file or the Worker that sets them)",
                    s.name
                ),
            );
            continue;
        };
        let Some(text) = (cx.read)(headers) else {
            out.add(
                Code::Thr007,
                format!(
                    "surface `{}`: headers file `{headers}` doesn't exist",
                    s.name
                ),
            );
            continue;
        };
        let text = text.to_ascii_lowercase();
        let missing: Vec<&str> = REQUIRED_HEADERS
            .iter()
            .filter(|(_, spellings)| !spellings.iter().any(|h| text.contains(h)))
            .map(|(name, _)| *name)
            .collect();
        if !missing.is_empty() {
            out.add_at(
                Code::Thr007,
                headers.to_string(),
                format!(
                    "surface `{}` is public but this file doesn't set {}",
                    s.name,
                    missing.join(", ")
                ),
            );
        }
    }
}

/// THR-008: every reachable surface has a threat, every threat a
/// mitigation.
fn check_threats(file: &ThreatFile, out: &mut Findings) {
    let names: BTreeSet<&str> = file.surface.iter().map(|s| s.name.as_str()).collect();
    for t in &file.threat {
        if t.mitigation.as_deref().is_none_or(|m| m.trim().is_empty()) {
            out.add(
                Code::Thr008,
                format!("threat `{}` has no `mitigation`", t.id),
            );
        }
        if let Some(surface) = t.surface.as_deref()
            && !names.contains(surface)
        {
            out.add(
                Code::Thr008,
                format!(
                    "threat `{}` is about surface `{surface}`, which the file doesn't list",
                    t.id
                ),
            );
        }
    }
    for s in file.surface.iter().filter(|s| s.access != Access::None) {
        let covered = file
            .threat
            .iter()
            .any(|t| t.surface.as_deref().is_none_or(|n| n == s.name));
        if !covered {
            out.add(
                Code::Thr008,
                format!(
                    "surface `{}` is reachable (access = \"{}\") but no [[threat]] covers it",
                    s.name,
                    s.access.as_str()
                ),
            );
        }
    }
}

/// THR-009: `reviewed` isn't older than [`REVIEW_MAX_DAYS`].
fn check_reviewed(file: &ThreatFile, today: Date, out: &mut Findings) {
    let age = today.days() - file.reviewed.days();
    if age > REVIEW_MAX_DAYS {
        out.add(
            Code::Thr009,
            format!(
                "reviewed {} ({age} days ago); check it against the code and update `reviewed`",
                file.reviewed
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "reviewed = \"2026-11-26\"\nsummary = \"x\"\n";
    const HEADERS: &str = "/*\n  Content-Security-Policy: default-src 'self'; frame-ancestors 'none'\n  X-Content-Type-Options: nosniff\n  Referrer-Policy: no-referrer\n  Strict-Transport-Security: max-age=31536000\n";

    fn settings() -> BTreeSet<String> {
        ["cf-token", "auth0-app"].map(String::from).into()
    }

    fn read(path: &str) -> Option<String> {
        match path {
            "app/_headers" => Some(HEADERS.to_string()),
            "app/thin_headers" => Some("/*\n  X-Frame-Options: DENY\n".to_string()),
            _ => None,
        }
    }

    fn run_with(
        text: Option<&str>,
        workflows: &[Workflow],
        today: &str,
    ) -> Vec<(&'static str, String, String)> {
        let settings = settings();
        let cx = Context {
            settings: &settings,
            read: &read,
            workflows,
            today: today.parse().expect("today"),
        };
        check_project("app", text, &cx)
            .into_iter()
            .map(|f| (f.code.as_str(), f.path, f.message))
            .collect()
    }

    fn run(extra: &str) -> Vec<(&'static str, String, String)> {
        run_with(Some(&format!("{BASE}{extra}")), &[], "2026-11-27")
    }

    fn codes(extra: &str) -> Vec<&'static str> {
        run(extra).into_iter().map(|(c, _, _)| c).collect()
    }

    const SITE: &str = "[[surface]]\nname = \"site\"\nkind = \"pages\"\nurl = \"https://jimhillr-app.pages.dev\"\naccess = \"public\"\ndata = \"public\"\npreviews = \"public\"\nheaders = \"app/_headers\"\n";
    const THREAT: &str =
        "[[threat]]\nid = \"T1\"\nstride = \"tampering\"\nwhat = \"x\"\nmitigation = \"y\"\n";

    #[test]
    fn test_a_good_file_has_no_findings() {
        assert_eq!(run(&format!("{SITE}{THREAT}")), []);
        assert_eq!(run(""), []);
    }

    #[test]
    fn test_thr001_missing_file() {
        let found = run_with(None, &[], "2026-11-27");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "THR-001");
        assert_eq!(found[0].1, "app/docs/threat.toml");
    }

    #[test]
    fn test_thr002_parse_errors_and_duplicates() {
        let found = run_with(Some("summary = 1\n"), &[], "2026-11-27");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "THR-002");
        let dup = format!("{SITE}{SITE}{THREAT}{THREAT}");
        let found = run(&dup);
        let msgs: Vec<&str> = found.iter().map(|f| f.2.as_str()).collect();
        assert!(msgs.contains(&"surface `site` is listed twice"), "{msgs:?}");
        assert!(msgs.contains(&"threat id `T1` is listed twice"), "{msgs:?}");
        let previews = "[[surface]]\nname = \"api\"\nkind = \"worker\"\naccess = \"none\"\ndata = \"public\"\npreviews = \"none\"\n";
        assert_eq!(codes(previews), ["THR-002"]);
    }

    #[test]
    fn test_thr003_private_data_on_a_public_surface() {
        let site = SITE.replace("data = \"public\"", "data = \"private\"");
        let found = run(&format!("{site}{THREAT}"));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].0, "THR-003");
        assert!(found[0].2.contains("private data"), "{}", found[0].2);
    }

    #[test]
    fn test_thr003_previews_must_match_access() {
        let open = SITE
            .replace("access = \"public\"", "access = \"cloudflare-access\"")
            .replace("data = \"public\"", "data = \"private\"");
        assert_eq!(codes(&format!("{open}{THREAT}")), ["THR-003"]);
        let closed = open.replace("previews = \"public\"", "previews = \"cloudflare-access\"");
        assert_eq!(codes(&format!("{closed}{THREAT}")), Vec::<&str>::new());
        let silent = SITE.replace("previews = \"public\"\n", "");
        assert_eq!(codes(&format!("{silent}{THREAT}")), ["THR-003"]);
        let undeployed =
            "[[surface]]\nname = \"s\"\nkind = \"pages\"\naccess = \"none\"\ndata = \"public\"\n";
        assert_eq!(codes(undeployed), Vec::<&str>::new());
    }

    #[test]
    fn test_thr003_token_access_is_closed() {
        let api = "[[surface]]\nname = \"state\"\nkind = \"api\"\naccess = \"token\"\ndata = \"private\"\n";
        assert_eq!(codes(&format!("{api}{THREAT}")), Vec::<&str>::new());
    }

    #[test]
    fn test_thr005_secrets_need_a_scope_and_a_known_setting() {
        let good = "[[secret]]\nname = \"TOKEN\"\nwhere = \"github\"\nscope = \"read\"\nsetting = \"cf-token\"\n";
        assert_eq!(codes(good), Vec::<&str>::new());
        let bare = "[[secret]]\nname = \"TOKEN\"\nwhere = \"github\"\n";
        assert_eq!(codes(bare), ["THR-005", "THR-005"]);
        let unknown =
            "[[secret]]\nname = \"TOKEN\"\nwhere = \"github\"\nscope = \" \"\nsetting = \"nope\"\n";
        let found = run(unknown);
        assert_eq!(found.len(), 2);
        assert!(found[1].2.contains("setting `nope`"), "{}", found[1].2);
    }

    #[test]
    fn test_thr005_workflow_secrets_must_be_listed() {
        let wf = |path: &str| {
            Workflow {
            path: path.to_string(),
            text: "jobs:\n  a:\n    env:\n      A: ${{ secrets.TOKEN }}\n      B: ${{ secrets.OTHER }}\n      # C: ${{ secrets.OLD }}\n      D: ${{ secrets.GITHUB_TOKEN }}\n".to_string(),
        }
        };
        let listed = "[[secret]]\nname = \"TOKEN\"\nwhere = \"github\"\nscope = \"read\"\nsetting = \"cf-token\"\n";
        let workflows = [
            wf(".github/workflows/app-deploy.yml"),
            wf(".github/workflows/apple.yml"),
            wf(".github/workflows/all-ci.yml"),
        ];
        let found = run_with(Some(&format!("{BASE}{listed}")), &workflows, "2026-11-27");
        assert_eq!(
            found,
            [(
                "THR-005",
                ".github/workflows/app-deploy.yml:5".to_string(),
                "passes secret `OTHER`, which app/docs/threat.toml doesn't list".to_string()
            )]
        );
    }

    #[test]
    fn test_workflow_ownership_and_secret_names() {
        assert!(owns_workflow(
            "tofu_state",
            ".github/workflows/tofu-state-deploy.yml"
        ));
        assert!(owns_workflow("evals", ".github/workflows/evals-store.yml"));
        assert!(owns_workflow("infra", ".github/workflows/infra.yaml"));
        assert!(!owns_workflow(
            "infra",
            ".github/workflows/infrastructure.yml"
        ));
        assert!(!owns_workflow("infra", ".github/workflows/infra-plan.txt"));
        let found = workflow_secrets("x: ${{ secrets.A_1 }} ${{secrets.B}} mysecrets.C secrets.\n");
        assert_eq!(found.keys().collect::<Vec<_>>(), ["A_1", "B"]);
    }

    #[test]
    fn test_thr006_sign_in_rules() {
        let auth = |flow: &str, callbacks: &str| {
            format!(
                "{SITE}{THREAT}[auth]\nprovider = \"Auth0\"\nflow = \"{flow}\"\ncallbacks = [{callbacks}]\nsetting = \"auth0-app\"\n"
            )
        };
        let ok = "\"https://jimhillr-app.pages.dev/callback\", \"https://abc.jimhillr-app.pages.dev/cb\", \"http://localhost:8080/callback\", \"http://127.0.0.1:3000\", \"com.jimhillr.app://callback\"";
        assert_eq!(
            codes(&auth("authorization-code-pkce", ok)),
            Vec::<&str>::new()
        );
        assert_eq!(codes(&auth("implicit", ok)), ["THR-006"]);
        assert_eq!(codes(&auth("client-credentials", ok)), ["THR-006"]);
        for bad in [
            "\"https://evil.example/callback\"",
            "\"http://jimhillr-app.pages.dev/callback\"",
            "\"https://jimhillr-app.pages.dev.evil.example/cb\"",
            "\"https://other-app.pages.dev/cb\"",
            "\"/callback\"",
        ] {
            assert_eq!(
                codes(&auth("authorization-code-pkce", bad)),
                ["THR-006"],
                "{bad}"
            );
        }
        let server = "[[surface]]\nname = \"api\"\nkind = \"worker\"\nurl = \"https://api.example.com\"\naccess = \"auth0\"\ndata = \"personal\"\n";
        let m2m = format!(
            "{server}{THREAT}[auth]\nprovider = \"auth0\"\nflow = \"client-credentials\"\n"
        );
        assert_eq!(codes(&m2m), Vec::<&str>::new());
        let other = format!(
            "{server}{THREAT}[auth]\nprovider = \"Okta\"\nflow = \"authorization-code-pkce\"\ncallbacks = [\"https://api.example.com/cb\"]\nsetting = \"nope\"\n"
        );
        assert_eq!(codes(&other), ["THR-006", "THR-006"]);
    }

    #[test]
    fn test_thr007_public_web_surfaces_need_headers() {
        let none = SITE.replace("headers = \"app/_headers\"\n", "");
        assert_eq!(codes(&format!("{none}{THREAT}")), ["THR-007"]);
        let gone = SITE.replace("app/_headers", "app/missing");
        assert_eq!(codes(&format!("{gone}{THREAT}")), ["THR-007"]);
        let thin = SITE.replace("app/_headers", "app/thin_headers");
        let found = run(&format!("{thin}{THREAT}"));
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].1, "app/thin_headers");
        assert!(
            found[0].2.ends_with("Content-Security-Policy, X-Content-Type-Options: nosniff, Referrer-Policy, Strict-Transport-Security"),
            "{}",
            found[0].2
        );
        let cli = "[[surface]]\nname = \"tool\"\nkind = \"cli\"\naccess = \"public\"\ndata = \"public\"\n";
        assert_eq!(codes(&format!("{cli}{THREAT}")), Vec::<&str>::new());
    }

    #[test]
    fn test_thr008_threats_cover_reachable_surfaces() {
        assert_eq!(codes(SITE), ["THR-008"]);
        let other =
            format!("{SITE}{THREAT}").replace("mitigation = \"y\"\n", "surface = \"nope\"\n");
        let found = run(&other);
        let found: Vec<&str> = found.iter().map(|f| f.2.as_str()).collect();
        assert_eq!(
            found,
            [
                "threat `T1` has no `mitigation`",
                "threat `T1` is about surface `nope`, which the file doesn't list",
                "surface `site` is reachable (access = \"public\") but no [[threat]] covers it",
            ]
        );
        let named = format!("{SITE}{THREAT}surface = \"site\"\n");
        assert_eq!(codes(&named), Vec::<&str>::new());
    }

    #[test]
    fn test_thr009_old_reviews_warn() {
        let text = BASE.to_string();
        assert_eq!(run_with(Some(&text), &[], "2027-05-25"), []);
        let found = run_with(Some(&text), &[], "2027-05-26");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "THR-009");
        assert!(!Code::Thr009.can_fail() && Code::Thr003.can_fail());
    }

    #[test]
    fn test_setting_ids() {
        let ids =
            parse_setting_ids("[[setting]]\nid = \"a\"\nvendor = \"x\"\n[[setting]]\nid = \"b\"\n")
                .expect("parses");
        assert_eq!(ids.into_iter().collect::<Vec<_>>(), ["a", "b"]);
        assert!(parse_setting_ids("[[setting]]\nvendor = \"x\"\n").is_err());
    }
}
