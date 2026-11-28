//! Builds pane's `snapshot.json` from a checkout of the monorepo.
//!
//! [`parse`] holds pure parsers over file contents; [`repo`] walks a checkout
//! and assembles a [`pane_model::Snapshot`] from them; [`evals`] reads the
//! `evals-results` branch.

pub mod blame;
pub mod evals;
pub mod merged;
pub mod parse;
pub mod repo;
pub mod time;
