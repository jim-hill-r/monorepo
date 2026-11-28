//! `<project>/docs/cost.toml`: what a project uses, on which plan, how much
//! of each free limit it expects to use, and its own money.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};

use crate::Month;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CostFile {
    /// The last time someone checked this file against the code.
    #[serde(deserialize_with = "date")]
    pub reviewed: NaiveDate,
    /// One or two sentences: what the project runs on and what it costs.
    pub summary: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub service: Vec<Service>,
    #[serde(default)]
    pub cost: Vec<Cost>,
    #[serde(default)]
    pub revenue: Vec<Revenue>,
}

/// An outside service the project uses.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    /// A vendor blip in `radar.json`.
    pub blip: String,
    pub plan: Plan,
    pub why: String,
    /// Expected use, keyed by `key` in `limits.toml`.
    #[serde(default)]
    pub usage: BTreeMap<String, u64>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Plan {
    Free,
    Paid,
}

/// Money that is this project's own, such as a domain.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cost {
    pub name: String,
    /// A vendor blip in `radar.json`.
    pub vendor: String,
    /// Whole US cents, so nothing is lost to floating point.
    pub cents: u64,
    pub every: Every,
    /// The first month it is paid.
    pub since: Month,
    /// The next charge, `YYYY-MM-DD`. Required for `every = "year"`, so a
    /// renewal can be seen coming; not allowed for `every = "once"`.
    #[serde(default, deserialize_with = "optional_date")]
    pub renews: Option<NaiveDate>,
    /// A decision record, repo-relative. Required when `cents > 0`.
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Every {
    Month,
    Year,
    Once,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revenue {
    pub month: Month,
    pub cents: u64,
    pub source: String,
}

fn date<'de, D: Deserializer<'de>>(deserializer: D) -> Result<NaiveDate, D::Error> {
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y-%m-%d")
        .map_err(|_| serde::de::Error::custom(format!("`{s}` is not a date written YYYY-MM-DD")))
}

fn optional_date<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<NaiveDate>, D::Error> {
    date(deserializer).map(Some)
}

/// Parses a `docs/cost.toml`. The error is one line, for a COST-002 finding.
pub fn parse_cost_file(text: &str) -> Result<CostFile, String> {
    toml::from_str(text).map_err(|e| e.message().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"
reviewed = "2026-11-26"
summary = "Static site on Cloudflare Pages with Auth0 sign-in. $0 a month."

[[service]]
blip = "Cloudflare Pages"
plan = "free"
why = "Hosts the site."
[service.usage]
pages_projects = 1
workers_requests_per_day = 2000

[[cost]]
name = "cookbook.example.com domain"
vendor = "Squarespace"
cents = 1200
every = "year"
since = "2026-12"
renews = "2027-12-01"
decision = "standards/docs/decisions/0006-cookbook-domain.md"

[[revenue]]
month = "2026-11"
cents = 0
source = "None yet"
"#;

    #[test]
    fn test_parses_the_example_from_the_plan() {
        let file = parse_cost_file(EXAMPLE).expect("parses");
        assert_eq!(
            file.reviewed,
            NaiveDate::from_ymd_opt(2026, 11, 26).expect("date")
        );
        assert_eq!(file.service[0].plan, Plan::Free);
        assert_eq!(file.service[0].usage["pages_projects"], 1);
        assert_eq!(file.cost[0].every, Every::Year);
        assert_eq!(
            file.cost[0].since,
            Month {
                year: 2026,
                month: 12
            }
        );
        assert_eq!(
            file.cost[0].renews,
            Some(NaiveDate::from_ymd_opt(2027, 12, 1).expect("date"))
        );
        assert_eq!(file.revenue[0].cents, 0);
    }

    #[test]
    fn test_summary_only_file_parses() {
        let file = parse_cost_file("reviewed = \"2026-11-26\"\nsummary = \"A library.\"\n")
            .expect("parses");
        assert!(file.service.is_empty() && file.cost.is_empty());
    }

    #[test]
    fn test_rejects_unknown_fields_plans_and_every() {
        let base = "reviewed = \"2026-11-26\"\nsummary = \"x\"\n";
        for (extra, want) in [
            ("colour = \"red\"\n", "unknown field"),
            (
                "[[service]]\nblip = \"Auth0\"\nplan = \"pro\"\nwhy = \"x\"\n",
                "unknown variant",
            ),
            (
                "[[cost]]\nname = \"x\"\nvendor = \"Squarespace\"\ncents = 1\nevery = \"week\"\nsince = \"2026-11\"\n",
                "unknown variant",
            ),
        ] {
            let err = parse_cost_file(&format!("{base}{extra}")).expect_err(extra);
            assert!(err.contains(want), "{err}");
        }
    }

    #[test]
    fn test_rejects_bad_dates() {
        let err =
            parse_cost_file("reviewed = \"26 Nov 2026\"\nsummary = \"x\"\n").expect_err("bad");
        assert!(err.contains("YYYY-MM-DD"), "{err}");
        let cost = "reviewed = \"2026-11-26\"\nsummary = \"x\"\n[[cost]]\nname = \"x\"\nvendor = \"Squarespace\"\ncents = 1\nevery = \"year\"\nsince = \"2026-11\"\nrenews = \"2027-11\"\n";
        let err = parse_cost_file(cost).expect_err("bad renews");
        assert!(err.contains("YYYY-MM-DD"), "{err}");
    }

    #[test]
    fn test_renews_is_optional() {
        let cost = "reviewed = \"2026-11-26\"\nsummary = \"x\"\n[[cost]]\nname = \"x\"\nvendor = \"Squarespace\"\ncents = 1\nevery = \"month\"\nsince = \"2026-11\"\n";
        let file = parse_cost_file(cost).expect("parses");
        assert_eq!(file.cost[0].renews, None);
    }
}
