//! Turning clippy's JSON messages and file lengths into findings: one per
//! lint, file, line and column, inside one project, with generated code left
//! out (QUAL-407).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The lint name of a file over the length limit (QUAL-206).
pub const FILE_TOO_LONG: &str = "jimhillr::file_too_long";

/// One quality finding. Paths are from the repo root, with `/`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Finding {
    pub lint: String,
    pub file: String,
    pub line: u64,
    pub column: u64,
}

/// Where findings are counted: the project folder and the paths to leave out.
#[derive(Debug, Clone)]
pub struct Scope<'a> {
    /// The repo root, absolute.
    pub root: &'a Path,
    /// The project folder from the repo root, such as `cookbook`.
    pub project: &'a str,
    /// Generated paths from the repo root; a folder ends with `/`.
    pub generated: &'a [String],
}

impl Scope<'_> {
    /// The path from the repo root, when it is inside the project and not
    /// generated.
    fn counts(&self, path: &Path) -> Option<String> {
        let rel = normalize(path).strip_prefix(self.root).ok()?.to_path_buf();
        let rel = rel.to_str()?.replace('\\', "/");
        let in_project = self.project == "."
            || rel == self.project
            || rel.starts_with(&format!("{}/", self.project));
        (in_project && !is_generated(&rel, self.generated)).then_some(rel)
    }
}

/// Whether a repo-relative path is generated code.
pub fn is_generated(rel: &str, generated: &[String]) -> bool {
    generated.iter().any(|g| {
        if g.ends_with('/') {
            rel.starts_with(g.as_str())
        } else {
            rel == g
        }
    })
}

#[derive(Deserialize)]
struct CargoMessage {
    reason: String,
    #[serde(default)]
    manifest_path: Option<PathBuf>,
    #[serde(default)]
    message: Option<Diagnostic>,
}

#[derive(Deserialize)]
struct Diagnostic {
    level: String,
    code: Option<Code>,
    #[serde(default)]
    spans: Vec<Span>,
}

#[derive(Deserialize)]
struct Code {
    code: String,
}

#[derive(Deserialize)]
struct Span {
    file_name: String,
    line_start: u64,
    column_start: u64,
    is_primary: bool,
}

/// The findings in `cargo clippy --message-format=json` output that ran in
/// `run_dir`. A message counts once even when several targets (lib and
/// tests) report it. Lines that are not compiler messages are skipped.
pub fn from_clippy(json: &str, run_dir: &Path, scope: &Scope) -> BTreeSet<Finding> {
    json.lines()
        .filter_map(|line| serde_json::from_str::<CargoMessage>(line).ok())
        .filter(|m| m.reason == "compiler-message")
        .filter_map(|m| finding(&m, run_dir, scope))
        .collect()
}

fn finding(msg: &CargoMessage, run_dir: &Path, scope: &Scope) -> Option<Finding> {
    let diag = msg.message.as_ref()?;
    if diag.level != "warning" && diag.level != "error" {
        return None;
    }
    let lint = diag.code.as_ref()?.code.clone();
    let span = diag.spans.iter().find(|s| s.is_primary)?;
    let path = resolve(run_dir, msg.manifest_path.as_deref(), &span.file_name);
    Some(Finding {
        lint,
        file: scope.counts(&path)?,
        line: span.line_start,
        column: span.column_start,
    })
}

/// Rustc names files relative to the workspace root it ran in, or, for a
/// path dependency outside it, relative to that package's own folder.
fn resolve(run_dir: &Path, manifest: Option<&Path>, file_name: &str) -> PathBuf {
    let file = Path::new(file_name);
    if file.is_absolute() {
        return file.to_path_buf();
    }
    let package_dir = manifest.and_then(Path::parent);
    match package_dir {
        Some(dir) if !normalize(dir).starts_with(normalize(run_dir)) => dir.join(file),
        _ => run_dir.join(file),
    }
}

/// Removes `.` and folds `..` without touching the file system.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other),
        }
    }
    out
}

/// One `jimhillr::file_too_long` finding for each file longer than
/// `max_lines`. `files` maps repo-relative paths to their line counts.
pub fn long_files(
    files: &BTreeMap<String, usize>,
    max_lines: usize,
    scope: &Scope,
) -> BTreeSet<Finding> {
    files
        .iter()
        .filter(|(_, lines)| **lines > max_lines)
        .filter_map(|(file, lines)| {
            let file = scope.counts(&scope.root.join(file))?;
            Some(Finding {
                lint: FILE_TOO_LONG.to_string(),
                file,
                line: u64::try_from(*lines).unwrap_or(u64::MAX),
                column: 1,
            })
        })
        .collect()
}

/// Findings counted per lint.
pub fn counts<'a>(findings: impl IntoIterator<Item = &'a Finding>) -> BTreeMap<String, u64> {
    let mut out = BTreeMap::new();
    for f in findings {
        *out.entry(f.lint.clone()).or_insert(0) += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "/repo";

    fn scope<'a>(project: &'a str, generated: &'a [String]) -> Scope<'a> {
        Scope {
            root: Path::new(ROOT),
            project,
            generated,
        }
    }

    fn message(manifest: &str, file: &str, line: u64, code: Option<&str>, level: &str) -> String {
        serde_json::json!({
            "reason": "compiler-message",
            "package_id": "path+file:///x",
            "manifest_path": manifest,
            "target": {"kind": ["lib"], "src_path": "/x"},
            "message": {
                "level": level,
                "code": code.map(|c| serde_json::json!({"code": c, "explanation": null})),
                "rendered": "warning: ...",
                "spans": [
                    {"file_name": "src/other.rs", "line_start": 1, "column_start": 1, "is_primary": false},
                    {"file_name": file, "line_start": line, "column_start": 5, "is_primary": true}
                ]
            }
        })
        .to_string()
    }

    #[test]
    fn workspace_members_are_relative_to_the_run_dir() {
        let json = [
            message(
                "/repo/cookbook/core/Cargo.toml",
                "core/src/lib.rs",
                10,
                Some("clippy::doc_markdown"),
                "warning",
            ),
            r#"{"reason":"build-finished","success":true}"#.to_string(),
        ]
        .join("\n");
        let found = from_clippy(&json, Path::new("/repo/cookbook"), &scope("cookbook", &[]));
        let found: Vec<_> = found.into_iter().collect();
        assert_eq!(
            found,
            vec![Finding {
                lint: "clippy::doc_markdown".into(),
                file: "cookbook/core/src/lib.rs".into(),
                line: 10,
                column: 5,
            }]
        );
    }

    #[test]
    fn lib_and_test_targets_count_once() {
        let one = message(
            "/repo/cost/Cargo.toml",
            "src/lib.rs",
            3,
            Some("clippy::too_many_lines"),
            "warning",
        );
        let json = format!("{one}\n{one}\n");
        let found = from_clippy(&json, Path::new("/repo/cost"), &scope("cost", &[]));
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn path_dependencies_outside_the_project_are_left_out() {
        // auth_sdk is built as cookbook's path dependency; rustc names its
        // files relative to auth_sdk itself.
        let json = message(
            "/repo/auth_sdk/Cargo.toml",
            "src/lib.rs",
            3,
            Some("clippy::doc_markdown"),
            "warning",
        );
        let found = from_clippy(&json, Path::new("/repo/cookbook"), &scope("cookbook", &[]));
        assert!(found.is_empty());
        // From auth_sdk's own view the same message counts.
        let found = from_clippy(&json, Path::new("/repo/cookbook"), &scope("auth_sdk", &[]));
        assert_eq!(
            found.iter().next().map(|f| f.file.as_str()),
            Some("auth_sdk/src/lib.rs")
        );
    }

    #[test]
    fn registry_crates_and_relative_escapes_are_left_out() {
        let registry = message(
            "/home/u/.cargo/registry/x/Cargo.toml",
            "/home/u/.cargo/registry/x/src/lib.rs",
            1,
            Some("clippy::x"),
            "warning",
        );
        let escape = message(
            "/repo/cookbook/Cargo.toml",
            "../pane/src/lib.rs",
            1,
            Some("clippy::x"),
            "warning",
        );
        let json = format!("{registry}\n{escape}");
        assert!(
            from_clippy(&json, Path::new("/repo/cookbook"), &scope("cookbook", &[])).is_empty()
        );
    }

    #[test]
    fn messages_without_a_code_or_below_warning_are_skipped() {
        let json = [
            message("/repo/cost/Cargo.toml", "src/lib.rs", 1, None, "warning"),
            message(
                "/repo/cost/Cargo.toml",
                "src/lib.rs",
                2,
                Some("clippy::x"),
                "note",
            ),
            message(
                "/repo/cost/Cargo.toml",
                "src/lib.rs",
                3,
                Some("dead_code"),
                "warning",
            ),
            "not json".to_string(),
        ]
        .join("\n");
        let found = from_clippy(&json, Path::new("/repo/cost"), &scope("cost", &[]));
        let lints: Vec<_> = found.iter().map(|f| f.lint.as_str()).collect();
        assert_eq!(lints, ["dead_code"]);
    }

    #[test]
    fn generated_code_is_left_out() {
        let generated = vec![
            "infra/src/tfschema/".to_string(),
            "infra/src/gen.rs".to_string(),
        ];
        let json = [
            message(
                "/repo/infra/Cargo.toml",
                "src/tfschema/github/mod.rs",
                1,
                Some("clippy::x"),
                "warning",
            ),
            message(
                "/repo/infra/Cargo.toml",
                "src/gen.rs",
                1,
                Some("clippy::x"),
                "warning",
            ),
            message(
                "/repo/infra/Cargo.toml",
                "src/main.rs",
                1,
                Some("clippy::x"),
                "warning",
            ),
        ]
        .join("\n");
        let found = from_clippy(&json, Path::new("/repo/infra"), &scope("infra", &generated));
        let files: Vec<_> = found.iter().map(|f| f.file.as_str()).collect();
        assert_eq!(files, ["infra/src/main.rs"]);
    }

    #[test]
    fn a_project_is_matched_by_whole_folder_names() {
        let json = message(
            "/repo/cost_extra/Cargo.toml",
            "src/lib.rs",
            1,
            Some("clippy::x"),
            "warning",
        );
        assert!(from_clippy(&json, Path::new("/repo/cost_extra"), &scope("cost", &[])).is_empty());
    }

    #[test]
    fn long_files_over_the_limit_only() {
        let files = BTreeMap::from([
            ("cookbook/web/src/main.rs".to_string(), 2054),
            ("cookbook/web/src/ok.rs".to_string(), 800),
            ("pane/src/main.rs".to_string(), 900),
        ]);
        let found = long_files(&files, 800, &scope("cookbook", &[]));
        let found: Vec<_> = found.into_iter().collect();
        assert_eq!(
            found,
            vec![Finding {
                lint: FILE_TOO_LONG.into(),
                file: "cookbook/web/src/main.rs".into(),
                line: 2054,
                column: 1,
            }]
        );
    }

    #[test]
    fn long_generated_files_are_left_out() {
        let generated = vec!["infra/src/tfschema/".to_string()];
        let files = BTreeMap::from([("infra/src/tfschema/cloudflare/mod.rs".to_string(), 90_000)]);
        assert!(long_files(&files, 800, &scope("infra", &generated)).is_empty());
    }

    #[test]
    fn counts_per_lint() {
        let f = |lint: &str, line| Finding {
            lint: lint.into(),
            file: "a.rs".into(),
            line,
            column: 1,
        };
        let found = [f("clippy::a", 1), f("clippy::a", 2), f("clippy::b", 1)];
        assert_eq!(
            counts(&found),
            BTreeMap::from([("clippy::a".to_string(), 2), ("clippy::b".to_string(), 1)])
        );
    }
}
