//! `standards/radar/limits.toml`: every free limit and our budget inside it.

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LimitsFile {
    #[serde(default)]
    pub limit: Vec<Limit>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limit {
    pub key: String,
    pub blip: String,
    pub item: String,
    #[serde(default)]
    pub free: Option<u64>,
    #[serde(default)]
    pub free_note: Option<String>,
    #[serde(default)]
    pub budget: Option<u64>,
    #[serde(default)]
    pub budget_note: Option<String>,
    pub scope: Scope,
    pub unit: String,
    pub past: String,
    pub source: String,
    pub checked: String,
}

impl Limit {
    /// What usage is held to: our budget, or the free limit when there is
    /// no budget. `None` when neither is a number.
    pub fn ceiling(&self) -> Option<u64> {
        self.budget.or(self.free)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    /// Summed over every project.
    Account,
    /// Each project on its own.
    Project,
}

/// Parses `limits.toml` and checks keys are unique and budgets sit inside
/// the free limit.
pub fn parse_limits(text: &str) -> Result<LimitsFile, String> {
    let file: LimitsFile = toml::from_str(text).map_err(|e| e.message().to_string())?;
    let mut seen = std::collections::BTreeSet::new();
    for limit in &file.limit {
        if !seen.insert(limit.key.as_str()) {
            return Err(format!("limit `{}` is listed twice", limit.key));
        }
        if let (Some(budget), Some(free)) = (limit.budget, limit.free)
            && budget > free
        {
            return Err(format!(
                "limit `{}`: budget {budget} is above the free limit {free}",
                limit.key
            ));
        }
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE: &str = r#"
[[limit]]
key = "pages_projects"
blip = "Cloudflare Pages"
item = "Pages projects"
free = 100
budget = 25
scope = "account"
unit = "projects"
past = "Creation fails"
source = "https://developers.cloudflare.com/pages/platform/limits"
checked = "2026-11-26"
"#;

    #[test]
    fn test_parses_a_limit() {
        let file = parse_limits(ONE).expect("parses");
        assert_eq!(file.limit[0].ceiling(), Some(25));
        assert_eq!(file.limit[0].scope, Scope::Account);
    }

    #[test]
    fn test_rejects_duplicates_and_budgets_above_free() {
        let twice = format!("{ONE}{ONE}");
        assert!(parse_limits(&twice).expect_err("dup").contains("twice"));
        let over = ONE.replace("budget = 25", "budget = 200");
        assert!(parse_limits(&over).expect_err("over").contains("above"));
    }

    #[test]
    fn test_ceiling_falls_back_to_the_free_limit() {
        let file = parse_limits(&ONE.replace("budget = 25\n", "")).expect("parses");
        assert_eq!(file.limit[0].ceiling(), Some(100));
    }

    #[test]
    fn test_the_repo_limits_file_parses() {
        let text = include_str!("../../../standards/radar/limits.toml");
        let file = parse_limits(text).expect("standards/radar/limits.toml parses");
        assert!(file.limit.iter().any(|l| l.key == "pages_projects"));
    }
}
