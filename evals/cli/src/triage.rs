//! `evals triage`: turn the requirements that are not green into issue files,
//! so the normal agent loop works them. See "Continuous improvement" and
//! "Triage" in `standards/docs/evals.md`.
//!
//! It files work for a requirement that is not met, at risk, or a `must`
//! still unknown (no production result), unless an open issue file already
//! names the requirement's ID. A requirement whose only problem is a
//! hand-made setting that Jim hasn't checked is left to the human checks
//! (`evals human-checks due`), since no agent can do that.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use evals_model::{EvalResult, Priority, ResultStatus, Stage, Verdict};

use crate::store::{RequirementVerdict, VerdictsFile};

/// The epic triage issues are filed under.
pub const EPIC: &str = "Eval triage";

/// The reasons (from `evals_model::verdict`) that only a person can clear:
/// an unchecked or stale human check of a hand-made setting.
const HUMAN_REASONS: [&str; 2] = ["has never been checked", "was last checked"];

/// The reason for a requirement with evals but no production result yet.
const NO_PRODUCTION_RESULTS: &str = "no production-stage results yet";

/// What kind of work a requirement needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Work {
    /// The product fails it in production: a `fix`.
    Fix,
    /// An eval is broken, flaky, stale or failing in CI: a `todo`.
    Eval,
    /// A `must` with no production evidence: a `todo` to add or run a
    /// production-stage eval.
    Unproven,
}

impl Work {
    fn kind(self) -> &'static str {
        match self {
            Work::Fix => "fix",
            Work::Eval | Work::Unproven => "todo",
        }
    }
}

/// One requirement that needs work, and what triage did about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// A new issue file was written (path relative to the repo root).
    Wrote(String),
    /// An open issue file already names the requirement.
    Open(String),
    /// Over the run's limit; the next run files it.
    Later,
    /// Waiting on Jim's human check of a setting.
    Human,
}

/// One line of the triage report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub project: String,
    pub requirement: String,
    pub title: String,
    pub verdict: Verdict,
    pub work: Work,
    pub outcome: Outcome,
}

impl Finding {
    pub fn line(&self) -> String {
        let what = match &self.outcome {
            Outcome::Wrote(path) => format!("wrote {path}"),
            Outcome::Open(path) => format!("open {path}"),
            Outcome::Later => "over the limit, next run".into(),
            Outcome::Human => "waiting on a human check".into(),
        };
        format!(
            "{} {} ({}, {}): {what}",
            self.requirement,
            self.title,
            verdict_word(self.verdict),
            work_word(self.work)
        )
    }
}

/// Decides which requirements need work, writes up to `limit` new issue
/// files when `write` is set, and returns one finding per requirement that
/// needs work, in `verdicts` order.
pub fn triage(
    root: &Path,
    verdicts: &VerdictsFile,
    results: &[EvalResult],
    now: DateTime<Utc>,
    write: Option<usize>,
) -> Result<Vec<Finding>> {
    let added = now.format("%Y-%m-%d").to_string();
    let mut left = write.unwrap_or(0);
    let mut findings = Vec::new();
    for project in &verdicts.projects {
        let open = open_issues(root, &project.project)?;
        for r in &project.requirements {
            let Some(work) = needs_work(r) else {
                continue;
            };
            let outcome = if human_only(r) {
                Outcome::Human
            } else if let Some(path) = open
                .iter()
                .find(|(_, text)| names(text, &r.requirement))
                .map(|(path, _)| path)
            {
                Outcome::Open(relative(root, path))
            } else {
                let rel = format!(
                    "{}/docs/issues/{}",
                    project.project,
                    file_name(&r.requirement, work)
                );
                if write.is_some() && left > 0 {
                    let path = root.join(&rel);
                    if let Some(dir) = path.parent() {
                        fs::create_dir_all(dir)
                            .with_context(|| format!("creating {}", dir.display()))?;
                    }
                    let mine: Vec<&EvalResult> = results
                        .iter()
                        .filter(|e| e.project == project.project && e.requirement == r.requirement)
                        .collect();
                    fs::write(&path, issue_file(r, work, &mine, &added))
                        .with_context(|| format!("writing {}", path.display()))?;
                    left -= 1;
                    Outcome::Wrote(rel)
                } else if write.is_some() {
                    Outcome::Later
                } else {
                    Outcome::Wrote(format!("{rel} (dry run)"))
                }
            };
            findings.push(Finding {
                project: project.project.clone(),
                requirement: r.requirement.clone(),
                title: r.title.clone(),
                verdict: r.verdict,
                work,
                outcome,
            });
        }
    }
    Ok(findings)
}

/// The work a requirement needs, or `None` when it is green (or a `should`
/// or `could` that simply has no production evidence yet).
pub fn needs_work(r: &RequirementVerdict) -> Option<Work> {
    match r.verdict {
        Verdict::Met => None,
        Verdict::NotMet => Some(Work::Fix),
        Verdict::AtRisk if r.reasons.iter().any(|x| production_failure(x)) => Some(Work::Fix),
        Verdict::AtRisk => Some(Work::Eval),
        Verdict::Unknown if r.priority == Priority::Must => {
            if r.reasons.iter().any(|x| x == "retired") {
                None
            } else {
                Some(Work::Unproven)
            }
        }
        Verdict::Unknown => None,
    }
}

/// A reason that says the product failed in production, or that Jim said so.
fn production_failure(reason: &str) -> bool {
    [" failed in cd ", " failed in chaos ", " failed in observe "]
        .iter()
        .any(|s| reason.contains(s))
        || reason.starts_with("Jim rated it not met")
        || reason.starts_with("Jim answered setting")
}

/// True when the reasons are a human check Jim hasn't done (or has let go
/// stale), so the work is his, through `evals human-checks due`. A
/// requirement checked only by a person also has no production results,
/// which isn't agent work either.
fn human_only(r: &RequirementVerdict) -> bool {
    let human = |x: &String| HUMAN_REASONS.iter().any(|h| x.contains(h));
    r.reasons.iter().any(human)
        && r.reasons
            .iter()
            .all(|x| human(x) || x == NO_PRODUCTION_RESULTS)
}

/// `eval-cook-005-not-met.md` and so on: one file per requirement and kind
/// of work, so a rerun never writes a second.
pub fn file_name(requirement: &str, work: Work) -> String {
    format!(
        "eval-{}-{}.md",
        requirement.to_ascii_lowercase(),
        match work {
            Work::Fix => "failing",
            Work::Eval => "eval-at-risk",
            Work::Unproven => "unproven",
        }
    )
}

/// The issue file: front matter, then what is wrong, the evidence and what
/// to do.
pub fn issue_file(
    r: &RequirementVerdict,
    work: Work,
    results: &[&EvalResult],
    added: &str,
) -> String {
    let priority = if work == Work::Fix || r.priority == Priority::Must {
        "priority"
    } else {
        "backlog"
    };
    let mut out = format!(
        "---\nkind: {}\npriority: {priority}\ntags: [agent-generated]\nepic: {EPIC}\nadded: {added}\n---\n\n",
        work.kind()
    );
    let headline = match work {
        Work::Fix => format!(
            "{} ({}) is {}: the product fails it in production.",
            r.requirement,
            r.title,
            verdict_word(r.verdict)
        ),
        Work::Eval => format!(
            "{} ({}) is at risk because of its evals, not a production failure.",
            r.requirement, r.title
        ),
        Work::Unproven => format!(
            "{} ({}) is a `must` with no production evidence, so its verdict is unknown.",
            r.requirement, r.title
        ),
    };
    out.push_str(&format!(
        "Eval triage: {headline} Filed by the daily `evals-triage.yml` run from `verdicts.json` on the `evals-results` branch.\n\n"
    ));
    out.push_str("## Reasons\n\n");
    for reason in &r.reasons {
        out.push_str(&format!("- {reason}\n"));
    }
    let evidence = latest_problems(results);
    if !evidence.is_empty() {
        out.push_str("\n## Latest failing results\n\n");
        for e in &evidence {
            out.push_str(&format!("- {e}\n"));
        }
    }
    out.push_str("\n## What to do\n\n");
    out.push_str(match work {
        Work::Fix => "Find why the product fails the requirement in production and fix it, adding a test that would have caught it. If a deploy caused it and the fix will take a while, roll the project back first: run Push Main CI/CD by hand at the last good commit above. Close this issue (delete the file) in the PR that makes the verdict green again; if the eval itself is wrong, fix the eval instead and say why.\n",
        Work::Eval => "Fix the eval, not the product: make an erroring eval run, make a flaky one reliable, or get a stale one running again (check the stage that should produce it). If the eval is wrong about the requirement, change it in the project's `evals.toml` and say why. Delete this file in the PR that makes the verdict green.\n",
        Work::Unproven => "Add a production-stage eval for it in the project's `evals.toml` (`cd` `http` with a `body_contains` check, a `smoke` Playwright test, or a `human-check`), or make the existing one produce results after a deploy. Format in `standards/docs/evals.md`. Delete this file in the PR that gives it a production verdict.\n",
    });
    out
}

/// The newest result of each (eval, stage) whose last result failed or
/// errored, with its evidence and the last commit it passed at.
fn latest_problems(results: &[&EvalResult]) -> Vec<String> {
    let mut sorted: Vec<&EvalResult> = results
        .iter()
        .copied()
        .filter(|r| r.status != ResultStatus::Skip)
        .collect();
    sorted.sort_by_key(|r| r.at);
    let mut keys: Vec<(&str, Stage)> = sorted.iter().map(|r| (r.eval.as_str(), r.stage)).collect();
    keys.sort_by_key(|(eval, stage)| (eval.to_string(), stage.as_str()));
    keys.dedup();
    let mut out = Vec::new();
    for (eval, stage) in keys {
        let runs: Vec<&&EvalResult> = sorted
            .iter()
            .filter(|r| r.eval == eval && r.stage == stage)
            .collect();
        let Some(last) = runs.last() else {
            continue;
        };
        if !matches!(last.status, ResultStatus::Fail | ResultStatus::Error) {
            continue;
        }
        let word = if last.status == ResultStatus::Fail {
            "failed"
        } else {
            "errored"
        };
        let mut line = format!(
            "`{eval}` {word} in {} at {} on commit {}",
            stage.as_str(),
            last.at.format("%Y-%m-%d %H:%M UTC"),
            short(&last.commit)
        );
        if let Some(note) = last.note.as_deref().filter(|n| !n.is_empty()) {
            line.push_str(&format!(": {note}"));
        }
        if let Some(link) = last.evidence.as_deref() {
            line.push_str(&format!(" ([evidence]({link}))"));
        }
        match runs.iter().rev().find(|r| r.status == ResultStatus::Pass) {
            Some(good) => line.push_str(&format!(
                ". Last passed on commit {} at {}",
                short(&good.commit),
                good.at.format("%Y-%m-%d %H:%M UTC")
            )),
            None => line.push_str(". It has never passed in this stage"),
        }
        line.push('.');
        out.push(line);
    }
    out
}

fn short(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}

/// Every open issue file of a project (any `docs/issues/` folder under it)
/// and of the repo-wide `standards/docs/issues/` (human tasks included),
/// with its text.
fn open_issues(root: &Path, project: &str) -> Result<Vec<(PathBuf, String)>> {
    let mut files = Vec::new();
    collect_issue_files(&root.join(project), false, &mut files)?;
    if project != "standards" {
        collect_issue_files(&root.join("standards/docs/issues"), true, &mut files)?;
    }
    let mut out = Vec::new();
    for path in files {
        let text =
            fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        out.push((path, text));
    }
    Ok(out)
}

/// Walks `dir` for `*.md` files inside a `docs/issues` folder, skipping build
/// output and dependencies. `inside` says `dir` is already in one.
fn collect_issue_files(dir: &Path, inside: bool, out: &mut Vec<PathBuf>) -> Result<()> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(());
    };
    let mut entries: Vec<_> = entries.collect::<std::io::Result<_>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if name.starts_with('.') || ["target", "node_modules", "dist"].contains(&name.as_str())
            {
                continue;
            }
            collect_issue_files(&path, inside || name == "issues", out)?;
        } else if inside && name.ends_with(".md") && name != "README.md" {
            out.push(path);
        }
    }
    Ok(())
}

/// True when `text` names `id` as a whole ID (`COOK-1` doesn't match
/// `COOK-12`), in any case.
fn names(text: &str, id: &str) -> bool {
    let text = text.to_ascii_uppercase();
    let id = id.to_ascii_uppercase();
    text.match_indices(&id).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + id.len()..].chars().next();
        !before.is_some_and(|c| c.is_ascii_alphanumeric())
            && !after.is_some_and(|c| c.is_ascii_digit())
    })
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn verdict_word(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Met => "met",
        Verdict::AtRisk => "at risk",
        Verdict::NotMet => "not met",
        Verdict::Unknown => "unknown",
    }
}

fn work_word(work: Work) -> &'static str {
    match work {
        Work::Fix => "product fix",
        Work::Eval => "eval fix",
        Work::Unproven => "needs a production eval",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::publish::parse_results;
    use crate::store::compute;
    use chrono::TimeZone;

    const REQUIREMENTS_MD: &str = "# Cookbook requirements

## COOK-001: A recipe is easy to cook from on a phone

- Priority: must
- Why: Jim cooks from his phone.
- Accept: no sideways scrolling.

## COOK-002: Sign-in works

- Priority: must
- Why: Jim saves favourites.
- Accept: signing in returns to the recipe.

## COOK-003: Shopping list

- Priority: must
- Why: Jim shops once a week.
- Accept: the week's ingredients in one list.

## COOK-004: Search

- Priority: should
- Why: Finding a recipe.
- Accept: typing a word finds it.

## COOK-005: Week plan

- Priority: must
- Why: Planning.
- Accept: seven days.

## COOK-012: Print

- Priority: should
- Why: Paper.
- Accept: one page.
";

    const EVALS_TOML: &str = r#"prefix = "COOK"

[[eval]]
id = "layout"
requirement = "COOK-001"
stages = ["ci", "cd"]
kind = "playwright"

[[eval]]
id = "auth0-callbacks"
requirement = "COOK-002"
stages = ["human"]
kind = "human-check"
setting = "auth0-app"

[[eval]]
id = "list"
requirement = "COOK-003"
stages = ["ci"]
kind = "cargo"

[[eval]]
id = "search"
requirement = "COOK-004"
stages = ["ci", "cd"]
kind = "playwright"

[[eval]]
id = "week"
requirement = "COOK-005"
stages = ["ci", "cd"]
kind = "playwright"

[[eval]]
id = "print"
requirement = "COOK-012"
stages = ["ci", "cd"]
kind = "playwright"
"#;

    const SETTINGS_TOML: &str = r#"[[setting]]
id = "auth0-app"
vendor = "Auth0"
where = "Applications"
expect = "The callback URL is listed."
why = "Sign-in fails without it."
every = "30d"
"#;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 12, 10, 12, 0, 0)
            .single()
            .unwrap_or_default()
    }

    fn line(req: &str, eval: &str, stage: &str, status: &str, at: &str, commit: &str) -> String {
        format!(
            r#"{{"project":"cookbook","requirement":"{req}","eval":"{eval}","stage":"{stage}","kind":"playwright","commit":"{commit}","status":"{status}","evidence":"https://github.com/jimhillr/monorepo/actions/runs/9","note":"expected the recipe title","at":"{at}","by":"ci"}}"#
        )
    }

    fn repo() -> Result<tempfile::TempDir> {
        let root = tempfile::tempdir()?;
        let p = root.path();
        fs::create_dir_all(p.join("cookbook/docs"))?;
        fs::create_dir_all(p.join("cookbook/web/docs/issues"))?;
        fs::create_dir_all(p.join("standards"))?;
        fs::create_dir_all(p.join("evals"))?;
        fs::write(p.join("cookbook/docs/requirements.md"), REQUIREMENTS_MD)?;
        fs::write(p.join("cookbook/evals.toml"), EVALS_TOML)?;
        fs::write(p.join("standards/settings.toml"), SETTINGS_TOML)?;
        fs::write(
            p.join("evals/verdicts.toml"),
            include_str!("../../verdicts.toml"),
        )?;
        // COOK-004 already has an issue, in a sub-project's folder.
        fs::write(
            p.join("cookbook/web/docs/issues/search-flaky.md"),
            "---\nkind: todo\npriority: backlog\nadded: 2026-12-01\n---\n\nMake the cook-004 search eval reliable.\n",
        )?;
        // Names COOK-012 only, which must not count for COOK-001.
        fs::write(
            p.join("cookbook/web/docs/issues/README.md"),
            "COOK-001 notes, not an issue.\n",
        )?;
        Ok(root)
    }

    fn results() -> Result<Vec<EvalResult>> {
        parse_results(
            &[
                // COOK-001: passed, then failed in cd: not met.
                line(
                    "COOK-001",
                    "layout",
                    "cd",
                    "pass",
                    "2026-12-09T09:00:00Z",
                    "aaaaaaaaaaaaaaaa",
                ),
                line(
                    "COOK-001",
                    "layout",
                    "cd",
                    "fail",
                    "2026-12-10T09:00:00Z",
                    "bbbbbbbbbbbbbbbb",
                ),
                line(
                    "COOK-001",
                    "layout",
                    "ci",
                    "pass",
                    "2026-12-10T08:00:00Z",
                    "bbbbbbbbbbbbbbbb",
                ),
                // COOK-003: ci only, so a must with no production result.
                line(
                    "COOK-003",
                    "list",
                    "ci",
                    "pass",
                    "2026-12-10T08:00:00Z",
                    "bbbbbbbbbbbbbbbb",
                ),
                // COOK-004: errors in cd: at risk (has an open issue).
                line(
                    "COOK-004",
                    "search",
                    "ci",
                    "pass",
                    "2026-12-10T08:00:00Z",
                    "bbbbbbbbbbbbbbbb",
                ),
                line(
                    "COOK-004",
                    "search",
                    "cd",
                    "error",
                    "2026-12-10T09:00:00Z",
                    "bbbbbbbbbbbbbbbb",
                ),
                // COOK-005: green.
                line(
                    "COOK-005",
                    "week",
                    "ci",
                    "pass",
                    "2026-12-10T08:00:00Z",
                    "bbbbbbbbbbbbbbbb",
                ),
                line(
                    "COOK-005",
                    "week",
                    "cd",
                    "pass",
                    "2026-12-10T09:00:00Z",
                    "bbbbbbbbbbbbbbbb",
                ),
                // COOK-012: a should failing in ci: at risk, eval work.
                line(
                    "COOK-012",
                    "print",
                    "cd",
                    "pass",
                    "2026-12-10T09:00:00Z",
                    "bbbbbbbbbbbbbbbb",
                ),
                line(
                    "COOK-012",
                    "print",
                    "ci",
                    "fail",
                    "2026-12-10T08:00:00Z",
                    "bbbbbbbbbbbbbbbb",
                ),
            ]
            .join("\n"),
        )
    }

    #[test]
    fn files_one_issue_per_requirement_that_is_not_green() -> Result<()> {
        let root = repo()?;
        let results = results()?;
        let verdicts = compute(root.path(), &results, None, now())?;
        let findings = triage(root.path(), &verdicts, &results, now(), Some(10))?;
        let lines: Vec<String> = findings.iter().map(Finding::line).collect();
        assert_eq!(
            lines,
            vec![
                "COOK-001 A recipe is easy to cook from on a phone (not met, product fix): wrote cookbook/docs/issues/eval-cook-001-failing.md",
                "COOK-002 Sign-in works (at risk, eval fix): waiting on a human check",
                "COOK-003 Shopping list (unknown, needs a production eval): wrote cookbook/docs/issues/eval-cook-003-unproven.md",
                "COOK-004 Search (at risk, eval fix): open cookbook/web/docs/issues/search-flaky.md",
                "COOK-012 Print (at risk, eval fix): wrote cookbook/docs/issues/eval-cook-012-eval-at-risk.md",
            ],
        );

        let fix = fs::read_to_string(
            root.path()
                .join("cookbook/docs/issues/eval-cook-001-failing.md"),
        )?;
        assert!(
            fix.starts_with(
                "---\nkind: fix\npriority: priority\ntags: [agent-generated]\nepic: Eval triage\nadded: 2026-12-10\n---\n\nEval triage: COOK-001 (A recipe is easy to cook from on a phone) is not met"
            ),
            "{fix}"
        );
        assert!(
            fix.contains("- `layout` failed in cd at 2026-12-10 09:00 UTC on commit bbbbbbbbbbbb: expected the recipe title ([evidence](https://github.com/jimhillr/monorepo/actions/runs/9)). Last passed on commit aaaaaaaaaaaa at 2026-12-09 09:00 UTC."),
            "{fix}"
        );
        let unproven = fs::read_to_string(
            root.path()
                .join("cookbook/docs/issues/eval-cook-003-unproven.md"),
        )?;
        assert!(unproven.starts_with("---\nkind: todo\npriority: priority\n"));
        let eval = fs::read_to_string(
            root.path()
                .join("cookbook/docs/issues/eval-cook-012-eval-at-risk.md"),
        )?;
        assert!(eval.starts_with("---\nkind: todo\npriority: backlog\n"));
        assert!(eval.contains("It has never passed in this stage"), "{eval}");

        // The next run finds its own files and writes nothing new.
        let again = triage(root.path(), &verdicts, &results, now(), Some(10))?;
        assert!(
            again
                .iter()
                .all(|f| !matches!(f.outcome, Outcome::Wrote(_))),
            "{again:?}"
        );
        Ok(())
    }

    #[test]
    fn writes_up_to_the_limit_and_nothing_on_a_dry_run() -> Result<()> {
        let root = repo()?;
        let results = results()?;
        let verdicts = compute(root.path(), &results, None, now())?;
        let dry = triage(root.path(), &verdicts, &results, now(), None)?;
        assert!(dry[0].line().ends_with("(dry run)"));
        assert!(!root.path().join("cookbook/docs/issues").exists());

        let findings = triage(root.path(), &verdicts, &results, now(), Some(1))?;
        let wrote = findings
            .iter()
            .filter(|f| matches!(f.outcome, Outcome::Wrote(_)))
            .count();
        let later = findings
            .iter()
            .filter(|f| f.outcome == Outcome::Later)
            .count();
        assert_eq!((wrote, later), (1, 2));
        Ok(())
    }

    #[test]
    fn matches_whole_ids_only() {
        assert!(names("Fix cook-001 now", "COOK-001"));
        assert!(names("(COOK-001)", "COOK-001"));
        assert!(!names("COOK-0012", "COOK-001"));
        assert!(!names("XCOOK-001", "COOK-001"));
        assert!(!names("COOK-01", "COOK-001"));
    }

    #[test]
    fn human_reasons_match_the_verdict_rules() -> Result<()> {
        // Ties HUMAN_REASONS to the words evals_model::verdict uses.
        let root = repo()?;
        let verdicts = compute(root.path(), &[], None, now())?;
        let cook_002 = verdicts.projects[0]
            .requirements
            .iter()
            .find(|r| r.requirement == "COOK-002")
            .ok_or_else(|| anyhow::anyhow!("no COOK-002"))?;
        assert_eq!(cook_002.verdict, Verdict::AtRisk);
        assert!(
            cook_002
                .reasons
                .iter()
                .any(|r| HUMAN_REASONS.iter().any(|h| r.contains(h))),
            "{cook_002:?}"
        );
        Ok(())
    }
}
