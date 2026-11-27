//! The threat model: the `docs/threat.toml` each project keeps and the rules
//! (THR-001 to THR-009; THR-004 comes in phase 3) that hold it to
//! `standards/docs/security.md`. The design is in
//! `standards/docs/plans/2026-11-threat-check.md`.
//!
//! No I/O: callers read the files and pass their text in, so pane, the eval
//! engine and the deep review can use the same rules.

mod date;
mod finding;
mod rules;
mod summary;
mod threat_file;

pub use date::Date;
pub use finding::{Code, Finding};
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
