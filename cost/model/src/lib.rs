//! The cost model: the `docs/cost.toml` each project keeps, the free limits
//! in `standards/radar/limits.toml`, the vendor blips in
//! `standards/radar/radar.json`, and the rules (COST-001 to COST-006 and
//! COST-008) that hold them together. The convention is in
//! `standards/docs/cost.md`.
//!
//! No I/O: callers read the files and pass their text in, so pane and the
//! eval engine can use the same rules.

mod cost_file;
mod finding;
mod limits;
mod month;
mod radar;
mod render;
mod rules;

pub use cost_file::{Cost, CostFile, Every, Plan, Revenue, Service, parse_cost_file};
pub use finding::{Code, Finding};
pub use limits::{Limit, LimitsFile, Scope, parse_limits};
pub use month::Month;
pub use radar::{Blip, Radar, parse_radar};
pub use render::render_limits;
pub use rules::{
    COST_FILE, Decision, Project, REVIEW_MAX_DAYS, check_account, check_project, monthly_cents,
};
