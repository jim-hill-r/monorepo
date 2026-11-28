//! Score a property against the buyer's preferences.
//!
//! [`HomePreferences`] holds the limits from the command line and
//! [`evaluate_property`] checks a property's [`PropertyDetails`] against
//! each one, giving a [`PropertyEvaluation`] with the criteria it misses. A
//! detail Zillow didn't report counts as not met.

use anyhow::Result;
use serde::Serialize;

use crate::report::format_dollars;
use crate::zillow::{PropertyDetails, bathroom_count};

/// Preferences for evaluating a home
#[derive(Debug, Default)]
pub struct HomePreferences {
    pub min_bedrooms: Option<u32>,
    pub max_bedrooms: Option<u32>,
    pub min_bathrooms: Option<f32>,
    pub max_price: Option<u64>,
    pub min_sqft: Option<u32>,
}

impl HomePreferences {
    /// Fail when no property could meet these preferences, such as
    /// `--min-bedrooms` above `--max-bedrooms`, so homer stops before it
    /// spends a RapidAPI free-quota request on a lookup that can only fail.
    pub fn validate(&self) -> Result<()> {
        if let (Some(min), Some(max)) = (self.min_bedrooms, self.max_bedrooms)
            && min > max
        {
            anyhow::bail!(
                "No property can meet these preferences: --min-bedrooms {min} is more than \
                 --max-bedrooms {max}. Lower --min-bedrooms or raise --max-bedrooms."
            );
        }
        Ok(())
    }
}

///
/// `--json` prints it as is, so renaming a field changes homer's JSON output.
#[derive(Debug, Serialize)]
pub struct PropertyEvaluation {
    pub details: PropertyDetails,
    pub meets_preferences: bool,
    pub unmet_criteria: Vec<String>,
    /// How many criteria the preferences set.
    pub criteria_checked: usize,
    /// How many of those criteria the property meets. An unknown value
    /// counts as not met.
    pub criteria_met: usize,
}

/// Clap value parser for `--min-bathrooms`: a non-negative, finite number.
pub fn parse_bathroom_count(arg: &str) -> std::result::Result<f32, String> {
    let value: f64 = arg
        .trim()
        .parse()
        .map_err(|_| format!("`{arg}` is not a number"))?;
    bathroom_count(value).ok_or_else(|| {
        format!("`{arg}` is not a valid bathroom count: use a non-negative number, such as 1.5")
    })
}

/// Which side of a preference's limit a value must be on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bound {
    AtLeast,
    AtMost,
}

impl Bound {
    fn allows<T: PartialOrd + Copy>(self, value: T, limit: T) -> bool {
        match self {
            Bound::AtLeast => value >= limit,
            Bound::AtMost => value <= limit,
        }
    }

    fn phrase(self) -> &'static str {
        match self {
            Bound::AtLeast => "need at least",
            Bound::AtMost => "max is",
        }
    }
}

/// What one criterion says about a property.
#[derive(Debug, PartialEq, Eq)]
enum Outcome {
    /// The preferences don't set this criterion.
    NotSet,
    Met,
    /// Not met, or the value is unknown, with the line to show the buyer.
    Unmet(String),
}

/// Check one criterion: `value` against `limit` on the side `bound` names.
///
/// Every criterion goes through here, so the count of criteria checked and
/// the list of unmet ones always come from the same list. `format` writes
/// both the value and the limit, such as `1,200 sqft` or `$800,000`.
fn check<T: PartialOrd + Copy>(
    label: &str,
    bound: Bound,
    limit: Option<T>,
    value: Option<T>,
    format: fn(T) -> String,
) -> Outcome {
    let Some(limit) = limit else {
        return Outcome::NotSet;
    };
    match value {
        Some(value) if bound.allows(value, limit) => Outcome::Met,
        Some(value) => Outcome::Unmet(format!(
            "{label}: {} ({} {})",
            format(value),
            bound.phrase(),
            format(limit)
        )),
        None => Outcome::Unmet(format!(
            "{label}: unknown ({} {})",
            bound.phrase(),
            format(limit)
        )),
    }
}

/// Evaluate whether a property meets the given preferences
pub fn evaluate_property(details: &PropertyDetails, prefs: &HomePreferences) -> PropertyEvaluation {
    let count = |n: u32| n.to_string();
    let outcomes = [
        check(
            "Bedrooms",
            Bound::AtLeast,
            prefs.min_bedrooms,
            details.bedrooms,
            count,
        ),
        check(
            "Bedrooms",
            Bound::AtMost,
            prefs.max_bedrooms,
            details.bedrooms,
            count,
        ),
        check(
            "Bathrooms",
            Bound::AtLeast,
            prefs.min_bathrooms,
            details.bathrooms,
            |baths| format!("{baths:.1}"),
        ),
        check(
            "Price",
            Bound::AtMost,
            prefs.max_price,
            details.price,
            format_dollars,
        ),
        check(
            "Square footage",
            Bound::AtLeast,
            prefs.min_sqft,
            details.living_area,
            |sqft| format!("{sqft} sqft"),
        ),
    ];

    let mut criteria_checked = 0;
    let mut unmet_criteria = Vec::new();
    for outcome in outcomes {
        match outcome {
            Outcome::NotSet => {}
            Outcome::Met => criteria_checked += 1,
            Outcome::Unmet(line) => {
                criteria_checked += 1;
                unmet_criteria.push(line);
            }
        }
    }
    let criteria_met = criteria_checked - unmet_criteria.len();

    PropertyEvaluation {
        details: details.clone(),
        meets_preferences: unmet_criteria.is_empty(),
        unmet_criteria,
        criteria_checked,
        criteria_met,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_check_not_set_when_no_limit() {
        let outcome = check("Bedrooms", Bound::AtLeast, None, Some(3), |n: u32| {
            n.to_string()
        });
        assert_eq!(outcome, Outcome::NotSet);
    }

    #[test]
    fn test_check_limit_is_inclusive_on_both_sides() {
        let count = |n: u32| n.to_string();
        assert_eq!(
            check("Bedrooms", Bound::AtLeast, Some(3), Some(3), count),
            Outcome::Met
        );
        assert_eq!(
            check("Bedrooms", Bound::AtMost, Some(3), Some(3), count),
            Outcome::Met
        );
    }

    #[test]
    fn test_check_unmet_and_unknown_lines() {
        let count = |n: u32| n.to_string();
        assert_eq!(
            check("Bedrooms", Bound::AtMost, Some(3), Some(4), count),
            Outcome::Unmet("Bedrooms: 4 (max is 3)".to_string())
        );
        assert_eq!(
            check("Bedrooms", Bound::AtLeast, Some(3), None, count),
            Outcome::Unmet("Bedrooms: unknown (need at least 3)".to_string())
        );
    }

    pub(crate) fn make_property(
        price: Option<u64>,
        bedrooms: Option<u32>,
        bathrooms: Option<f32>,
        living_area: Option<u32>,
    ) -> PropertyDetails {
        PropertyDetails {
            address: "123 Main St".to_string(),
            price,
            bedrooms,
            bathrooms,
            living_area,
            year_built: None,
            home_type: None,
            description: None,
            zillow_url: None,
        }
    }

    #[test]
    fn test_evaluate_keeps_every_detail_field() {
        let details = PropertyDetails {
            address: "9 Elm St".to_string(),
            price: Some(500_000),
            bedrooms: Some(2),
            bathrooms: Some(1.5),
            living_area: Some(1100),
            year_built: Some(1987),
            home_type: Some("CONDO".to_string()),
            description: Some("Sunny corner unit".to_string()),
            zillow_url: Some("https://www.zillow.com/homedetails/9".to_string()),
        };
        let eval = evaluate_property(&details, &HomePreferences::default());
        assert_eq!(eval.details, details);
    }

    #[test]
    fn test_evaluate_meets_all_preferences() {
        let details = make_property(Some(750_000), Some(3), Some(2.0), Some(1500));
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            max_bedrooms: Some(5),
            min_bathrooms: Some(2.0),
            max_price: Some(800_000),
            min_sqft: Some(1200),
        };
        let eval = evaluate_property(&details, &prefs);
        assert!(eval.meets_preferences);
        assert!(eval.unmet_criteria.is_empty());
    }

    #[test]
    fn test_evaluate_match_score_counts_met_criteria() {
        let details = make_property(Some(900_000), Some(3), Some(1.0), Some(1500));
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            max_bedrooms: Some(5),
            min_bathrooms: Some(2.0),
            max_price: Some(800_000),
            min_sqft: Some(1200),
        };
        let eval = evaluate_property(&details, &prefs);
        assert_eq!(eval.criteria_checked, 5);
        assert_eq!(eval.criteria_met, 3);
    }

    #[test]
    fn test_evaluate_match_score_all_met() {
        let details = make_property(Some(750_000), Some(3), Some(2.0), Some(1500));
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            max_price: Some(800_000),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert_eq!((eval.criteria_met, eval.criteria_checked), (2, 2));
    }

    #[test]
    fn test_evaluate_match_score_unknown_value_is_not_met() {
        let details = make_property(None, Some(3), Some(2.0), Some(1500));
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            max_price: Some(800_000),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert_eq!((eval.criteria_met, eval.criteria_checked), (1, 2));
    }

    #[test]
    fn test_evaluate_match_score_no_preferences() {
        let details = make_property(Some(750_000), Some(3), Some(2.0), Some(1500));
        let eval = evaluate_property(&details, &HomePreferences::default());
        assert_eq!((eval.criteria_met, eval.criteria_checked), (0, 0));
    }

    #[test]
    fn test_evaluate_fails_price() {
        let details = make_property(Some(900_000), Some(3), Some(2.0), Some(1500));
        let prefs = HomePreferences {
            max_price: Some(800_000),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert!(!eval.meets_preferences);
        assert_eq!(eval.unmet_criteria.len(), 1);
        assert_eq!(eval.unmet_criteria[0], "Price: $900,000 (max is $800,000)");
    }

    #[test]
    fn test_evaluate_unknown_price_formats_max() {
        let details = make_property(None, None, None, None);
        let prefs = HomePreferences {
            max_price: Some(1_250_000),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert_eq!(
            eval.unmet_criteria,
            vec!["Price: unknown (max is $1,250,000)".to_string()]
        );
    }

    #[test]
    fn test_evaluate_fails_bedrooms() {
        let details = make_property(Some(500_000), Some(2), Some(2.0), Some(1500));
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert!(!eval.meets_preferences);
        assert_eq!(eval.unmet_criteria.len(), 1);
        assert!(eval.unmet_criteria[0].contains("Bedrooms"));
    }

    #[test]
    fn test_evaluate_fails_bathrooms() {
        let details = make_property(Some(500_000), Some(3), Some(1.0), Some(1500));
        let prefs = HomePreferences {
            min_bathrooms: Some(2.0),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert!(!eval.meets_preferences);
        assert!(eval.unmet_criteria[0].contains("Bathrooms"));
    }

    #[test]
    fn test_evaluate_fails_sqft() {
        let details = make_property(Some(500_000), Some(3), Some(2.0), Some(800));
        let prefs = HomePreferences {
            min_sqft: Some(1200),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert!(!eval.meets_preferences);
        assert!(eval.unmet_criteria[0].contains("Square footage"));
    }

    #[test]
    fn test_evaluate_no_preferences() {
        let details = make_property(Some(500_000), Some(3), Some(2.0), Some(1500));
        let prefs = HomePreferences::default();
        let eval = evaluate_property(&details, &prefs);
        assert!(eval.meets_preferences);
    }

    #[test]
    fn test_evaluate_missing_data_fails_min_bedrooms() {
        let details = make_property(None, None, None, None);
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert!(!eval.meets_preferences);
        assert!(eval.unmet_criteria[0].contains("unknown"));
    }

    #[test]
    fn test_evaluate_max_bedrooms_exceeded() {
        let details = make_property(None, Some(6), None, None);
        let prefs = HomePreferences {
            max_bedrooms: Some(4),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert!(!eval.meets_preferences);
    }

    #[test]
    fn test_evaluate_missing_data_fails_max_bedrooms() {
        let details = make_property(None, None, None, None);
        let prefs = HomePreferences {
            max_bedrooms: Some(4),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert!(!eval.meets_preferences);
        assert_eq!(eval.unmet_criteria, vec!["Bedrooms: unknown (max is 4)"]);
    }

    #[test]
    fn test_evaluate_max_bedrooms_at_limit_passes() {
        let details = make_property(None, Some(4), None, None);
        let prefs = HomePreferences {
            max_bedrooms: Some(4),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        assert!(eval.meets_preferences);
    }

    #[test]
    fn test_validate_rejects_min_bedrooms_above_max() {
        let prefs = HomePreferences {
            min_bedrooms: Some(4),
            max_bedrooms: Some(2),
            ..Default::default()
        };
        let message = prefs.validate().expect_err("4 > 2 must fail").to_string();
        assert!(message.contains("--min-bedrooms 4"), "{message}");
        assert!(message.contains("--max-bedrooms 2"), "{message}");
    }

    #[test]
    fn test_validate_accepts_equal_bedroom_bounds() {
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            max_bedrooms: Some(3),
            ..Default::default()
        };
        assert!(prefs.validate().is_ok());
    }

    #[test]
    fn test_validate_accepts_one_sided_or_no_bounds() {
        assert!(HomePreferences::default().validate().is_ok());
        let only_min = HomePreferences {
            min_bedrooms: Some(9),
            ..Default::default()
        };
        assert!(only_min.validate().is_ok());
        let only_max = HomePreferences {
            max_bedrooms: Some(0),
            ..Default::default()
        };
        assert!(only_max.validate().is_ok());
    }

    #[test]
    fn test_parse_bathroom_count_accepts_valid_values() {
        assert_eq!(parse_bathroom_count("0"), Ok(0.0));
        assert_eq!(parse_bathroom_count("2"), Ok(2.0));
        assert_eq!(parse_bathroom_count("1.5"), Ok(1.5));
    }

    #[test]
    fn test_parse_bathroom_count_rejects_bad_values() {
        for bad in ["NaN", "inf", "-inf", "infinity", "-1", "-0.5", "two", ""] {
            assert!(
                parse_bathroom_count(bad).is_err(),
                "{bad} should be rejected"
            );
        }
    }
}
