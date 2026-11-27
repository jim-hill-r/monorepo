//! The file exactly as written: every field optional, words as strings and
//! numbers as `i64`, so validation can say what is wrong in the format's own
//! terms instead of failing on the first unexpected value.

use std::collections::BTreeMap;

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct RawFile {
    pub schema: Option<i64>,
    pub project: Option<RawProject>,
    pub license: Option<RawLicense>,
    #[serde(default)]
    pub payee: Vec<RawPayee>,
    pub terms: Option<RawTerms>,
    #[serde(default)]
    pub tier: Vec<RawTier>,
    pub caps: Option<RawCaps>,
    pub income: Option<RawIncome>,
    pub measure: Option<RawMeasure>,
    pub consumer: Option<RawConsumer>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawProject {
    pub name: Option<String>,
    pub description: Option<String>,
    pub repository: Option<String>,
    pub packages: Option<Vec<String>>,
    pub family: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawLicense {
    pub code: Option<String>,
    pub binding: Option<String>,
    pub terms: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawPayee {
    pub name: Option<String>,
    pub share: Option<i64>,
    pub github: Option<String>,
    #[serde(default)]
    pub method: Vec<RawMethod>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawMethod {
    pub kind: Option<String>,
    pub account: Option<String>,
    pub contact: Option<String>,
    pub url: Option<String>,
    pub network: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawTerms {
    pub mode: Option<String>,
    pub currency: Option<String>,
    pub period: Option<String>,
    pub reach: Option<String>,
    pub trial_days: Option<i64>,
    pub minimum_payment_cents: Option<i64>,
    pub report_to: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawTier {
    pub id: Option<String>,
    pub title: Option<String>,
    pub when: Option<RawWhen>,
    pub free: Option<bool>,
    #[serde(default)]
    pub price: Vec<RawPrice>,
}

/// Tier conditions. An unknown condition is an error, not a warning: if it
/// were ignored, the tier would match more uses than the producer meant and
/// could charge someone it shouldn't.
#[derive(Debug, Default, Deserialize)]
pub struct RawWhen {
    pub environment: Option<OneOrMany>,
    pub entity: Option<OneOrMany>,
    pub class: Option<OneOrMany>,
    pub revenue_below_usd: Option<i64>,
    pub seats_at_most: Option<i64>,
    pub dependency: Option<OneOrMany>,
    #[serde(flatten)]
    pub unknown: BTreeMap<String, toml::Value>,
}

/// A condition value: one word or a list of words.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    pub fn into_vec(self) -> Vec<String> {
        match self {
            OneOrMany::One(one) => vec![one],
            OneOrMany::Many(many) => many,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct RawPrice {
    pub unit: Option<String>,
    pub cents: Option<i64>,
    pub per: Option<i64>,
    pub included: Option<i64>,
    pub max_units: Option<i64>,
    pub basis_points: Option<i64>,
    pub describe: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawCaps {
    pub project: Option<RawCap>,
    pub person: Option<RawCap>,
    pub organization: Option<RawCap>,
    pub consumer: Option<RawCap>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawCap {
    pub period_cents: Option<i64>,
    pub year_cents: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawIncome {
    pub ceiling_cents: Option<i64>,
    pub window: Option<String>,
    pub on_reach: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawMeasure {
    pub sources: Option<Vec<String>>,
    pub sample_one_in: Option<i64>,
    pub runtime: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawConsumer {
    pub id: Option<String>,
    pub entity: Option<String>,
    pub name: Option<String>,
    pub class: Option<String>,
    pub annual_revenue_below_usd: Option<i64>,
    pub stage: Option<String>,
    pub seats: Option<i64>,
    #[serde(default)]
    pub deployment: Vec<RawDeployment>,
    pub budget: Option<RawBudget>,
    pub policy: Option<RawPolicy>,
    pub measure: Option<RawConsumerMeasure>,
    #[serde(default)]
    pub package: Vec<RawPackage>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawDeployment {
    pub name: Option<String>,
    pub manifest: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawBudget {
    pub period_cents: Option<i64>,
    pub year_cents: Option<i64>,
    pub per_producer_period_cents: Option<i64>,
    pub on_exceed: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawPolicy {
    pub bindings: Option<Vec<String>>,
    pub modes: Option<Vec<String>>,
    pub transitive: Option<String>,
    pub review_above_cents: Option<i64>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawConsumerMeasure {
    pub sources: Option<Vec<String>>,
    pub run: Option<String>,
    pub runtime: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RawPackage {
    pub name: Option<String>,
    pub action: Option<String>,
    pub period_cents: Option<i64>,
    pub unit: Option<String>,
    pub units: Option<i64>,
    pub reason: Option<String>,
}
