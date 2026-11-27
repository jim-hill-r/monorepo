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

/// Makes the consumer a git repository with commits from `authors` people
/// in November 2026, plus a bot, and one person in October.
fn commit_as(dir: &Path, authors: usize) {
    let git = |args: &[&str], envs: &[(&str, String)]| {
        let output = match Command::new("git")
            .args(args)
            .envs(envs.iter().map(|(k, v)| (*k, v.as_str())))
            .current_dir(dir)
            .output()
        {
            Ok(output) => output,
            Err(error) => panic!("can't run git: {error}"),
        };
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
    git(&["init", "-q"], &[]);
    let mut people: Vec<(String, String, &str)> = (1..=authors)
        .map(|i| {
            (
                format!("Person {i}"),
                format!("person{i}@acme.example"),
                "2026-11-10T12:00:00+00:00",
            )
        })
        .collect();
    // The same person twice counts once; bots and October don't count.
    people.push((
        "Person 1".into(),
        "PERSON1@acme.example".into(),
        "2026-11-11T12:00:00+00:00",
    ));
    people.push((
        "dependabot[bot]".into(),
        "49699333+dependabot[bot]@users.noreply.github.com".into(),
        "2026-11-12T12:00:00+00:00",
    ));
    people.push((
        "Old Timer".into(),
        "old@acme.example".into(),
        "2026-10-31T23:00:00+00:00",
    ));
    for (i, (name, email, date)) in people.iter().enumerate() {
        let _ = fs::write(dir.join("CHANGES"), format!("{i}\n"));
        git(&["add", "-A"], &[]);
        git(
            &["commit", "-q", "-m", "change"],
            &[
                ("GIT_AUTHOR_NAME", name.clone()),
                ("GIT_AUTHOR_EMAIL", email.clone()),
                ("GIT_COMMITTER_NAME", name.clone()),
                ("GIT_COMMITTER_EMAIL", email.clone()),
                ("GIT_AUTHOR_DATE", date.to_string()),
                ("GIT_COMMITTER_DATE", date.to_string()),
            ],
        );
    }
}

fn scan(dir: &Path) -> String {
    let output = run(
        env!("CARGO_BIN_EXE_royalty"),
        &["scan", "--period", "2026-11"],
        dir,
    );
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
    // A direct dependency with terms; with no git history, seats are not
    // measured and add nothing.
    assert!(line(&out, "chartlib 0.4.2 (direct): team").ends_with("$0.00"));
    assert_eq!(line(&out, "seat:"), "seat: not measured");
    // Two packages of one family are billed once, on the newest version.
    assert!(
        line(&out, "webframe 2.1.0 (direct): production")
            .starts_with("webframe 2.1.0 (direct): production, $20.00")
    );
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

#[test]
fn royl_004_what_acme_owes_in_one_month() {
    // The table in docs/royalty-toml.md: 14 people committing, two
    // deployments, webframe's trial over.
    let dir = fixtures();
    let acme = dir.path().join("acme_shop");
    commit_as(&acme, 14);
    let out = scan(&acme);
    assert_eq!(
        line(&out, "chartlib"),
        "chartlib 0.4.2 (direct): team, $25.00 (priced at $70.00), set by the consumer's total budget",
        "{out}"
    );
    assert_eq!(
        line(&out, "webframe"),
        "webframe 2.1.0 (direct): production, $20.00"
    );
    assert_eq!(line(&out, "total:"), "total: $45.00 of a $45.00 budget");
}

#[test]
fn deployments_count_only_the_manifests_whose_graph_holds_the_family() {
    let dir = fixtures();
    let acme = dir.path().join("acme_shop");
    // Only the API deployment depends on chartlib; price chartlib by deployment.
    let chartlib = dir.path().join("producers/chartlib/royalty.toml");
    let text = fs::read_to_string(&chartlib).unwrap_or_default();
    let _ = fs::write(
        &chartlib,
        text.replace("unit = \"seat\"", "unit = \"deployment\""),
    );
    let out = scan(&acme);
    assert!(
        line(&out, "chartlib").starts_with("chartlib 0.4.2 (direct): team, $5.00"),
        "{out}"
    );

    // With no [[consumer.deployment]], a production project is one deployment.
    let consumer = acme.join("royalty.toml");
    let text = fs::read_to_string(&consumer).unwrap_or_default();
    let text = text.replace("[[consumer.deployment]]\nname = \"shop-worker\"\nmanifest = \"services/worker/Cargo.toml\"\n", "");
    let text = text.replace(
        "[[consumer.deployment]]\nname = \"shop-api\"\nmanifest = \"services/api/Cargo.toml\"\n",
        "",
    );
    let _ = fs::write(&consumer, text);
    let out = scan(&acme);
    // One deployment, and webframe includes the first one: $0.
    assert!(
        line(&out, "webframe").contains("production, $0.00"),
        "{out}"
    );
}

#[test]
fn royl_005_seats_never_print_emails() {
    let dir = fixtures();
    let acme = dir.path().join("acme_shop");
    commit_as(&acme, 3);
    let out = scan(&acme);
    assert!(!out.contains("acme.example"), "{out}");
    assert!(!out.contains("Person"), "{out}");
}
