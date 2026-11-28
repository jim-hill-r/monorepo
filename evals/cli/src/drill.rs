//! `evals drill`: the eval fire drill (standards/docs/evals.md#fire-drill).
//!
//! Proves the loop still catches a failure, without touching production or
//! the stored results: one `http` eval is pointed at text the live page
//! doesn't have, run against the site, and its failure is added, in memory
//! only, to the results read from a checkout of `evals-results`. The
//! requirement must turn red (amber for a `should` or `could`), triage must
//! say it needs work, and after the real eval runs and passes the verdict
//! must be back where it started. Nothing is written to disk.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use evals_model::{EvalResult, EvalsFile, Kind, ResultStatus, Stage, Verdict, parse_evals};

use crate::check;
use crate::run::{self, HttpRunner, Reports, RunContext};
use crate::store;
use crate::triage;

/// What to drill.
pub struct Drill<'a> {
    pub root: &'a Path,
    /// A checkout of the `evals-results` branch, read only.
    pub dir: &'a Path,
    pub project: &'a str,
    pub eval: &'a str,
    pub deploy_url: &'a str,
    pub now: DateTime<Utc>,
}

/// One checked step, for the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub name: &'static str,
    pub outcome: String,
}

/// The drill's report: the steps that ran, and the first one that didn't
/// hold, if any.
#[derive(Debug, Default)]
pub struct Report {
    pub requirement: String,
    pub steps: Vec<Step>,
    pub failure: Option<String>,
}

impl Report {
    fn step(&mut self, name: &'static str, outcome: impl Into<String>) {
        self.steps.push(Step {
            name,
            outcome: outcome.into(),
        });
    }

    /// The Markdown job summary.
    pub fn markdown(&self, drill: &Drill) -> String {
        let mut out = format!(
            "## Eval fire drill: {} {} ({})\n\nAgainst {}. Results stayed in memory; nothing was written or pushed.\n\n| Step | Outcome |\n| --- | --- |\n",
            drill.project, drill.eval, self.requirement, drill.deploy_url
        );
        for step in &self.steps {
            out.push_str(&format!("| {} | {} |\n", step.name, step.outcome));
        }
        match &self.failure {
            Some(why) => out.push_str(&format!("| Result | **Failed:** {why} |\n")),
            None => out.push_str(
                "| Result | **Passed:** red on a broken check, back to the start on the next good run |\n",
            ),
        }
        out
    }
}

/// The text the broken eval looks for. Never on any page.
pub fn missing_text(now: DateTime<Utc>) -> String {
    format!(
        "jimhillr-fire-drill-{}-not-on-the-page",
        now.format("%Y%m%d%H%M%S")
    )
}

/// Runs the drill. An `Err` means it couldn't run; a drill that ran and
/// found the loop broken returns a report with `failure` set.
pub fn run(drill: &Drill, runner: &HttpRunner) -> Result<Report> {
    let path = drill.root.join(drill.project).join(check::EVALS);
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let file = parse_evals(&text).map_err(|errors| {
        let list: Vec<String> = errors.iter().map(ToString::to_string).collect();
        anyhow!("{} is invalid: {}", path.display(), list.join("; "))
    })?;
    let Some(eval) = file.evals.iter().find(|e| e.id == drill.eval) else {
        bail!("{} has no eval {}", path.display(), drill.eval);
    };
    if !matches!(eval.kind, Kind::Http { .. }) {
        bail!(
            "{} is a {} eval; the drill needs an http one",
            drill.eval,
            eval.kind.name()
        );
    }

    let mut report = Report {
        requirement: eval.requirement.clone(),
        ..Report::default()
    };
    let mut results = store::read_results(drill.dir)?;

    // 1. Where the requirement starts.
    let baseline = verdict_of(drill, &results, &report.requirement)?;
    report.step("1. Baseline verdict", code(word(baseline)));

    // 2. Red run: the eval looks for text that isn't on the page.
    let mut broken = eval.clone();
    if let Kind::Http { body_contains, .. } = &mut broken.kind {
        *body_contains = Some(missing_text(drill.now));
    }
    let red = one_result(
        drill,
        &file.prefix,
        broken,
        "fire-drill-red",
        drill.now,
        runner,
    )?;
    report.step(
        "2. Broken eval's result",
        format!(
            "{}: {}",
            code(status_word(red.status)),
            red.note.as_deref().unwrap_or("")
        ),
    );
    if red.status != ResultStatus::Fail {
        report.failure = Some(format!(
            "the broken eval's result was {}, not fail",
            status_word(red.status)
        ));
        return Ok(report);
    }
    results.push(red);

    // 3. The failure turns the requirement red (amber for should/could).
    let red_verdict = verdict_of(drill, &results, &report.requirement)?;
    report.step("3. Verdict after the failure", code(word(red_verdict)));
    if !matches!(red_verdict, Verdict::NotMet | Verdict::AtRisk) {
        report.failure = Some(format!(
            "after the failure the verdict was {}, not not-met or at-risk",
            word(red_verdict)
        ));
        return Ok(report);
    }

    // 4. Triage (dry run) says it needs work.
    let verdicts = store::compute(drill.root, &results, Some(drill.project), drill.now)?;
    let findings = triage::triage(drill.root, &verdicts, &results, drill.now, None)?;
    let Some(finding) = findings
        .iter()
        .find(|f| f.requirement == report.requirement)
    else {
        report.step("4. Triage would file", "nothing");
        report.failure = Some(format!("triage didn't pick up {}", report.requirement));
        return Ok(report);
    };
    report.step("4. Triage would file", finding.line());

    // 5. Green run: the real eval passes and the verdict recovers.
    let green = one_result(
        drill,
        &file.prefix,
        eval.clone(),
        "fire-drill-green",
        drill.now + chrono::Duration::seconds(1),
        runner,
    )?;
    let green_status = green.status;
    results.push(green);
    let green_verdict = verdict_of(drill, &results, &report.requirement)?;
    report.step(
        "5. Verdict after the good run",
        format!(
            "{} (result {})",
            code(word(green_verdict)),
            code(status_word(green_status))
        ),
    );
    if green_status != ResultStatus::Pass {
        report.failure = Some(format!(
            "the unchanged eval's result was {}, not pass",
            status_word(green_status)
        ));
    } else if green_verdict != baseline {
        report.failure = Some(format!(
            "after the good run the verdict was {}, not {} as before",
            word(green_verdict),
            word(baseline)
        ));
    }
    Ok(report)
}

/// Runs one eval in stage `cd` and returns its result.
fn one_result(
    drill: &Drill,
    prefix: &str,
    eval: evals_model::Eval,
    commit: &str,
    at: DateTime<Utc>,
    runner: &HttpRunner,
) -> Result<EvalResult> {
    let file = EvalsFile {
        prefix: prefix.to_string(),
        evals: vec![evals_model::Eval {
            stages: vec![Stage::Cd],
            ..eval
        }],
    };
    let ctx = RunContext {
        project: drill.project.to_string(),
        stage: Stage::Cd,
        commit: commit.to_string(),
        run: None,
        evidence: None,
        at,
        partial: false,
        deploy_url: Some(drill.deploy_url.to_string()),
    };
    let (mut results, _) = run::collect(&file, &Reports::default(), &ctx, runner);
    results
        .pop()
        .ok_or_else(|| anyhow!("the eval gave no result"))
}

fn verdict_of(drill: &Drill, results: &[EvalResult], requirement: &str) -> Result<Verdict> {
    let file = store::compute(drill.root, results, Some(drill.project), drill.now)?;
    file.projects
        .iter()
        .flat_map(|p| &p.requirements)
        .find(|r| r.requirement == requirement)
        .map(|r| r.verdict)
        .ok_or_else(|| anyhow!("{requirement} has no verdict in {}", drill.project))
}

/// The `DEPLOY_URL` in a `mise.toml` of the project (its own or a
/// sub-project's, such as `cookbook/cloudflare`).
pub fn deploy_url(root: &Path, project: &str) -> Result<Option<String>> {
    let dir = root.join(project);
    let mut files = vec![dir.join("mise.toml")];
    for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path().join("mise.toml");
        if path.is_file() {
            files.push(path);
        }
    }
    for file in files {
        let Ok(text) = fs::read_to_string(&file) else {
            continue;
        };
        // A plain `DEPLOY_URL = "..."` line, as every project's [env] has.
        let url = text.lines().find_map(|line| {
            let (key, value) = line.split_once('=')?;
            (key.trim() == "DEPLOY_URL").then(|| value.trim().trim_matches('"').to_string())
        });
        if url.is_some() {
            return Ok(url);
        }
    }
    Ok(None)
}

fn code(text: &str) -> String {
    format!("`{text}`")
}

fn word(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Met => "met",
        Verdict::AtRisk => "at-risk",
        Verdict::NotMet => "not-met",
        Verdict::Unknown => "unknown",
    }
}

fn status_word(status: ResultStatus) -> &'static str {
    match status {
        ResultStatus::Pass => "pass",
        ResultStatus::Fail => "fail",
        ResultStatus::Error => "error",
        ResultStatus::Skip => "skip",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{Fetch, Response, Retry};
    use chrono::TimeZone;
    use std::time::Duration;

    const REQUIREMENTS_MD: &str = "# Cookbook requirements

## COOK-001: There is a recipe for every day

- Priority: must
- Why: Jim cooks every day.
- Accept: a recipe page for each day.
";

    const EVALS_TOML: &str = r#"prefix = "COOK"

[[eval]]
id = "recipe-page-live"
requirement = "COOK-001"
stages = ["cd"]
kind = "http"
path = "/recipe/1"

[[eval]]
id = "day-files-exist"
requirement = "COOK-001"
stages = ["ci"]
kind = "cargo"
filter = "test_all_365_day_files_exist"
"#;

    /// Answers every request with the same page.
    struct Page(&'static str);

    impl Fetch for Page {
        fn fetch(&self, _: &str, _: &str, _: bool) -> Result<Response, String> {
            Ok(Response {
                status: 200,
                body: self.0.as_bytes().to_vec(),
                ..Response::default()
            })
        }
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 27, 14, 0, 0)
            .single()
            .unwrap_or_default()
    }

    fn line(eval: &str, stage: &str, at: &str) -> String {
        format!(
            r#"{{"project":"cookbook","requirement":"COOK-001","eval":"{eval}","stage":"{stage}","kind":"http","commit":"aaaa","status":"pass","at":"{at}","by":"ci"}}"#
        )
    }

    fn repo() -> Result<(tempfile::TempDir, tempfile::TempDir)> {
        let root = tempfile::tempdir()?;
        let p = root.path();
        fs::create_dir_all(p.join("cookbook/docs"))?;
        fs::create_dir_all(p.join("cookbook/cloudflare"))?;
        fs::create_dir_all(p.join("standards"))?;
        fs::create_dir_all(p.join("evals"))?;
        fs::write(p.join("cookbook/docs/requirements.md"), REQUIREMENTS_MD)?;
        fs::write(p.join("cookbook/evals.toml"), EVALS_TOML)?;
        fs::write(p.join("cookbook/mise.toml"), "[tools]\n")?;
        fs::write(
            p.join("cookbook/cloudflare/mise.toml"),
            "[env]\nDEPLOY_URL = \"https://jimhillr-cookbook.pages.dev\"\n",
        )?;
        fs::write(p.join("standards/settings.toml"), "")?;
        fs::write(
            p.join("evals/verdicts.toml"),
            include_str!("../../verdicts.toml"),
        )?;
        let dir = tempfile::tempdir()?;
        fs::create_dir_all(dir.path().join("results/cookbook"))?;
        fs::write(
            dir.path().join("results/cookbook/2026-11.jsonl"),
            [
                line("recipe-page-live", "cd", "2026-11-27T10:00:00Z"),
                line("day-files-exist", "ci", "2026-11-27T10:00:00Z"),
            ]
            .join("\n")
                + "\n",
        )?;
        Ok((root, dir))
    }

    fn drill<'a>(root: &'a Path, dir: &'a Path) -> Drill<'a> {
        Drill {
            root,
            dir,
            project: "cookbook",
            eval: "recipe-page-live",
            deploy_url: "https://jimhillr-cookbook.pages.dev",
            now: now(),
        }
    }

    fn runner(fetch: &dyn Fetch) -> HttpRunner<'_> {
        HttpRunner {
            fetch,
            retry: Retry {
                attempts: 1,
                delay: Duration::ZERO,
            },
        }
    }

    #[test]
    fn a_working_loop_goes_red_and_back_to_green() -> Result<()> {
        let (root, dir) = repo()?;
        let page = Page("<html>Cookbook recipe</html>");
        let report = run(&drill(root.path(), dir.path()), &runner(&page))?;
        assert_eq!(report.failure, None, "{report:?}");
        let outcomes: Vec<&str> = report.steps.iter().map(|s| s.outcome.as_str()).collect();
        assert_eq!(outcomes[0], "`met`");
        assert!(outcomes[1].starts_with("`fail`"), "{outcomes:?}");
        assert_eq!(outcomes[2], "`not-met`");
        assert!(outcomes[3].contains("COOK-001"), "{outcomes:?}");
        assert_eq!(outcomes[4], "`met` (result `pass`)");
        // Nothing was written.
        assert!(!root.path().join("cookbook/docs/issues").exists());
        let stored = fs::read_to_string(dir.path().join("results/cookbook/2026-11.jsonl"))?;
        assert_eq!(stored.lines().count(), 2);
        Ok(())
    }

    #[test]
    fn a_site_that_is_down_fails_the_drill_at_the_good_run() -> Result<()> {
        struct Down;
        impl Fetch for Down {
            fn fetch(&self, _: &str, _: &str, _: bool) -> Result<Response, String> {
                Err("connection refused".into())
            }
        }
        let (root, dir) = repo()?;
        let report = run(&drill(root.path(), dir.path()), &runner(&Down))?;
        let failure = report.failure.unwrap_or_default();
        assert!(failure.contains("not pass"), "{failure}");
        Ok(())
    }

    #[test]
    fn only_http_evals_can_be_drilled() -> Result<()> {
        let (root, dir) = repo()?;
        let page = Page("");
        let mut d = drill(root.path(), dir.path());
        d.eval = "day-files-exist";
        let error = run(&d, &runner(&page)).err().map(|e| e.to_string());
        assert!(
            error.as_deref().unwrap_or("").contains("needs an http one"),
            "{error:?}"
        );
        Ok(())
    }

    #[test]
    fn finds_the_deploy_url_in_a_sub_project() -> Result<()> {
        let (root, _dir) = repo()?;
        assert_eq!(
            deploy_url(root.path(), "cookbook")?.as_deref(),
            Some("https://jimhillr-cookbook.pages.dev")
        );
        Ok(())
    }
}
