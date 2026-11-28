//! `repo::audit` reads the saved audit output into the snapshot.

use std::fs;
use std::path::PathBuf;

use pane_model::audit::Level;
use pane_model::{Project, ProjectStatus, Snapshot};
use pane_snapshot::repo;

type TestResult = Result<(), Box<dyn std::error::Error>>;

struct TempFile(PathBuf);

impl TempFile {
    fn new(name: &str, contents: &str) -> std::io::Result<Self> {
        let path =
            std::env::temp_dir().join(format!("pane_audit_{name}_{}.txt", std::process::id()));
        fs::write(&path, contents)?;
        Ok(Self(path))
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

fn snapshot() -> Snapshot {
    let mut s = Snapshot::new("abc", "2026-11-27T12:00:00Z");
    for path in ["app", "app/web"] {
        s.projects.push(Project {
            path: path.into(),
            kind: "rust".into(),
            deploy_url: None,
            summary: None,
            status: ProjectStatus::Active,
        });
    }
    s
}

#[test]
fn reads_findings_with_their_projects() -> TestResult {
    let file = TempFile::new(
        "findings",
        "warning[DOC-001] app has no README.md\n\
         error[STD-002] app/web/mise.toml must include exactly one task file\n\
         audit: 1 error(s)\n",
    )?;
    let audit = repo::audit(&file.0, &snapshot())?.ok_or("a finished audit")?;
    assert_eq!(audit.findings.len(), 2);
    assert_eq!(audit.findings[1].level, Level::Error);
    assert_eq!(audit.findings[1].project.as_deref(), Some("app/web"));
    assert_eq!(audit.errors("app/web").count(), 1);
    assert_eq!(audit.errors("app").count(), 0);
    Ok(())
}

#[test]
fn an_unfinished_audit_is_not_run() -> TestResult {
    let file = TempFile::new("unfinished", "error[STD-001] app has no mise.toml\n")?;
    assert_eq!(repo::audit(&file.0, &snapshot())?, None);
    Ok(())
}

#[test]
fn a_missing_file_fails() {
    let missing = std::env::temp_dir().join("pane_audit_missing_file.txt");
    assert!(repo::audit(&missing, &snapshot()).is_err());
}

#[test]
fn audit_sh_still_prints_the_format_pane_reads() -> TestResult {
    // pane_model::audit::parse reads `error[CODE] <subject> ...` lines and a
    // closing `audit: ok` or `audit: N error(s)`; keep audit.sh printing them.
    let script = fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../standards/scripts/audit.sh"),
    )?;
    for needle in [
        r#"err() { echo "error[$1] $2";"#,
        r#"echo "audit: $errors error(s)""#,
        r#"echo "audit: ok""#,
    ] {
        assert!(
            script.contains(needle),
            "audit.sh no longer prints {needle}"
        );
    }
    Ok(())
}
