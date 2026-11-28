//! The threat model: the `docs/threat.toml` each project keeps and the rules
//! (THR-001 to THR-009) that hold it to `standards/docs/security.md`.
//! THR-004 (drift) reads the matches of the cost check's shared detectors
//! in `standards/radar/detectors.toml`, through `cost_model`. The design is in
//! `standards/docs/plans/2026-11-threat-check.md`.
//!
//! No I/O: callers read the files and pass their text in, so pane, the eval
//! engine and the deep review can use the same rules.

mod date;
mod drift;
mod finding;
mod live;
mod rules;
mod summary;
mod threat_file;

pub use date::Date;
pub use drift::check_drift;
pub use finding::{Code, Finding};
pub use live::{
    Account, LiveFinding, Model, Orphan, PagesProject, Probe, Target, check_live, render_live,
    targets,
};
pub use rules::{
    Context, REQUIRED_HEADERS, REVIEW_MAX_DAYS, THREAT_FILE, Workflow, check_project,
    parse_setting_ids, workflow_secrets,
};
pub use summary::{
    Change, FilePair, NEEDS_HUMAN_LABEL, SECURITY_LABEL, Summary, is_security_file, summarize,
};
pub use threat_file::{
    Access, Auth, Data, Flow, Kind, Secret, Stride, Surface, Threat, ThreatFile, Where,
    parse_threat_file,
};
