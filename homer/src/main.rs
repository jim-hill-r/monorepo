use anyhow::{Context, Result};
use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

/// Homer - Look up house details on Zillow and score them against your preferences.
#[derive(Parser, Debug)]
#[command(name = "homer")]
#[command(about = "Look up house details on Zillow and evaluate against your preferences")]
struct Args {
    /// The property address to look up (e.g. "123 Main St, Seattle, WA 98101")
    #[arg(short, long)]
    address: String,

    /// RapidAPI key for Zillow API access
    #[arg(long, env = "RAPIDAPI_KEY")]
    api_key: String,

    /// Minimum number of bedrooms
    #[arg(long)]
    min_bedrooms: Option<u32>,

    /// Maximum number of bedrooms
    #[arg(long)]
    max_bedrooms: Option<u32>,

    /// Minimum number of bathrooms
    #[arg(long)]
    min_bathrooms: Option<f32>,

    /// Maximum listing price
    #[arg(long)]
    max_price: Option<u64>,

    /// Minimum square footage
    #[arg(long)]
    min_sqft: Option<u32>,
}

/// Preferences for evaluating a home
#[derive(Debug, Default)]
pub struct HomePreferences {
    pub min_bedrooms: Option<u32>,
    pub max_bedrooms: Option<u32>,
    pub min_bathrooms: Option<f32>,
    pub max_price: Option<u64>,
    pub min_sqft: Option<u32>,
}

/// Property details returned by the Zillow API
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct PropertyDetails {
    pub address: String,
    pub price: Option<u64>,
    pub bedrooms: Option<u32>,
    pub bathrooms: Option<f32>,
    pub living_area: Option<u32>,
    pub year_built: Option<u32>,
    pub home_type: Option<String>,
    pub description: Option<String>,
    pub zillow_url: Option<String>,
}

/// Result of evaluating a property against preferences
#[derive(Debug)]
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

/// RapidAPI host for the Zillow API, sent as the `x-rapidapi-host` header.
const RAPIDAPI_HOST: &str = "zillow-com1.p.rapidapi.com";

/// Default base URL for the Zillow API on RapidAPI.
pub const DEFAULT_BASE_URL: &str = "https://zillow-com1.p.rapidapi.com";

/// Zillow API client backed by RapidAPI
pub struct ZillowClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl ZillowClient {
    /// Create a client that talks to the real Zillow API on RapidAPI.
    pub fn new(api_key: String) -> Self {
        Self::with_base_url(api_key, DEFAULT_BASE_URL.to_string())
    }

    /// Create a client that sends requests to `base_url` instead of RapidAPI,
    /// for example a local mock server in tests.
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Look up property details for a given address
    pub fn property_details(&self, address: &str) -> Result<PropertyDetails> {
        let url = format!("{}/property", self.base_url);
        let response = self
            .client
            .get(url)
            .header("x-rapidapi-host", RAPIDAPI_HOST)
            .header("x-rapidapi-key", &self.api_key)
            .query(&[("address", address)])
            .send()
            .context("Failed to contact Zillow API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!("Zillow API returned status {}: {}", status, body);
        }

        let raw: serde_json::Value = response
            .json()
            .context("Failed to parse Zillow API response")?;

        parse_property_details(&raw, address)
    }
}

/// Read a JSON number as a non-negative whole number.
///
/// Accepts integers (`1600`) and whole-number floats (`1600.0`). Returns
/// `None` for negative or fractional values, non-finite or out-of-range
/// floats, and non-numbers.
fn json_whole_number(value: &serde_json::Value) -> Option<u64> {
    if let Some(n) = value.as_u64() {
        return Some(n);
    }
    let f = value.as_f64()?;
    // 2^64 is exactly representable as f64; anything at or above it overflows u64.
    const U64_LIMIT: f64 = 18_446_744_073_709_551_616.0;
    if f >= 0.0 && f.fract() == 0.0 && f < U64_LIMIT {
        // The checks above make this cast exact.
        Some(f as u64)
    } else {
        None
    }
}

/// Read a JSON number as a `u32`, or `None` if it isn't a whole number that fits.
fn json_u32(value: &serde_json::Value) -> Option<u32> {
    json_whole_number(value).and_then(|n| u32::try_from(n).ok())
}

/// Parse property details from the raw Zillow API JSON response
pub fn parse_property_details(raw: &serde_json::Value, address: &str) -> Result<PropertyDetails> {
    let price = raw
        .get("price")
        .and_then(json_whole_number)
        .or_else(|| raw.get("zestimate").and_then(json_whole_number));

    let bedrooms = raw.get("bedrooms").and_then(json_u32);

    let bathrooms = raw
        .get("bathrooms")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);

    let living_area = raw.get("livingArea").and_then(json_u32);

    let year_built = raw.get("yearBuilt").and_then(json_u32);

    let home_type = raw
        .get("homeType")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    let description = raw
        .get("description")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    let zillow_url = raw
        .get("hdpUrl")
        .and_then(|v| v.as_str())
        .map(|url| format!("https://www.zillow.com{}", url));

    Ok(PropertyDetails {
        address: address.to_string(),
        price,
        bedrooms,
        bathrooms,
        living_area,
        year_built,
        home_type,
        description,
        zillow_url,
    })
}

/// Evaluate whether a property meets the given preferences
pub fn evaluate_property(details: &PropertyDetails, prefs: &HomePreferences) -> PropertyEvaluation {
    let mut unmet_criteria = Vec::new();

    if let Some(min_beds) = prefs.min_bedrooms {
        match details.bedrooms {
            Some(beds) if beds >= min_beds => {}
            Some(beds) => {
                unmet_criteria.push(format!("Bedrooms: {} (need at least {})", beds, min_beds))
            }
            None => unmet_criteria.push(format!("Bedrooms: unknown (need at least {})", min_beds)),
        }
    }

    if let Some(max_beds) = prefs.max_bedrooms {
        match details.bedrooms {
            Some(beds) if beds <= max_beds => {}
            Some(beds) => unmet_criteria.push(format!("Bedrooms: {} (max is {})", beds, max_beds)),
            None => unmet_criteria.push(format!("Bedrooms: unknown (max is {})", max_beds)),
        }
    }

    if let Some(min_baths) = prefs.min_bathrooms {
        match details.bathrooms {
            Some(baths) if baths >= min_baths => {}
            Some(baths) => unmet_criteria.push(format!(
                "Bathrooms: {:.1} (need at least {:.1})",
                baths, min_baths
            )),
            None => unmet_criteria.push(format!(
                "Bathrooms: unknown (need at least {:.1})",
                min_baths
            )),
        }
    }

    if let Some(max_price) = prefs.max_price {
        match details.price {
            Some(price) if price <= max_price => {}
            Some(price) => {
                unmet_criteria.push(format!("Price: ${} (max is ${})", price, max_price))
            }
            None => unmet_criteria.push(format!("Price: unknown (max is ${})", max_price)),
        }
    }

    if let Some(min_sqft) = prefs.min_sqft {
        match details.living_area {
            Some(sqft) if sqft >= min_sqft => {}
            Some(sqft) => unmet_criteria.push(format!(
                "Square footage: {} sqft (need at least {} sqft)",
                sqft, min_sqft
            )),
            None => unmet_criteria.push(format!(
                "Square footage: unknown (need at least {} sqft)",
                min_sqft
            )),
        }
    }

    let criteria_checked = [
        prefs.min_bedrooms.is_some(),
        prefs.max_bedrooms.is_some(),
        prefs.min_bathrooms.is_some(),
        prefs.max_price.is_some(),
        prefs.min_sqft.is_some(),
    ]
    .into_iter()
    .filter(|set| *set)
    .count();
    let criteria_met = criteria_checked - unmet_criteria.len();
    let meets_preferences = unmet_criteria.is_empty();

    PropertyEvaluation {
        details: details.clone(),
        meets_preferences,
        unmet_criteria,
        criteria_checked,
        criteria_met,
    }
}

/// Format property details for display
pub fn format_property(evaluation: &PropertyEvaluation) -> String {
    let details = &evaluation.details;
    let mut lines = vec![format!("Address:    {}", details.address)];

    if let Some(home_type) = &details.home_type {
        lines.push(format!("Type:       {}", home_type));
    }
    if let Some(price) = details.price {
        lines.push(format!("Price:      ${}", price));
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

fn main() -> Result<()> {
    let args = Args::parse();

    let prefs = HomePreferences {
        min_bedrooms: args.min_bedrooms,
        max_bedrooms: args.max_bedrooms,
        min_bathrooms: args.min_bathrooms,
        max_price: args.max_price,
        min_sqft: args.min_sqft,
    };

    let client = ZillowClient::new(args.api_key);
    let details = client
        .property_details(&args.address)
        .context("Failed to look up property")?;

    let evaluation = evaluate_property(&details, &prefs);
    println!("{}", format_property(&evaluation));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_property(
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
    fn test_format_property_no_preferences_omits_match_score() {
        let details = make_property(Some(750_000), Some(3), Some(2.0), Some(1500));
        let output = format_property(&evaluate_property(&details, &HomePreferences::default()));
        assert!(!output.contains("Match:"), "{output}");
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
        assert!(eval.unmet_criteria[0].contains("Price"));
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
    fn test_parse_property_details_full() {
        let raw = json!({
            "price": 750000,
            "bedrooms": 3,
            "bathrooms": 2.0,
            "livingArea": 1600,
            "yearBuilt": 1985,
            "homeType": "SINGLE_FAMILY",
            "description": "A lovely home.",
            "hdpUrl": "/homedetails/123-main-st/12345_zpid/"
        });
        let details = parse_property_details(&raw, "123 Main St").unwrap();
        assert_eq!(details.price, Some(750_000));
        assert_eq!(details.bedrooms, Some(3));
        assert_eq!(details.bathrooms, Some(2.0));
        assert_eq!(details.living_area, Some(1600));
        assert_eq!(details.year_built, Some(1985));
        assert_eq!(details.home_type, Some("SINGLE_FAMILY".to_string()));
        assert_eq!(details.description, Some("A lovely home.".to_string()));
        assert!(
            details
                .zillow_url
                .unwrap()
                .starts_with("https://www.zillow.com")
        );
    }

    #[test]
    fn test_parse_property_details_uses_zestimate_when_no_price() {
        let raw = json!({
            "zestimate": 600000,
            "bedrooms": 2
        });
        let details = parse_property_details(&raw, "456 Elm St").unwrap();
        assert_eq!(details.price, Some(600_000));
    }

    #[test]
    fn test_parse_property_details_empty() {
        let raw = json!({});
        let details = parse_property_details(&raw, "789 Oak Ave").unwrap();
        assert_eq!(details.address, "789 Oak Ave");
        assert!(details.price.is_none());
        assert!(details.bedrooms.is_none());
    }

    #[test]
    fn test_parse_property_details_accepts_whole_number_floats() {
        let raw = json!({
            "price": 750000.0,
            "bedrooms": 3.0,
            "livingArea": 1600.0,
            "yearBuilt": 1985.0
        });
        let details = parse_property_details(&raw, "123 Main St").unwrap();
        assert_eq!(details.price, Some(750_000));
        assert_eq!(details.bedrooms, Some(3));
        assert_eq!(details.living_area, Some(1600));
        assert_eq!(details.year_built, Some(1985));
    }

    #[test]
    fn test_parse_property_details_float_zestimate_used_when_no_price() {
        let raw = json!({ "zestimate": 600000.0 });
        let details = parse_property_details(&raw, "456 Elm St").unwrap();
        assert_eq!(details.price, Some(600_000));
    }

    #[test]
    fn test_parse_property_details_rejects_fractional_and_invalid_numbers() {
        let raw = json!({
            "price": 750000.5,
            "bedrooms": 2.5,
            "livingArea": -1600.0,
            "yearBuilt": -1985
        });
        let details = parse_property_details(&raw, "123 Main St").unwrap();
        assert!(details.price.is_none());
        assert!(details.bedrooms.is_none());
        assert!(details.living_area.is_none());
        assert!(details.year_built.is_none());
    }

    #[test]
    fn test_parse_property_details_u32_overflow_is_none_not_truncated() {
        // 2^32 + 3 would truncate to 3 with an `as u32` cast.
        let too_big = u64::from(u32::MAX) + 4;
        let raw = json!({
            "bedrooms": too_big,
            "livingArea": too_big as f64,
            "yearBuilt": too_big
        });
        let details = parse_property_details(&raw, "123 Main St").unwrap();
        assert!(details.bedrooms.is_none());
        assert!(details.living_area.is_none());
        assert!(details.year_built.is_none());
    }

    #[test]
    fn test_parse_property_details_u32_max_is_kept() {
        let raw = json!({ "bedrooms": u32::MAX, "livingArea": f64::from(u32::MAX) });
        let details = parse_property_details(&raw, "123 Main St").unwrap();
        assert_eq!(details.bedrooms, Some(u32::MAX));
        assert_eq!(details.living_area, Some(u32::MAX));
    }

    #[test]
    fn test_property_details_float_numbers_from_mock_server() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"price": 650000.0, "bedrooms": 3.0, "livingArea": 1600.0, "yearBuilt": 1985.0}"#)
            .create();

        let client = ZillowClient::with_base_url("test-key".to_string(), server.url());
        let details = client
            .property_details("123 Main St")
            .expect("mock server returns valid property JSON");

        assert_eq!(details.price, Some(650_000));
        assert_eq!(details.bedrooms, Some(3));
        assert_eq!(details.living_area, Some(1600));
        assert_eq!(details.year_built, Some(1985));
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
        assert!(output.contains("750000"));
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

    #[test]
    fn test_property_details_success_against_mock_server() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::UrlEncoded(
                "address".into(),
                "123 Main St".into(),
            ))
            .match_header("x-rapidapi-key", "test-key")
            .match_header("x-rapidapi-host", "zillow-com1.p.rapidapi.com")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "price": 650000,
                    "bedrooms": 3,
                    "bathrooms": 2.0,
                    "livingArea": 1600
                })
                .to_string(),
            )
            .create();

        let client = ZillowClient::with_base_url("test-key".to_string(), server.url());
        let details = client
            .property_details("123 Main St")
            .expect("mock server returns valid property JSON");

        mock.assert();
        assert_eq!(details.address, "123 Main St");
        assert_eq!(details.price, Some(650_000));
        assert_eq!(details.bedrooms, Some(3));
        assert_eq!(details.living_area, Some(1600));
    }

    #[test]
    fn test_property_details_non_2xx_status_is_error() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(429)
            .with_body("rate limited")
            .create();

        let client = ZillowClient::with_base_url("test-key".to_string(), server.url());
        let err = client
            .property_details("123 Main St")
            .expect_err("a 429 response must be an error");

        let message = err.to_string();
        assert!(message.contains("429"), "unexpected error: {message}");
        assert!(
            message.contains("rate limited"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn test_property_details_non_json_body_is_error() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body("<html>not json</html>")
            .create();

        let client = ZillowClient::with_base_url("test-key".to_string(), server.url());
        let err = client
            .property_details("123 Main St")
            .expect_err("a non-JSON body must be an error");

        assert!(
            err.to_string()
                .contains("Failed to parse Zillow API response"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_with_base_url_tolerates_trailing_slash() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body("{}")
            .create();

        let client =
            ZillowClient::with_base_url("test-key".to_string(), format!("{}/", server.url()));
        client
            .property_details("123 Main St")
            .expect("trailing slash in base URL still reaches /property");

        mock.assert();
    }
}
