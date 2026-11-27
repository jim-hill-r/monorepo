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

fn report(dir: &Path, out: &Path) {
    let out = out.to_string_lossy().to_string();
    let output = run(
        env!("CARGO_BIN_EXE_royalty"),
        &["report", "--period", "2026-11", "--out", &out],
        dir,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn royl_006_the_same_inputs_give_the_same_report_bytes() {
    let dir = fixtures();
    let acme = dir.path().join("acme_shop");
    commit_as(&acme, 14);
    let (a, b) = (dir.path().join("a"), dir.path().join("b"));
    report(&acme, &a);
    report(&acme, &b);
    for name in ["royalty-report.json", "royalty-report.md"] {
        let first = fs::read(a.join(name)).unwrap_or_default();
        assert!(!first.is_empty(), "{name} is empty");
        assert_eq!(first, fs::read(b.join(name)).unwrap_or_default(), "{name}");
    }
}

#[test]
fn royl_005_the_report_holds_counts_and_amounts_only() {
    let dir = fixtures();
    let acme = dir.path().join("acme_shop");
    commit_as(&acme, 14);
    report(&acme, &acme);
    let json = fs::read_to_string(acme.join("royalty-report.json")).unwrap_or_default();
    let md = fs::read_to_string(acme.join("royalty-report.md")).unwrap_or_default();
    let temp = dir.path().to_string_lossy().to_string();
    for text in [&json, &md] {
        assert!(!text.contains(&temp), "a path leaked:\n{text}");
        assert!(!text.contains("acme.example"), "an email leaked:\n{text}");
        assert!(!text.contains("Person 1"), "a name leaked:\n{text}");
        assert!(
            !text.contains("Acme Example Ltd"),
            "the consumer's name leaked:\n{text}"
        );
    }
}

#[test]
fn royl_004_the_report_shows_every_cap_and_how_to_pay() {
    let dir = fixtures();
    let acme = dir.path().join("acme_shop");
    commit_as(&acme, 14);
    report(&acme, &acme);
    let md = fs::read_to_string(acme.join("royalty-report.md")).unwrap_or_default();
    assert!(
        md.contains("**Total owed: $45.00** of a $45.00 budget."),
        "{md}"
    );
    assert!(
        md.contains("| `chartlib` | 0.4.2 | direct | team | 14 seat | $70.00 | $40.00 | $25.00 | Set by the consumer's total budget. |"),
        "{md}"
    );
    assert!(
        md.contains("| `webframe` | 2.1.0 | direct | production | 2 deployment | $20.00 | $20.00 | $20.00 | Binding `agreement`: never scaled. Note: the producer offers a 90-day trial"),
        "{md}"
    );
    assert!(
        md.contains("| `tinycolor` | 1.0.3 | transitive | not billable (transitive) |"),
        "{md}"
    );
    assert!(
        md.contains("  - Chartlib maintainers: $25.00 by github-sponsors: https://github.com/sponsors/chartlib-example"),
        "{md}"
    );
    let json = fs::read_to_string(acme.join("royalty-report.json")).unwrap_or_default();
    let value: serde_json::Value = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(e) => panic!("{e}"),
    };
    assert_eq!(value["format"], 1);
    assert_eq!(value["period"], "2026-11");
    assert_eq!(value["total_cents"], 4500);
    let chartlib = &value["families"][0];
    assert_eq!(chartlib["family"], "chartlib");
    assert_eq!(chartlib["steps"][0]["limit"], "the producer's project cap");
    assert_eq!(chartlib["steps"][0]["cents"], 4000);
    assert_eq!(chartlib["steps"][1]["cents"], 2500);
}

fn check(dir: &Path) -> Output {
    run(
        env!("CARGO_BIN_EXE_royalty"),
        &["check", "--period", "2026-11"],
        dir,
    )
}

fn edit(path: &Path, from: &str, to: &str) {
    let text = fs::read_to_string(path).unwrap_or_default();
    assert!(text.contains(from), "{} has no {from:?}", path.display());
    if fs::write(path, text.replace(from, to)).is_err() {
        panic!("can't write {}", path.display());
    }
}

#[test]
fn royl_004_check_passes_within_budget() {
    let dir = fixtures();
    let acme = dir.path().join("acme_shop");
    commit_as(&acme, 14);
    let output = check(&acme);
    let out = stdout_of(&output);
    assert!(output.status.success(), "{out}");
    assert!(
        out.contains("royalty check: 2026-11: $45.00 owed of a $45.00 budget"),
        "{out}"
    );
    assert!(out.contains("ok (within budget)"), "{out}");
}

#[test]
fn royl_004_check_fails_when_an_agreement_breaks_the_budget() {
    let dir = fixtures();
    let acme = dir.path().join("acme_shop");
    commit_as(&acme, 14);
    // webframe now asks $60 a deployment: $60 is over $45 and over the
    // $40 per-producer limit, and an agreement is never scaled.
    // Both packages of the family ship the terms; edit both.
    for package in ["webframe", "webframe_router"] {
        edit(
            &dir.path().join(format!("producers/{package}/royalty.toml")),
            "cents = 2000",
            "cents = 6000",
        );
    }
    let output = check(&acme);
    let out = stdout_of(&output);
    assert_eq!(output.status.code(), Some(1), "{out}");
    assert!(
        out.contains("error[RTOML-014] ./royalty.toml: webframe owes $60.00, over per_producer_period_cents ($40.00)"),
        "{out}"
    );
    assert!(
        out.contains("the total owed, $60.00, is over the budget ($45.00)"),
        "{out}"
    );
}

#[test]
fn royl_004_check_fails_on_fail_mode_and_on_a_refused_binding() {
    let dir = fixtures();
    let acme = dir.path().join("acme_shop");
    commit_as(&acme, 14);
    let consumer = acme.join("royalty.toml");
    edit(&consumer, "on_exceed = \"scale\"", "on_exceed = \"fail\"");
    let out = stdout_of(&check(&acme));
    assert!(
        out.contains("the total owed, $60.00, is over the budget ($45.00) and on_exceed is \"fail\": chartlib $40.00, webframe $20.00"),
        "{out}"
    );
    edit(
        &consumer,
        "bindings = [\"voluntary\", \"agreement\"]",
        "bindings = [\"voluntary\"]",
    );
    let output = check(&acme);
    let out = stdout_of(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        out.contains("webframe asks for payment under binding \"agreement\", which [consumer.policy] bindings (voluntary) does not accept"),
        "{out}"
    );
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn the_committed_acme_example_report_is_current() {
    // docs/examples/acme_shop holds this run's report. Regenerate it with
    // ROYALTY_BLESS=1 cargo test after a change to the report's format.
    let dir = fixtures();
    let acme = dir.path().join("acme_shop");
    commit_as(&acme, 14);
    let out = dir.path().join("report");
    report(&acme, &out);
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/examples/acme_shop");
    for name in ["royalty-report.json", "royalty-report.md"] {
        let fresh = fs::read_to_string(out.join(name)).unwrap_or_default();
        if std::env::var_os("ROYALTY_BLESS").is_some() {
            let _ = fs::create_dir_all(&examples);
            let _ = fs::write(examples.join(name), &fresh);
        }
        let committed = fs::read_to_string(examples.join(name)).unwrap_or_default();
        assert_eq!(
            committed, fresh,
            "docs/examples/acme_shop/{name} is out of date; run ROYALTY_BLESS=1 cargo test"
        );
    }
}
