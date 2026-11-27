//! `royalty scan` on the fixture consumer in tests/fixtures/acme_shop.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn copy_dir(from: &Path, to: &Path) {
    if fs::create_dir_all(to).is_err() {
        panic!("can't create {}", to.display());
    }
    let Ok(entries) = fs::read_dir(from) else {
        panic!("can't read {}", from.display());
    };
    for entry in entries.flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            if entry.file_name() != "target" {
                copy_dir(&entry.path(), &target);
            }
        } else if fs::copy(entry.path(), &target).is_err() {
            panic!("can't copy {}", entry.path().display());
        }
    }
}

fn run(program: &str, args: &[&str], dir: &Path) -> Output {
    match Command::new(program).args(args).current_dir(dir).output() {
        Ok(output) => output,
        Err(error) => panic!("can't run {program}: {error}"),
    }
}

/// The fixtures, copied to a temporary directory, with a Cargo.lock made
/// offline (every dependency is a path, so nothing is downloaded).
pub fn fixtures() -> tempfile::TempDir {
    let Ok(dir) = tempfile::tempdir() else {
        panic!("no temp dir");
    };
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    copy_dir(&from, dir.path());
    let lock = run(
        "cargo",
        &["generate-lockfile", "--offline"],
        &dir.path().join("acme_shop"),
    );
    assert!(
        lock.status.success(),
        "{}",
        String::from_utf8_lossy(&lock.stderr)
    );
    dir
}

fn scan(dir: &Path) -> String {
    let output = run(env!("CARGO_BIN_EXE_royalty"), &["scan"], dir);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn line<'a>(out: &'a str, start: &str) -> &'a str {
    match out.lines().find(|l| l.trim_start().starts_with(start)) {
        Some(line) => line.trim(),
        None => panic!("no line starting {start:?} in:\n{out}"),
    }
}

#[test]
fn the_scan_finds_families_reach_and_non_production_use() {
    let dir = fixtures();
    let out = scan(&dir.path().join("acme_shop"));
    // A direct dependency with terms.
    assert!(line(&out, "chartlib 0.4.2 (direct): team").ends_with("$0.00"));
    // Two packages of one family are billed once, on the newest version.
    assert!(line(&out, "webframe 2.1.0 (direct): production").ends_with("$0.00"));
    assert_eq!(
        line(&out, "packages:"),
        "packages: webframe@2.1.0, webframe_router@2.1.0"
    );
    // Reached only through webframe, and its reach is direct.
    assert_eq!(
        line(&out, "tinycolor"),
        "tinycolor 1.0.3 (transitive): not billable (transitive), $0.00"
    );
    // Only a dev-dependency: non-production, free whatever its terms say.
    assert_eq!(
        line(&out, "testkit"),
        "testkit 0.9.0 (transitive): non-production (free), $0.00"
    );
    // No file, and a file copied from another package.
    assert_eq!(
        line(&out, "no terms:"),
        "no terms: copycat@0.1.0, plainlib@1.2.0"
    );
    assert!(line(&out, "notice: copycat@0.1.0").contains("not this package; ignored"));
}

#[test]
fn royl_002_a_development_stage_consumer_owes_nothing() {
    let dir = fixtures();
    let consumer = dir.path().join("acme_shop/royalty.toml");
    let text = fs::read_to_string(&consumer).unwrap_or_default();
    if fs::write(
        &consumer,
        text.replace("stage = \"production\"", "stage = \"development\""),
    )
    .is_err()
    {
        panic!("can't write");
    }
    let out = scan(&dir.path().join("acme_shop"));
    assert!(
        line(&out, "webframe").contains("non-production (free), $0.00"),
        "{out}"
    );
    assert!(
        line(&out, "chartlib").contains("non-production (free), $0.00"),
        "{out}"
    );
    assert_eq!(line(&out, "total:"), "total: $0.00 of a $45.00 budget");
}

#[test]
fn royl_005_a_project_without_a_lockfile_is_refused_not_fetched() {
    let dir = fixtures();
    let consumer = dir.path().join("acme_shop");
    if fs::remove_file(consumer.join("Cargo.lock")).is_err() {
        panic!("no lockfile");
    }
    let output = run(env!("CARGO_BIN_EXE_royalty"), &["scan"], &consumer);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cargo metadata failed"));
}
