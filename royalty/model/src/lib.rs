//! The royalty model: schema 1 of `royalty.toml` (the format in
//! `docs/royalty-toml.md`), its parser and validation (RTOML-001 onwards).
//!
//! No I/O: callers read the files and pass their text in, so the CLI, the
//! audit and any other tool share the same rules. Nothing here is ever
//! linked into a consumer's program (ROYL-001).

mod finding;
mod parse;
mod raw;
mod schema;
mod secrets;

pub use finding::{Code, Finding, Level};
pub use parse::{Parsed, SCHEMA, parse};
pub use schema::*;
