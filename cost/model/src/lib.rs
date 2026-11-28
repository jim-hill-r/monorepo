//! The cost model: the `docs/cost.toml` each project keeps, the free limits
//! in `standards/radar/limits.toml`, the vendor blips in
//! `standards/radar/radar.json`, the detectors in `standards/radar/detectors.toml`, and the rules
//! (COST-001 to COST-010) that hold them together, plus the per-PR summary and the renewals list.
//! The convention is in `standards/docs/cost.md`.
//!
//! No I/O: callers read the files and pass their text in, so pane and the
//! eval engine can use the same rules.

mod cost_file;
mod detectors;
mod finding;
mod limits;
mod month;
mod radar;
mod render;
mod rules;
mod summary;

pub use cost_file::{Cost, CostFile, Every, Plan, Revenue, Service, parse_cost_file};
pub use detectors::{
    Allow, Detection, Detector, Detectors, DetectorsFile, Exempt, Hit, Scan, SourceFile,
    ThreatHint, UNKNOWN_HOST, UnknownHost, check_drift, parse_detectors,
};
pub use finding::{Code, Finding};
pub use limits::{Limit, LimitsFile, Scope, parse_limits};
pub use month::Month;
pub use radar::{Blip, Radar, parse_radar};
pub use render::render_limits;
pub use rules::{
    COST_FILE, Decision, Project, REVIEW_MAX_DAYS, Renewal, check_account, check_project,
    monthly_cents, render_renewals, renewals,
};
pub use summary::{
    BudgetChange, ProjectChange, Snapshot, Summary, UsageChange, render_summary, summarize,
};
