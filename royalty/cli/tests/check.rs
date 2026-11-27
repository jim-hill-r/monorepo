//! `royalty check` end to end, through the built binary.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn royalty(args: &[&str], dir: &Path) -> Output {
    match Command::new(env!("CARGO_BIN_EXE_royalty"))
        .args(args)
        .current_dir(dir)
        .output()
    {
        Ok(output) => output,
        Err(error) => panic!("can't run royalty: {error}"),
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn dir_with(text: &str) -> tempfile::TempDir {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("no temp dir");
    };
    if fs::write(dir.path().join("royalty.toml"), text).is_err() {
        panic!("can't write");
    }
    dir
}

const PROJECT: &str = "[project]\nname = \"demo\"\ndescription = \"A demo.\"\n";

#[test]
fn royl_007_a_project_only_file_is_ok_with_no_payment_terms() {
    let dir = dir_with(PROJECT);
    let output = royalty(&["check"], dir.path());
    assert!(output.status.success());
    assert_eq!(
        stdout(&output),
        "royalty check: ./royalty.toml ok (no payment terms)\n"
    );
    // A path to the file works the same, and so does --strict.
    let output = royalty(&["check", "royalty.toml", "--strict"], dir.path());
    assert!(output.status.success(), "{}", stdout(&output));
}

#[test]
fn errors_exit_1_and_name_the_requirement() {
    let dir = dir_with(&format!(
        "{PROJECT}[[tier]]\nid = \"dev\"\nwhen = {{ environment = [\"staging\"] }}\nfree = true\n"
    ));
    let output = royalty(&["check", "."], dir.path());
    assert_eq!(output.status.code(), Some(1));
    let out = stdout(&output);
    assert!(
        out.contains("error[RTOML-008] ./royalty.toml tier[0].when.environment[0]: "),
        "{out}"
    );
    assert!(out.contains("(ROYL-002)"), "{out}");
}

#[test]
fn warnings_pass_unless_strict() {
    let dir = dir_with(&format!("{PROJECT}homepage = \"https://demo.example\"\n"));
    let output = royalty(&["check"], dir.path());
    assert!(output.status.success());
    assert!(stdout(&output).starts_with("warning[RTOML-003] ./royalty.toml project.homepage: "));
    let output = royalty(&["check", "--strict"], dir.path());
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn a_missing_file_exits_2() {
    let dir = dir_with(PROJECT);
    let output = royalty(&["check", "nope.toml"], dir.path());
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn royl_007_every_royalty_toml_in_the_repo_passes_strict() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for project in ["royalty", "standards", "auth_sdk", "homer"] {
        let output = royalty(&["check", project, "--strict"], &root);
        assert!(output.status.success(), "{project}: {}", stdout(&output));
    }
}
