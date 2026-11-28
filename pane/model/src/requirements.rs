//! Requirements and their eval results, from the `evals-results` branch.
//!
//! The eval engine (`evals/model`) judges every requirement and writes the
//! verdicts to `verdicts.json`, and every eval run to
//! `results/<project>/<yyyy-mm>.jsonl`. `pane_snapshot` reads both into
//! [`Requirements`] at build time, keeping each eval's newest runs, so the
//! Project page can show each requirement's verdict with its evidence and
//! history, and the Health page a **Requirements** light. Projects here are
//! top-level folders (`cookbook`, not `cookbook/web`), as the eval engine
//! names them.

use serde::{Deserialize, Serialize};

/// Runs kept per eval and stage: the newest first, enough to see a flaky
/// eval without making the snapshot large.
pub const HISTORY: usize = 10;

/// Everything the eval engine says about the repo's requirements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Requirements {
    /// When the verdicts were computed, RFC 3339.
    pub generated_at: String,
    #[serde(default)]
    pub projects: Vec<ProjectRequirements>,
}

impl Requirements {
    /// The entry for a top-level project, if the eval engine lists it.
    pub fn project(&self, project: &str) -> Option<&ProjectRequirements> {
        self.projects.iter().find(|p| p.project == project)
    }
}

/// One project's requirements, from its `docs/requirements.md`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectRequirements {
    /// The top-level project folder, such as `cookbook`.
    pub project: String,
    /// Active `must` requirements.
    #[serde(default)]
    pub musts: u32,
    /// `must` requirements with at least one production-stage result.
    #[serde(default)]
    pub musts_in_production: u32,
    #[serde(default)]
    pub coverage: Coverage,
    #[serde(default)]
    pub requirements: Vec<Requirement>,
}

/// A project's eval coverage, as `verdicts.json` states it.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Coverage {
    /// Active requirements.
    #[serde(default)]
    pub requirements: u32,
    /// Requirements whose verdict is met.
    #[serde(default)]
    pub met: u32,
    /// `met` as a percentage of `requirements`.
    #[serde(default)]
    pub percent: f64,
    #[serde(default)]
    pub musts_met: u32,
    /// `musts_met` as a percentage of the musts; `None` with no musts.
    #[serde(default)]
    pub musts_percent: Option<f64>,
    /// Coverage is below the eval engine's warning line.
    #[serde(default)]
    pub warning: bool,
}

/// One requirement, its verdict and its evals' runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    /// Its id, such as `PANE-001`.
    #[serde(rename = "requirement")]
    pub id: String,
    pub title: String,
    /// `must`, `should` or `could`.
    pub priority: String,
    pub verdict: Verdict,
    /// Why it has this verdict; empty when met.
    #[serde(default)]
    pub reasons: Vec<String>,
    /// Each eval and stage with results, by eval id then stage.
    #[serde(default)]
    pub evals: Vec<EvalRuns>,
}

impl Requirement {
    pub fn is_must(&self) -> bool {
        self.priority == "must"
    }
}

/// A requirement's state, as the eval engine judges it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    Met,
    AtRisk,
    NotMet,
    Unknown,
}

impl Verdict {
    /// How pane writes it: `Met`, `At risk`, `Not met` or `Unknown`.
    pub fn label(self) -> &'static str {
        match self {
            Self::Met => "Met",
            Self::AtRisk => "At risk",
            Self::NotMet => "Not met",
            Self::Unknown => "Unknown",
        }
    }

    /// Its traffic-light colour: met is green, at risk amber, not met red
    /// and unknown grey.
    pub fn health(self) -> crate::Health {
        match self {
            Self::Met => crate::Health::Green,
            Self::AtRisk => crate::Health::Amber,
            Self::NotMet => crate::Health::Red,
            Self::Unknown => crate::Health::Grey,
        }
    }
}

/// One eval's runs at one stage, newest first (at most [`HISTORY`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalRuns {
    /// The eval's id, or `human` for Jim's ratings.
    pub eval: String,
    /// `ci`, `cd`, `chaos`, `observe` or `human`.
    pub stage: String,
    /// Every run, newest first; the first is the latest result.
    pub runs: Vec<EvalRun>,
}

impl EvalRuns {
    /// The latest run.
    pub fn latest(&self) -> Option<&EvalRun> {
        self.runs.first()
    }
}

/// One run of an eval: a line of a results file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvalRun {
    pub status: RunStatus,
    /// When it ran, RFC 3339.
    pub at: String,
    /// The commit it ran against.
    #[serde(default)]
    pub commit: String,
    /// A link to the run, report, recording or query.
    #[serde(default)]
    pub evidence: Option<String>,
    /// A short reason, such as `2 of 3 tests failed`.
    #[serde(default)]
    pub note: Option<String>,
}

/// The outcome of one eval run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunStatus {
    Pass,
    Fail,
    Error,
    Skip,
}

impl RunStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::Error => "error",
            Self::Skip => "skip",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Health;

    #[test]
    fn reads_a_verdicts_json_project() -> Result<(), serde_json::Error> {
        let json = r#"{"generated_at":"2026-11-28T04:37:48Z","coverage":{"projects":2},
          "projects":[{"project":"pane","musts":2,"musts_in_production":1,
            "coverage":{"requirements":3,"met":1,"percent":33.3,"musts_met":1,"musts_percent":50.0,"warning":true},
            "requirements":[
              {"requirement":"PANE-001","title":"Today shows what needs Jim","priority":"must","verdict":"met"},
              {"requirement":"PANE-002","title":"On main","priority":"must","verdict":"not-met","reasons":["x failed in cd"]}]},
            {"project":"athenaeum","musts":0,"musts_in_production":0,
             "coverage":{"requirements":0,"met":0,"percent":0.0,"musts_met":0,"warning":true},"requirements":[]}]}"#;
        let r: Requirements = serde_json::from_str(json)?;
        let pane = r.project("pane").map(|p| p.requirements.clone());
        let pane = pane.unwrap_or_default();
        assert_eq!(pane[1].verdict, Verdict::NotMet);
        assert_eq!(pane[1].reasons, ["x failed in cd"]);
        assert!(pane[0].reasons.is_empty() && pane[0].evals.is_empty());
        assert!(pane[0].is_must());
        let athenaeum = r.project("athenaeum").map(|p| p.coverage.musts_percent);
        assert_eq!(athenaeum, Some(None));
        assert!(r.project("cookbook").is_none());
        Ok(())
    }

    #[test]
    fn verdicts_have_labels_and_lights() {
        assert_eq!(Verdict::AtRisk.label(), "At risk");
        assert_eq!(Verdict::NotMet.health(), Health::Red);
        assert_eq!(Verdict::Unknown.health(), Health::Grey);
        assert_eq!(Verdict::Met.health(), Health::Green);
    }

    #[test]
    fn a_run_reads_a_results_line() -> Result<(), serde_json::Error> {
        let line = r#"{"project":"bear_down","requirement":"BEAR-002","eval":"games-ranked-by-swing","stage":"ci","kind":"cargo","commit":"9d5461a","status":"pass","evidence":"https://github.com/jimhillr/monorepo/actions/runs/1","run":"1","note":"1 test passed","at":"2026-11-26T22:53:28Z","by":"ci"}"#;
        let run: EvalRun = serde_json::from_str(line)?;
        assert_eq!(run.status, RunStatus::Pass);
        assert_eq!(run.note.as_deref(), Some("1 test passed"));
        assert_eq!(RunStatus::Error.label(), "error");
        Ok(())
    }
}
