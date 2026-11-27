//! `evals check`: validates every project's `docs/requirements.md` and
//! `evals.toml`, and the human-check files (`standards/settings.toml` and
//! `standards/settings-checks.jsonl`), against `standards/docs/evals.md`.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use evals_model::{
    Eval, Kind, Priority, Requirement, SettingCheck, parse_evals, parse_requirements,
    parse_settings,
};

pub const REQUIREMENTS: &str = "docs/requirements.md";
pub const EVALS: &str = "evals.toml";
pub const NAMING: &str = "standards/docs/naming.md";
pub const SETTINGS: &str = "standards/settings.toml";
pub const SETTING_CHECKS: &str = "standards/settings-checks.jsonl";

/// One problem, printed as `warning[EVL-001] cookbook/evals.toml: ...`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// `EVL-001` to `EVL-006`, or `None` for a malformed file.
    pub code: Option<&'static str>,
    pub path: String,
    pub line: Option<usize>,
    pub message: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.code {
            Some(code) => write!(f, "warning[{code}] ")?,
            None => write!(f, "warning ")?,
        }
        write!(f, "{}", self.path)?;
        if let Some(line) = self.line {
            write!(f, ":{line}")?;
        }
        write!(f, ": {}", self.message)
    }
}

/// What the tests in a project look like, for EVL-003.
#[derive(Debug, Default)]
pub struct TestIndex {
    /// Names of Rust functions (`fn name`) in `.rs` files.
    pub rust_fns: Vec<String>,
    /// Contents of `.ts` and `.js` files, which hold Playwright tests.
    pub script_sources: Vec<String>,
    /// Every file path, relative to the project, with `/` separators.
    pub files: Vec<String>,
}

impl TestIndex {
    fn matches(&self, eval: &Eval) -> bool {
        let Some(selector) = eval.test_selector() else {
            return true;
        };
        match &eval.kind {
            Kind::Cargo { .. } => self.rust_fns.iter().any(|f| f.starts_with(&selector)),
            Kind::Playwright { .. } | Kind::Fault { .. } => {
                self.script_sources.iter().any(|s| s.contains(&selector))
            }
            Kind::Trycmd { .. } => self.files.iter().any(|f| glob_matches_tail(&selector, f)),
            _ => true,
        }
    }
}

/// Checks one project from its files' text. `None` means the file is absent.
/// `settings` are the ids in `standards/settings.toml`.
pub fn check_project(
    project: &str,
    expected_prefix: Option<&str>,
    requirements_text: Option<&str>,
    evals_text: Option<&str>,
    tests: &TestIndex,
    settings: &[String],
) -> Vec<Finding> {
    let requirements_path = format!("{project}/{REQUIREMENTS}");
    let evals_path = format!("{project}/{EVALS}");
    let mut findings = Vec::new();
    let mut finding = |code, path: &str, line, message: String| {
        findings.push(Finding {
            code,
            path: path.to_string(),
            line,
            message,
        });
    };

    let Some(prefix) = expected_prefix else {
        finding(
            None,
            if evals_text.is_some() {
                &evals_path
            } else {
                &requirements_path
            },
            None,
            format!(
                "{project} has no requirement prefix in {NAMING}; add one before its first requirement"
            ),
        );
        return findings;
    };

    let mut requirements: Vec<Requirement> = Vec::new();
    if let Some(text) = requirements_text {
        let (parsed, errors) = parse_requirements(text);
        for error in errors {
            finding(None, &requirements_path, error.line, error.message);
        }
        for requirement in &parsed {
            if requirement.prefix() != prefix {
                finding(
                    None,
                    &requirements_path,
                    Some(requirement.line),
                    format!(
                        "{} does not use {project}'s prefix {prefix}",
                        requirement.id
                    ),
                );
            }
        }
        requirements = parsed;
    }

    let mut evals: Vec<Eval> = Vec::new();
    if let Some(text) = evals_text {
        match parse_evals(text) {
            Ok(file) if file.prefix != prefix => finding(
                None,
                &evals_path,
                None,
                format!(
                    "prefix `{}` does not match {project}'s prefix {prefix} in {NAMING}",
                    file.prefix
                ),
            ),
            Ok(file) => evals = file.evals,
            Err(errors) => {
                for error in errors {
                    finding(None, &evals_path, error.line, error.message);
                }
            }
        }
    }

    // EVL-002: duplicate IDs.
    let mut first_line: HashMap<&str, usize> = HashMap::new();
    for requirement in &requirements {
        if let Some(first) = first_line.insert(&requirement.id, requirement.line) {
            finding(
                Some("EVL-002"),
                &requirements_path,
                Some(requirement.line),
                format!(
                    "{} is used twice (first on line {first}); IDs are never reused",
                    requirement.id
                ),
            );
        }
    }
    let mut seen_evals: Vec<&str> = Vec::new();
    for eval in &evals {
        if seen_evals.contains(&eval.id.as_str()) {
            finding(
                Some("EVL-002"),
                &evals_path,
                None,
                format!("eval id `{}` is used twice", eval.id),
            );
        }
        seen_evals.push(&eval.id);
    }

    // EVL-001: evals must name an active requirement.
    let by_id: BTreeMap<&str, &Requirement> =
        requirements.iter().map(|r| (r.id.as_str(), r)).collect();
    for eval in &evals {
        match by_id.get(eval.requirement.as_str()) {
            None => finding(
                Some("EVL-001"),
                &evals_path,
                None,
                format!(
                    "eval `{}` names {}, which is not in {requirements_path}",
                    eval.id, eval.requirement
                ),
            ),
            Some(requirement) if !requirement.is_active() => finding(
                Some("EVL-001"),
                &evals_path,
                None,
                format!(
                    "eval `{}` names {}, which is retired",
                    eval.id, eval.requirement
                ),
            ),
            Some(_) => {}
        }
    }

    // EVL-001, widened: a human-check must name a setting in the manifest.
    for eval in &evals {
        if let Kind::HumanCheck { setting } = &eval.kind
            && !settings.contains(setting)
        {
            finding(
                Some("EVL-001"),
                &evals_path,
                None,
                format!(
                    "eval `{}` names setting `{setting}`, which is not in {SETTINGS}",
                    eval.id
                ),
            );
        }
    }

    // EVL-003: test-report evals must match a real test.
    for eval in &evals {
        if !tests.matches(eval) {
            let selector = eval.test_selector().unwrap_or_default();
            finding(
                Some("EVL-003"),
                &evals_path,
                None,
                format!(
                    "{} eval `{}` matches no test in {project} (looked for `{selector}`)",
                    eval.kind.name(),
                    eval.id
                ),
            );
        }
    }

    // EVL-006: a page request must check what came back, not only that
    // something answered.
    for eval in &evals {
        if shallow_http(&eval.kind) {
            finding(
                Some("EVL-006"),
                &evals_path,
                None,
                format!(
                    "http eval `{}` only checks that its path answers; a single-page app \
                     answers every path with the same page, so add body_contains or asset_contains",
                    eval.id
                ),
            );
        }
    }

    // EVL-004: every active must requirement needs a production-stage eval.
    for requirement in &requirements {
        if requirement.priority != Priority::Must || !requirement.is_active() {
            continue;
        }
        let covered = evals
            .iter()
            .any(|e| e.requirement == requirement.id && e.runs_in_production());
        if !covered {
            finding(
                Some("EVL-004"),
                &requirements_path,
                Some(requirement.line),
                format!(
                    "{} is a must with no eval in a production stage (cd, chaos or observe)",
                    requirement.id
                ),
            );
        }
    }

    findings
}

/// EVL-006: a `GET` of a `path` that expects a 2xx and checks no content.
fn shallow_http(kind: &Kind) -> bool {
    let Kind::Http {
        path: Some(_),
        method,
        status,
        body_contains: None,
        asset_contains: None,
        ..
    } = kind
    else {
        return false;
    };
    method.as_deref().unwrap_or("GET") == "GET" && (200..300).contains(&status.unwrap_or(200))
}

/// Reads the `## Requirement prefixes` table in `naming.md`.
pub fn parse_prefixes(naming: &str) -> BTreeMap<String, String> {
    let mut prefixes = BTreeMap::new();
    let mut in_section = false;
    for line in naming.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            in_section = heading.trim() == "Requirement prefixes";
            continue;
        }
        if !in_section {
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if let [project, prefix] = cells.as_slice() {
            let prefix = prefix.trim_matches('`');
            if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_uppercase()) {
                prefixes.insert(project.to_string(), prefix.to_string());
            }
        }
    }
    prefixes
}

/// Checks every top-level project in the repo at `root`.
pub fn check_repo(root: &Path) -> Result<Vec<Finding>> {
    let naming_path = root.join(NAMING);
    let naming = fs::read_to_string(&naming_path)
        .with_context(|| format!("reading {}", naming_path.display()))?;
    let prefixes = parse_prefixes(&naming);

    let (mut findings, settings) = check_settings(root)?;
    for project in top_level_dirs(root)? {
        let dir = root.join(&project);
        let requirements = read_optional(&dir.join(REQUIREMENTS))?;
        let evals = read_optional(&dir.join(EVALS))?;
        if requirements.is_none() && evals.is_none() {
            continue;
        }
        let tests = index_tests(&dir)?;
        findings.extend(check_project(
            &project,
            prefixes.get(&project).map(String::as_str),
            requirements.as_deref(),
            evals.as_deref(),
            &tests,
            &settings,
        ));
    }
    Ok(findings)
}

/// Checks the settings manifest and the answers file, returning the
/// findings and the manifest's setting ids.
fn check_settings(root: &Path) -> Result<(Vec<Finding>, Vec<String>)> {
    let mut findings = Vec::new();
    let mut ids = Vec::new();
    if let Some(text) = read_optional(&root.join(SETTINGS))? {
        match parse_settings(&text) {
            Ok(settings) => ids = settings.into_iter().map(|s| s.id).collect(),
            Err(errors) => findings.extend(errors.into_iter().map(|e| Finding {
                code: None,
                path: SETTINGS.to_string(),
                line: e.line,
                message: e.message,
            })),
        }
    }
    if let Some(text) = read_optional(&root.join(SETTING_CHECKS))? {
        for (index, line) in text.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let message = match serde_json::from_str::<SettingCheck>(line) {
                Err(e) => format!("not an answer line: {e}"),
                Ok(check) if !ids.contains(&check.setting) => {
                    format!("setting `{}` is not in {SETTINGS}", check.setting)
                }
                Ok(_) => continue,
            };
            findings.push(Finding {
                code: None,
                path: SETTING_CHECKS.to_string(),
                line: Some(index + 1),
                message,
            });
        }
    }
    Ok((findings, ids))
}

/// Finds the repo root: the nearest ancestor of `start` holding `naming.md`.
pub fn find_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(NAMING).is_file())
        .map(Path::to_path_buf)
}

pub fn read_optional(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

pub fn top_level_dirs(root: &Path) -> Result<Vec<String>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("listing {}", root.display()))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !name.starts_with('.') && entry.file_type()?.is_dir() {
            dirs.push(name);
        }
    }
    dirs.sort();
    Ok(dirs)
}

/// Directories never searched for tests.
const SKIP_DIRS: [&str; 5] = [
    "target",
    "node_modules",
    "dist",
    "test-results",
    "playwright-report",
];

/// Indexes the tests under a project directory.
pub fn index_tests(dir: &Path) -> Result<TestIndex> {
    let mut index = TestIndex::default();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in
            fs::read_dir(&current).with_context(|| format!("listing {}", current.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                if !name.starts_with('.') && !SKIP_DIRS.contains(&name.as_str()) {
                    stack.push(path);
                }
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let relative = path
                .strip_prefix(dir)
                .unwrap_or(&path)
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            index.files.push(relative);
            match path.extension().and_then(|e| e.to_str()) {
                Some("rs") => {
                    if let Ok(text) = fs::read_to_string(&path) {
                        index.rust_fns.extend(rust_fn_names(&text));
                    }
                }
                Some("ts" | "js" | "mjs") => {
                    if let Ok(text) = fs::read_to_string(&path) {
                        index.script_sources.push(text);
                    }
                }
                _ => {}
            }
        }
    }
    index.files.sort();
    Ok(index)
}

/// The names following `fn ` in Rust source.
fn rust_fn_names(text: &str) -> Vec<String> {
    text.split("fn ")
        .skip(1)
        .filter_map(|rest| {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            (!name.is_empty()).then_some(name)
        })
        .collect()
}

/// True if `pattern` (with `*` matching within one path segment) matches the
/// whole of `path` or a tail of it that starts at a segment boundary, so
/// `tests/cmd/COOK-001-*.toml` finds `cli/tests/cmd/COOK-001-help.toml`.
fn glob_matches_tail(pattern: &str, path: &str) -> bool {
    let pattern: Vec<&str> = pattern.split('/').collect();
    let path: Vec<&str> = path.split('/').collect();
    if pattern.len() > path.len() {
        return false;
    }
    let tail = &path[path.len() - pattern.len()..];
    pattern
        .iter()
        .zip(tail)
        .all(|(p, s)| segment_matches(p.as_bytes(), s.as_bytes()))
}

fn segment_matches(pattern: &[u8], text: &[u8]) -> bool {
    match pattern.split_first() {
        None => text.is_empty(),
        Some((b'*', rest)) => (0..=text.len()).any(|i| segment_matches(rest, &text[i..])),
        Some((c, rest)) => text
            .split_first()
            .is_some_and(|(t, text_rest)| t == c && segment_matches(rest, text_rest)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REQS: &str = "\
# Cookbook requirements

## COOK-001: Readable on a phone

- Priority: must
- Why: w
- Accept: a

## COOK-002: Searchable

- Priority: should
- Why: w
- Accept: a

## COOK-003: Printable

- Priority: must
- Why: w
- Accept: a
- Status: retired (2026-12-01, nobody prints)
";

    const EVALS_OK: &str = r#"
prefix = "COOK"

[[eval]]
id = "layout"
requirement = "COOK-001"
stages = ["ci", "cd"]
kind = "playwright"

[[eval]]
id = "search"
requirement = "COOK-002"
stages = ["ci"]
kind = "cargo"
"#;

    fn tests_index() -> TestIndex {
        TestIndex {
            rust_fns: vec!["cook_002_finds_leeks".into()],
            script_sources: vec!["test('layout @smoke @req:COOK-001', async () => {})".into()],
            files: vec!["cli/tests/cmd/COOK-002-help.toml".into()],
        }
    }

    fn check(reqs: Option<&str>, evals: Option<&str>) -> Vec<Finding> {
        check_project(
            "cookbook",
            Some("COOK"),
            reqs,
            evals,
            &tests_index(),
            &["auth0-cookbook-app".to_string()],
        )
    }

    fn codes(findings: &[Finding]) -> Vec<Option<&'static str>> {
        findings.iter().map(|f| f.code).collect()
    }

    #[test]
    fn a_clean_project_has_no_findings() {
        assert_eq!(check(Some(REQS), Some(EVALS_OK)), vec![]);
    }

    #[test]
    fn evl_001_unknown_or_retired_requirement() {
        let evals = format!(
            "{EVALS_OK}\n[[eval]]\nid = \"gone\"\nrequirement = \"COOK-009\"\nstages = [\"cd\"]\nkind = \"http\"\npath = \"/\"\nbody_contains = \"x\"\n\
             [[eval]]\nid = \"print\"\nrequirement = \"COOK-003\"\nstages = [\"cd\"]\nkind = \"http\"\npath = \"/print\"\nbody_contains = \"x\"\n"
        );
        let findings = check(Some(REQS), Some(&evals));
        assert_eq!(codes(&findings), vec![Some("EVL-001"), Some("EVL-001")]);
        assert!(findings[0].message.contains("COOK-009, which is not in"));
        assert!(findings[1].message.contains("retired"));
        assert_eq!(findings[0].path, "cookbook/evals.toml");
    }

    #[test]
    fn evl_002_duplicate_ids() {
        let reqs =
            format!("{REQS}\n## COOK-002: Again\n\n- Priority: could\n- Why: w\n- Accept: a\n");
        let evals = format!(
            "{EVALS_OK}\n[[eval]]\nid = \"layout\"\nrequirement = \"COOK-001\"\nstages = [\"cd\"]\nkind = \"http\"\npath = \"/\"\nbody_contains = \"x\"\n"
        );
        let findings = check(Some(&reqs), Some(&evals));
        assert_eq!(codes(&findings), vec![Some("EVL-002"), Some("EVL-002")]);
        assert!(
            findings[0]
                .message
                .contains("COOK-002 is used twice (first on line 9)")
        );
        assert_eq!(findings[0].line, Some(22));
        assert!(findings[1].message.contains("`layout` is used twice"));
    }

    #[test]
    fn evl_003_eval_matches_no_test() {
        let evals = r#"
prefix = "COOK"

[[eval]]
id = "layout"
requirement = "COOK-001"
stages = ["cd"]
kind = "playwright"
grep = "@req:COOK-001-missing"

[[eval]]
id = "units"
requirement = "COOK-002"
stages = ["ci"]
kind = "cargo"
filter = "cook_002_units"

[[eval]]
id = "help"
requirement = "COOK-002"
stages = ["ci"]
kind = "trycmd"

[[eval]]
id = "help-missing"
requirement = "COOK-001"
stages = ["ci"]
kind = "trycmd"
"#;
        let findings = check(Some(REQS), Some(evals));
        assert_eq!(
            codes(&findings),
            vec![Some("EVL-003"), Some("EVL-003"), Some("EVL-003")]
        );
        assert!(findings[0].message.contains("`@req:COOK-001-missing`"));
        assert!(findings[1].message.contains("`cook_002_units`"));
        assert!(findings[2].message.contains("`tests/cmd/COOK-001-*.toml`"));
    }

    #[test]
    fn evl_004_must_without_production_eval() {
        let evals = "prefix = \"COOK\"\n[[eval]]\nid = \"layout\"\nrequirement = \"COOK-001\"\nstages = [\"ci\"]\nkind = \"playwright\"\n";
        let findings = check(Some(REQS), Some(evals));
        assert_eq!(codes(&findings), vec![Some("EVL-004")]);
        assert_eq!(findings[0].path, "cookbook/docs/requirements.md");
        assert_eq!(findings[0].line, Some(3));

        // Without an evals.toml every active must is reported.
        assert_eq!(codes(&check(Some(REQS), None)), vec![Some("EVL-004")]);
    }

    #[test]
    fn evl_006_http_eval_without_a_content_check() {
        let base = "prefix = \"COOK\"\n[[eval]]\nid = \"live\"\nrequirement = \"COOK-001\"\nstages = [\"cd\"]\nkind = \"http\"\n";
        let findings = check(Some(REQS), Some(&format!("{base}path = \"/recipe/1\"\n")));
        assert_eq!(codes(&findings), vec![Some("EVL-006")]);
        assert!(findings[0].message.contains("`live`"));

        for fields in [
            "path = \"/\"\nbody_contains = \"Cookbook\"",
            "path = \"/\"\nasset_contains = \"Pancakes\"",
            "path = \"/old\"\nstatus = 301",
            "path = \"/\"\nmethod = \"OPTIONS\"\nstatus = 204",
            "url = \"https://a/\"",
        ] {
            let findings = check(Some(REQS), Some(&format!("{base}{fields}\n")));
            assert_eq!(codes(&findings), vec![], "{fields}");
        }
    }

    #[test]
    fn evals_without_requirements_file() {
        let findings = check(None, Some(EVALS_OK));
        assert_eq!(codes(&findings), vec![Some("EVL-001"), Some("EVL-001")]);
    }

    #[test]
    fn malformed_files_are_reported_without_a_code() {
        let findings = check(Some("## COOK-1: Bad\n"), Some("prefix = 1\n"));
        assert_eq!(codes(&findings), vec![None, None]);
        assert_eq!(findings[0].line, Some(1));
        assert_eq!(
            findings[0].to_string(),
            format!(
                "warning cookbook/docs/requirements.md:1: {}",
                findings[0].message
            )
        );
    }

    #[test]
    fn prefixes_must_match_naming_md() {
        let findings = check(None, Some("prefix = \"PANE\"\n"));
        assert!(
            findings[0]
                .message
                .contains("does not match cookbook's prefix COOK")
        );

        let reqs = "## PANE-001: T\n- Priority: could\n- Why: w\n- Accept: a\n";
        let findings = check(Some(reqs), None);
        assert!(
            findings[0]
                .message
                .contains("PANE-001 does not use cookbook's prefix COOK")
        );

        let findings = check_project("newproj", None, None, Some(EVALS_OK), &tests_index(), &[]);
        assert!(findings[0].message.contains("no requirement prefix"));
    }

    #[test]
    fn findings_display_with_code_and_line() {
        let finding = Finding {
            code: Some("EVL-004"),
            path: "cookbook/docs/requirements.md".into(),
            line: Some(3),
            message: "m".into(),
        };
        assert_eq!(
            finding.to_string(),
            "warning[EVL-004] cookbook/docs/requirements.md:3: m"
        );
    }

    #[test]
    fn reads_the_prefix_table() {
        let naming = "- rules\n\n## Requirement prefixes\n\nText.\n\n| Project | Prefix |\n| --- | --- |\n| cookbook | `COOK` |\n| controlled_chaos | `CHAOS` |\n\n## Other\n\n| x | `NOPE` |\n";
        let prefixes = parse_prefixes(naming);
        assert_eq!(prefixes.len(), 2);
        assert_eq!(prefixes.get("cookbook").map(String::as_str), Some("COOK"));
        assert_eq!(
            prefixes.get("controlled_chaos").map(String::as_str),
            Some("CHAOS")
        );
    }

    #[test]
    fn reads_the_real_naming_md() {
        let naming = include_str!("../../../standards/docs/naming.md");
        let prefixes = parse_prefixes(naming);
        assert_eq!(prefixes.get("evals").map(String::as_str), Some("EVAL"));
        assert_eq!(prefixes.get("cookbook").map(String::as_str), Some("COOK"));
    }

    #[test]
    fn finds_rust_function_names() {
        let names = rust_fn_names(
            "fn main() {}\n#[test]\nfn cook_001_layout() {}\npub(crate) fn helper<T>() {}",
        );
        assert_eq!(names, vec!["main", "cook_001_layout", "helper"]);
    }

    #[test]
    fn matches_globs_against_path_tails() {
        assert!(glob_matches_tail(
            "tests/cmd/COOK-001-*.toml",
            "tests/cmd/COOK-001-help.toml"
        ));
        assert!(glob_matches_tail(
            "tests/cmd/COOK-001-*.toml",
            "cli/tests/cmd/COOK-001-help.toml"
        ));
        assert!(!glob_matches_tail(
            "tests/cmd/COOK-001-*.toml",
            "tests/cmd/COOK-002-help.toml"
        ));
        assert!(!glob_matches_tail(
            "tests/cmd/COOK-001-*.toml",
            "xtests/cmd/COOK-001-help.toml"
        ));
        assert!(!glob_matches_tail("tests/cmd/*.toml", "cmd/a.toml"));
        assert!(!glob_matches_tail("tests/*.toml", "tests/sub/a.toml"));
    }

    #[test]
    fn checks_a_repo_on_disk() -> Result<()> {
        let root = tempfile::tempdir()?;
        let root = root.path();
        fs::create_dir_all(root.join("standards/docs"))?;
        fs::write(
            root.join(NAMING),
            "## Requirement prefixes\n\n| Project | Prefix |\n| --- | --- |\n| cookbook | `COOK` |\n",
        )?;
        fs::create_dir_all(root.join("cookbook/docs"))?;
        fs::create_dir_all(root.join("cookbook/web/tests"))?;
        fs::create_dir_all(root.join("cookbook/web/node_modules/x"))?;
        fs::create_dir_all(root.join("cookbook/core/src"))?;
        fs::write(root.join("cookbook").join(REQUIREMENTS), REQS)?;
        fs::write(root.join("cookbook").join(EVALS), EVALS_OK)?;
        fs::write(
            root.join("cookbook/web/tests/layout.spec.ts"),
            "test('layout @req:COOK-001', async () => {});",
        )?;
        fs::write(
            root.join("cookbook/core/src/lib.rs"),
            "#[test]\nfn cook_002_search() {}\n",
        )?;
        // A project with neither file is skipped.
        fs::create_dir_all(root.join("homer/src"))?;

        assert_eq!(check_repo(root)?, vec![]);
        assert_eq!(
            find_root(&root.join("cookbook/web/tests")).as_deref(),
            Some(root)
        );

        // Tests under node_modules don't count.
        fs::remove_file(root.join("cookbook/web/tests/layout.spec.ts"))?;
        fs::write(
            root.join("cookbook/web/node_modules/x/a.ts"),
            "test('@req:COOK-001')",
        )?;
        let findings = check_repo(root)?;
        assert_eq!(codes(&findings), vec![Some("EVL-003")]);
        Ok(())
    }

    #[test]
    fn evl_001_human_check_names_an_unknown_setting() {
        let evals = format!(
            "{EVALS_OK}\n[[eval]]\nid = \"callbacks\"\nrequirement = \"COOK-001\"\nstages = [\"human\"]\nkind = \"human-check\"\nsetting = \"auth0-cookbook-app\"\n\n[[eval]]\nid = \"gone\"\nrequirement = \"COOK-001\"\nstages = [\"human\"]\nkind = \"human-check\"\nsetting = \"no-such-setting\"\n"
        );
        let findings = check(Some(REQS), Some(&evals));
        assert_eq!(codes(&findings), vec![Some("EVL-001")], "{findings:?}");
        assert!(findings[0].message.contains("no-such-setting"));
    }

    #[test]
    fn checks_the_settings_manifest_and_answers() -> Result<()> {
        let root = tempfile::tempdir()?;
        let root = root.path();
        fs::create_dir_all(root.join("standards/docs"))?;
        fs::write(root.join(NAMING), "## Requirement prefixes\n")?;
        let entry = "id = \"a-setting\"\nvendor = \"v\"\nwhere = \"w\"\nexpect = \"e\"\nwhy = \"y\"\nevery = \"30d\"\n";
        fs::write(root.join(SETTINGS), format!("[[setting]]\n{entry}"))?;
        fs::write(
            root.join(SETTING_CHECKS),
            concat!(
                r#"{"setting":"a-setting","status":"pass","note":"ok","at":"2026-12-01T00:00:00Z","by":"jim"}"#,
                "\n\n",
                r#"{"setting":"other","status":"pass","at":"2026-12-01T00:00:00Z","by":"jim"}"#,
                "\nnot json\n",
            ),
        )?;
        let findings = check_repo(root)?;
        let lines: Vec<String> = findings.iter().map(ToString::to_string).collect();
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(
            lines[0].starts_with(
                "warning standards/settings-checks.jsonl:3: setting `other` is not in"
            )
        );
        assert!(
            lines[1].starts_with("warning standards/settings-checks.jsonl:4: not an answer line")
        );

        fs::write(
            root.join(SETTINGS),
            format!("[[setting]]\n{entry}value = \"secret\"\n"),
        )?;
        fs::remove_file(root.join(SETTING_CHECKS))?;
        let findings = check_repo(root)?;
        assert_eq!(findings.len(), 1);
        assert!(
            findings[0].message.contains("unknown field `value`"),
            "{findings:?}"
        );
        Ok(())
    }

    #[test]
    fn checks_the_real_settings_manifest() -> Result<()> {
        let root = find_root(Path::new(env!("CARGO_MANIFEST_DIR")))
            .ok_or_else(|| anyhow::anyhow!("no repo root"))?;
        let (findings, ids) = check_settings(&root)?;
        assert_eq!(findings, vec![]);
        assert!(!ids.is_empty());
        Ok(())
    }

    #[test]
    fn a_repo_without_naming_md_is_an_error() -> Result<()> {
        let root = tempfile::tempdir()?;
        assert!(check_repo(root.path()).is_err());
        assert_eq!(find_root(root.path()), None);
        Ok(())
    }
}
