//! The royalty model: schema 1 of `royalty.toml` (the format in
//! `docs/royalty-toml.md`), its parser and validation (RTOML-001 onwards), and the
//! calculation of what a consumer owes.
//!
//! No I/O: callers read the files and pass their text in, so the CLI, the
//! audit and any other tool share the same rules. Nothing here is ever
//! linked into a consumer's program (ROYL-001).

mod finding;
mod owe;
mod parse;
mod price;
mod raw;
mod schema;
mod secrets;

pub use finding::{Code, Finding, Level};
pub use owe::{FamilyOwed, FamilyUse, Limit, Owed, Status, Step, owe};
pub use parse::{Parsed, SCHEMA, parse};
pub use price::{MILLIONTHS, PriceLine, Priced, TierChoice, Usage, matches, names_family, price};
pub use schema::*;
