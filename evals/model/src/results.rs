//! One record per eval run, as written to `evals-results.jsonl` and the
//! `evals-results` branch. See the Results section of the eval engine plan.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::evals::Stage;

/// The outcome of one eval run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResultStatus {
    Pass,
    Fail,
    /// The eval itself broke. Counts as at risk, not as a product failure.
    Error,
    /// The eval didn't run. Ignored by the verdicts.
    Skip,
}

/// Who produced a result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum By {
    Ci,
    Agent,
    Jim,
}

/// One line of a results file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalResult {
    pub project: String,
    pub requirement: String,
    /// The eval's `id`, or `human` for Jim's rating.
    pub eval: String,
    pub stage: Stage,
    pub kind: String,
    pub commit: String,
    /// The GitHub deployment id, for `cd`, `chaos` and `observe` results.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deploy: Option<String>,
    pub status: ResultStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
    /// The threshold the value was compared with, such as `<= 2500`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<String>,
    /// A link to the run, report, recording or query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    /// The workflow run id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    pub at: DateTime<Utc>,
    pub by: By,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_jsonl_line() {
        let line = r#"{"project":"cookbook","requirement":"COOK-001","eval":"recipe-lcp","stage":"observe","kind":"metric","commit":"abc1234","deploy":"42","status":"pass","value":2310,"threshold":"<= 2500","evidence":"https://example.com/q","run":"7","at":"2026-12-02T04:00:00Z","by":"ci"}"#;
        let result: Result<EvalResult, _> = serde_json::from_str(line);
        let result = result.map_err(|e| e.to_string());
        let result = match result {
            Ok(r) => r,
            Err(e) => panic!("{e}"),
        };
        assert_eq!(result.stage, Stage::Observe);
        assert_eq!(result.status, ResultStatus::Pass);
        assert_eq!(result.value, Some(2310.0));
        assert_eq!(result.by, By::Ci);
        assert_eq!(result.at.to_rfc3339(), "2026-12-02T04:00:00+00:00");
    }

    #[test]
    fn optional_fields_may_be_left_out() {
        let line = r#"{"project":"cookbook","requirement":"COOK-001","eval":"human","stage":"human","kind":"human","commit":"abc1234","status":"fail","at":"2026-12-02T04:00:00Z","by":"jim"}"#;
        let result: Result<EvalResult, _> = serde_json::from_str(line);
        assert!(result.is_ok(), "{result:?}");
    }
}
