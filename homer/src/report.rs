//! Print a [`PropertyEvaluation`] for the person running homer: a text
//! report by default, or one JSON object with `--json`.

use anyhow::{Context, Result};

use crate::evaluate::PropertyEvaluation;
use crate::zillow::same_address;

/// Format a whole-dollar amount with thousands separators, such as `$750,000`.
pub fn format_dollars(amount: u64) -> String {
    let digits = amount.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    out.push('$');
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Format property details for display
pub fn format_property(evaluation: &PropertyEvaluation) -> String {
    let details = &evaluation.details;
    let mut lines = match &details.matched_address {
        Some(matched) if !same_address(matched, &details.address) => vec![
            format!("Address:    {matched}"),
            format!(
                "Searched:   {} (Zillow matched a different address)",
                details.address
            ),
        ],
        Some(matched) => vec![format!("Address:    {matched}")],
        None => vec![format!("Address:    {}", details.address)],
    };
    if let Some(zpid) = details.zpid {
        lines.push(format!("ZPID:       {zpid}"));
    }

    if let Some(home_type) = &details.home_type {
        lines.push(format!("Type:       {}", home_type));
    }
    if let Some(price) = details.price {
        lines.push(format!("Price:      {}", format_dollars(price)));
    }
    if let Some(beds) = details.bedrooms {
        lines.push(format!("Bedrooms:   {}", beds));
    }
    if let Some(baths) = details.bathrooms {
        lines.push(format!("Bathrooms:  {:.1}", baths));
    }
    if let Some(sqft) = details.living_area {
        lines.push(format!("Sqft:       {}", sqft));
    }
    if let Some(year) = details.year_built {
        lines.push(format!("Year Built: {}", year));
    }
    if let Some(url) = &details.zillow_url {
        lines.push(format!("Zillow URL: {}", url));
    }
    if let Some(desc) = &details.description
        && !desc.is_empty()
    {
        lines.push(format!("\nDescription:\n{}", desc));
    }

    lines.push(String::new());

    if evaluation.criteria_checked > 0 {
        lines.push(format!(
            "Match:      {} of {} criteria met",
            evaluation.criteria_met, evaluation.criteria_checked
        ));
    }

    if evaluation.meets_preferences {
        lines.push("✓ This property meets all your preferences.".to_string());
    } else {
        lines.push("✗ This property does not meet all your preferences:".to_string());
        for criterion in &evaluation.unmet_criteria {
            lines.push(format!("  - {}", criterion));
        }
    }

    lines.join("\n")
}

/// Format the property details and the evaluation as one pretty-printed
/// JSON object, for `--json`.
pub fn format_json(evaluation: &PropertyEvaluation) -> Result<String> {
    serde_json::to_string_pretty(evaluation).context("Failed to write the result as JSON")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluate::tests::make_property;
    use crate::evaluate::{HomePreferences, evaluate_property};
    use crate::zillow::parse_property_details;
    use crate::zillow::tests::zillow_address;
    use serde_json::json;

    #[test]
    fn test_format_property_same_match_shows_no_searched_line() {
        let details = parse_property_details(
            &zillow_address("123 Main St"),
            "123 main st,  seattle, wa 98101",
        )
        .expect("found");
        let output = format_property(&evaluate_property(&details, &HomePreferences::default()));
        assert!(
            output.starts_with("Address:    123 Main St, Seattle, WA 98101\nZPID:       2077\n"),
            "{output}"
        );
        assert!(!output.contains("Searched:"), "{output}");
    }

    #[test]
    fn test_format_property_different_match_shows_typed_address() {
        let typed = "123 Main St, Seattle, WA 98101";
        let details =
            parse_property_details(&zillow_address("1230 Main St"), typed).expect("found");
        let output = format_property(&evaluate_property(&details, &HomePreferences::default()));
        assert!(
            output.starts_with("Address:    1230 Main St, Seattle, WA 98101\n"),
            "{output}"
        );
        assert!(
            output.contains(&format!(
                "Searched:   {typed} (Zillow matched a different address)"
            )),
            "{output}"
        );
    }

    #[test]
    fn test_format_property_without_match_shows_typed_address() {
        let details = make_property(None, None, None, None);
        let output = format_property(&evaluate_property(&details, &HomePreferences::default()));
        assert!(output.starts_with("Address:    123 Main St\n"), "{output}");
        assert!(
            !output.contains("Searched:") && !output.contains("ZPID:"),
            "{output}"
        );
    }

    #[test]
    fn test_format_property_shows_match_score() {
        let details = make_property(Some(900_000), Some(3), Some(2.0), Some(1500));
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            max_price: Some(800_000),
            ..Default::default()
        };
        let output = format_property(&evaluate_property(&details, &prefs));
        assert!(
            output.contains("Match:      1 of 2 criteria met"),
            "{output}"
        );
    }

    #[test]
    fn test_format_json_has_details_and_evaluation() {
        let details = make_property(Some(900_000), Some(3), Some(2.5), Some(1500));
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            max_price: Some(800_000),
            ..Default::default()
        };
        let output = format_json(&evaluate_property(&details, &prefs)).expect("serializes");
        let value: serde_json::Value = serde_json::from_str(&output).expect("valid JSON");
        assert_eq!(
            value,
            json!({
                "details": {
                    "address": "123 Main St",
                    "matched_address": null,
                    "zpid": null,
                    "price": 900_000,
                    "bedrooms": 3,
                    "bathrooms": 2.5,
                    "living_area": 1500,
                    "year_built": null,
                    "home_type": null,
                    "description": null,
                    "zillow_url": null,
                },
                "criteria_checked": 2,
                "criteria_met": 1,
                "meets_preferences": false,
                "unmet_criteria": ["Price: $900,000 (max is $800,000)"],
            })
        );
    }

    #[test]
    fn test_format_json_no_preferences_meets_with_zero_criteria() {
        let details = make_property(None, None, None, None);
        let output = format_json(&evaluate_property(&details, &HomePreferences::default()))
            .expect("serializes");
        let value: serde_json::Value = serde_json::from_str(&output).expect("valid JSON");
        assert_eq!(value["criteria_checked"], 0);
        assert_eq!(value["criteria_met"], 0);
        assert_eq!(value["meets_preferences"], true);
        assert_eq!(value["unmet_criteria"], json!([]));
        assert_eq!(value["details"]["price"], serde_json::Value::Null);
    }

    #[test]
    fn test_format_json_bathrooms_keep_their_decimal() {
        // f32 values must not pick up float noise such as 1.100000023841858.
        let details = make_property(None, None, Some(1.1), None);
        let output = format_json(&evaluate_property(&details, &HomePreferences::default()))
            .expect("serializes");
        assert!(output.contains("\"bathrooms\": 1.1,"), "{output}");
    }

    #[test]
    fn test_format_property_no_preferences_omits_match_score() {
        let details = make_property(Some(750_000), Some(3), Some(2.0), Some(1500));
        let output = format_property(&evaluate_property(&details, &HomePreferences::default()));
        assert!(!output.contains("Match:"), "{output}");
    }

    #[test]
    fn test_format_property_price_has_separators() {
        let details = make_property(Some(750_000), None, None, None);
        let output = format_property(&evaluate_property(&details, &HomePreferences::default()));
        assert!(output.contains("Price:      $750,000"), "{output}");
    }

    #[test]
    fn test_format_dollars() {
        assert_eq!(format_dollars(0), "$0");
        assert_eq!(format_dollars(999), "$999");
        assert_eq!(format_dollars(1_000), "$1,000");
        assert_eq!(format_dollars(12_345), "$12,345");
        assert_eq!(format_dollars(750_000), "$750,000");
        assert_eq!(format_dollars(1_000_000), "$1,000,000");
        assert_eq!(format_dollars(u64::MAX), "$18,446,744,073,709,551,615");
    }

    #[test]
    fn test_format_property_meets_preferences() {
        let details = make_property(Some(750_000), Some(3), Some(2.0), Some(1500));
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            max_price: Some(800_000),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        let output = format_property(&eval);
        assert!(output.contains("✓"));
        assert!(output.contains("$750,000"));
        assert!(output.contains("3"));
    }

    #[test]
    fn test_format_property_unmet_criteria() {
        let details = make_property(Some(900_000), Some(2), Some(1.0), Some(800));
        let prefs = HomePreferences {
            min_bedrooms: Some(3),
            max_price: Some(800_000),
            ..Default::default()
        };
        let eval = evaluate_property(&details, &prefs);
        let output = format_property(&eval);
        assert!(output.contains("✗"));
    }
}
