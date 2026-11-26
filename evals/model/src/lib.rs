//! Types, parsers and verdict rules for the eval engine.
//!
//! The formats are defined in `standards/docs/evals.md` and the verdict rules
//! in `standards/docs/plans/2026-11-eval-engine.md`. This crate does no I/O:
//! callers read the files and pass their text in, so pane can run the same
//! rules on the same data.

mod evals;
mod ids;
mod requirements;
mod results;
mod verdict;

pub use evals::{Eval, EvalsFile, Kind, METRIC_SOURCES, Stage, parse_evals};
pub use ids::{is_kebab_case, parse_requirement_id, snake_case_id};
pub use requirements::{Priority, Requirement, Status, parse_requirements};
pub use results::{By, EvalResult, ResultStatus};
pub use verdict::{Thresholds, Verdict, VerdictReport, requirement_order, verdict};

/// A problem found while parsing a file, with the 1-based line it is on when
/// that is known.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{}{message}", line.map(|l| format!("line {l}: ")).unwrap_or_default())]
pub struct ParseError {
    pub line: Option<usize>,
    pub message: String,
}

impl ParseError {
    pub(crate) fn at(line: usize, message: impl Into<String>) -> Self {
        Self {
            line: Some(line),
            message: message.into(),
        }
    }

    pub(crate) fn whole_file(message: impl Into<String>) -> Self {
        Self {
            line: None,
            message: message.into(),
        }
    }
}
