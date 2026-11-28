//! homer looks a house up on Zillow and scores it against a buyer's
//! preferences.
//!
//! The binary in `main.rs` only reads the command line; the work is here:
//!
//! - [`zillow`]: the RapidAPI client and the parsing of its response,
//! - [`evaluate`]: the preferences and the score,
//! - [`report`]: the text and JSON output,
//! - [`quota`]: the guard that keeps homer inside RapidAPI's free plan.

pub mod evaluate;
pub mod quota;
pub mod report;
pub mod zillow;
