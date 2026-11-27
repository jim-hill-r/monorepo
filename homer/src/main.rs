use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

mod quota;
use quota::QuotaState;

/// Homer - Look up house details on Zillow and score them against your preferences.
#[derive(Parser, Debug)]
#[command(name = "homer")]
#[command(about = "Look up house details on Zillow and evaluate against your preferences")]
struct Args {
    /// The property address to look up (e.g. "123 Main St, Seattle, WA 98101")
    #[arg(short, long)]
    address: String,

    /// RapidAPI key for Zillow API access
    #[arg(long, env = "RAPIDAPI_KEY", hide_env_values = true)]
    api_key: String,

    /// File where homer keeps the RapidAPI quota it last saw, so it can stop
    /// before going past the free plan [default:
    /// $XDG_STATE_HOME/homer/rapidapi-quota.json or
    /// ~/.local/state/homer/rapidapi-quota.json]
    #[arg(long, env = "HOMER_QUOTA_FILE")]
    quota_file: Option<PathBuf>,

    /// Base URL of the Zillow API. Only the CLI transcript tests set it, to
    /// point homer at a local mock server instead of RapidAPI.
    #[arg(long, env = "HOMER_BASE_URL", default_value = DEFAULT_BASE_URL, hide = true)]
    base_url: String,

    /// Minimum number of bedrooms
    #[arg(long)]
    min_bedrooms: Option<u32>,

    /// Maximum number of bedrooms
    #[arg(long)]
    max_bedrooms: Option<u32>,

    /// Minimum number of bathrooms (a non-negative number, such as 1.5)
    #[arg(long, value_parser = parse_bathroom_count)]
    min_bathrooms: Option<f32>,

    /// Maximum listing price
    #[arg(long)]
    max_price: Option<u64>,

    /// Minimum square footage
    #[arg(long)]
    min_sqft: Option<u32>,

    /// Print the property details and the evaluation as one JSON object
    /// instead of the text report
    #[arg(long)]
    json: bool,
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

/// Result of evaluating a property against preferences.
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

/// RapidAPI host for the Zillow API, sent as the `x-rapidapi-host` header.
const RAPIDAPI_HOST: &str = "zillow-com1.p.rapidapi.com";

/// Default base URL for the Zillow API on RapidAPI.
pub const DEFAULT_BASE_URL: &str = "https://zillow-com1.p.rapidapi.com";

/// Zillow API client backed by RapidAPI
pub struct ZillowClient {
    client: Client,
    api_key: String,
    base_url: String,
    /// Where the last RapidAPI quota is kept. `None` turns the quota guard
    /// off, which only tests do.
    quota_file: Option<PathBuf>,
}

impl ZillowClient {
    /// Create a client that sends requests to `base_url` instead of RapidAPI,
    /// for example a local mock server in tests.
    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            quota_file: None,
        }
    }

    /// Keep the RapidAPI quota in `path`: stop before a request when the
    /// saved quota is used up, and save the quota each response reports.
    pub fn with_quota_file(mut self, path: PathBuf) -> Self {
        self.quota_file = Some(path);
        self
    }

    /// Stop if the saved quota is used up and hasn't reset yet.
    fn check_quota(&self) -> Result<()> {
        let Some(path) = &self.quota_file else {
            return Ok(());
        };
        match quota::load(path)? {
            Some(state) => state.ensure_request_allowed(quota::now_secs()),
            None => Ok(()),
        }
    }

    /// Save the quota a response reports and warn when it is nearly used up.
    ///
    /// A response that reports no quota still used a request, so the saved
    /// quota goes down by one instead of staying stale.
    fn record_quota(&self, headers: &reqwest::header::HeaderMap) -> Result<()> {
        let now = quota::now_secs();
        let reported = QuotaState::from_headers(headers, now);
        let state = match (reported, &self.quota_file) {
            (Some(state), _) => state,
            (None, Some(path)) => {
                match quota::load(path)?.and_then(|saved| saved.after_unreported_request(now)) {
                    Some(state) => state,
                    None => return Ok(()),
                }
            }
            (None, None) => return Ok(()),
        };
        if let Some(warning) = state.warning() {
            eprintln!("{warning}");
        }
        if let Some(path) = &self.quota_file {
            quota::save(path, &state)?;
        }
        Ok(())
    }

    /// Look up property details for a given address
    pub fn property_details(&self, address: &str) -> Result<PropertyDetails> {
        self.check_quota()?;

        let url = format!("{}/property", self.base_url);
        let response = self
            .client
            .get(url)
            .header("x-rapidapi-host", RAPIDAPI_HOST)
            .header("x-rapidapi-key", &self.api_key)
            .query(&[("address", address)])
            .send()
            .context("Failed to contact Zillow API")?;

        // Error responses (a 429 especially) report the quota too.
        self.record_quota(response.headers())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!(
                "Zillow API returned status {}: {}",
                status,
                summarize_error_body(&body)
            );
        }

        let raw: serde_json::Value = response
            .json()
            .context("Failed to parse Zillow API response")?;

        parse_property_details(&raw, address)
    }
}

/// How many characters of an error response body to show.
const ERROR_BODY_LIMIT: usize = 300;

/// Shorten an error response body for an error message.
///
/// Collapses runs of whitespace into single spaces and keeps at most
/// [`ERROR_BODY_LIMIT`] characters (never splitting a character), adding
/// " (truncated)" when it cuts. A gateway error page can be long HTML that
/// would otherwise flood the terminal and hide the status.
fn summarize_error_body(body: &str) -> String {
    let collapsed = body.split_whitespace().collect::<Vec<_>>().join(" ");
    match collapsed.char_indices().nth(ERROR_BODY_LIMIT) {
        Some((cut, _)) => format!("{} (truncated)", &collapsed[..cut]),
        None => collapsed,
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

/// A bathroom count as an `f32`, or `None` if it is negative or not finite
/// (including a value too big for an `f32`).
fn bathroom_count(value: f64) -> Option<f32> {
    // Narrowing is intended: bathroom counts are small, and the finiteness
    // check below catches values that overflow to infinity.
    #[allow(clippy::cast_possible_truncation)]
    let count = value as f32;
    (count.is_finite() && count >= 0.0).then_some(count)
}

/// Clap value parser for `--min-bathrooms`: a non-negative, finite number.
fn parse_bathroom_count(arg: &str) -> std::result::Result<f32, String> {
    let value: f64 = arg
        .trim()
        .parse()
        .map_err(|_| format!("`{arg}` is not a number"))?;
    bathroom_count(value).ok_or_else(|| {
        format!("`{arg}` is not a valid bathroom count: use a non-negative number, such as 1.5")
    })
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
        .and_then(bathroom_count);

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
            Some(price) => unmet_criteria.push(format!(
                "Price: {} (max is {})",
                format_dollars(price),
                format_dollars(max_price)
            )),
            None => unmet_criteria.push(format!(
                "Price: unknown (max is {})",
                format_dollars(max_price)
            )),
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

/// Format a whole-dollar amount with thousands separators, such as `$750,000`.
fn format_dollars(amount: u64) -> String {
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
    let mut lines = vec![format!("Address:    {}", details.address)];

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

fn main() -> Result<()> {
    let args = Args::parse();

    let prefs = HomePreferences {
        min_bedrooms: args.min_bedrooms,
        max_bedrooms: args.max_bedrooms,
        min_bathrooms: args.min_bathrooms,
        max_price: args.max_price,
        min_sqft: args.min_sqft,
    };

    let quota_file = args.quota_file.or_else(quota::default_quota_file).context(
        "No place to keep the RapidAPI quota: set --quota-file, HOMER_QUOTA_FILE or HOME",
    )?;
    let client =
        ZillowClient::with_base_url(args.api_key, args.base_url).with_quota_file(quota_file);
    let details = client
        .property_details(&args.address)
        .context("Failed to look up property")?;

    let evaluation = evaluate_property(&details, &prefs);
    if args.json {
        println!("{}", format_json(&evaluation)?);
    } else {
        println!("{}", format_property(&evaluation));
    }

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
    fn test_parse_property_details_negative_bathrooms_is_unknown() {
        let raw = json!({ "bathrooms": -1.5 });
        let details = parse_property_details(&raw, "123 Main St").unwrap();
        assert_eq!(details.bathrooms, None);
    }

    #[test]
    fn test_parse_property_details_bathrooms_too_big_for_f32_is_unknown() {
        // 1e300 is a valid JSON number but becomes infinity as an f32.
        let raw = json!({ "bathrooms": 1e300 });
        let details = parse_property_details(&raw, "123 Main St").unwrap();
        assert_eq!(details.bathrooms, None);
    }

    #[test]
    fn test_parse_property_details_zero_and_half_bathrooms_are_kept() {
        let zero = parse_property_details(&json!({ "bathrooms": 0 }), "a").unwrap();
        assert_eq!(zero.bathrooms, Some(0.0));
        let half = parse_property_details(&json!({ "bathrooms": 1.5 }), "a").unwrap();
        assert_eq!(half.bathrooms, Some(1.5));
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
    fn test_property_details_long_error_body_is_truncated() {
        let mut server = mockito::Server::new();
        let page = format!(
            "<html>\n  <head><title>502 Bad Gateway</title></head>\n  <body>\n{}\n  </body>\n</html>",
            "    <p>upstream proxy error</p>\n".repeat(200)
        );
        let _mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(502)
            .with_body(page)
            .create();

        let client = ZillowClient::with_base_url("test-key".to_string(), server.url());
        let err = client
            .property_details("123 Main St")
            .expect_err("a 502 response must be an error");

        let message = err.to_string();
        assert!(message.contains("502"), "unexpected error: {message}");
        assert!(
            message.contains("<title>502 Bad Gateway</title>"),
            "unexpected error: {message}"
        );
        assert!(
            message.ends_with("(truncated)"),
            "unexpected error: {message}"
        );
        assert!(
            !message.contains('\n'),
            "whitespace not collapsed: {message}"
        );
        assert!(
            message.len() < 400,
            "message too long: {} bytes",
            message.len()
        );
    }

    #[test]
    fn test_summarize_error_body_keeps_short_body() {
        assert_eq!(summarize_error_body("rate limited"), "rate limited");
    }

    #[test]
    fn test_summarize_error_body_collapses_whitespace() {
        assert_eq!(
            summarize_error_body("  bad\n\n   request\t here  "),
            "bad request here"
        );
    }

    #[test]
    fn test_summarize_error_body_empty() {
        assert_eq!(summarize_error_body(""), "");
        assert_eq!(summarize_error_body(" \n\t "), "");
    }

    #[test]
    fn test_summarize_error_body_at_limit_is_not_truncated() {
        let body = "a".repeat(ERROR_BODY_LIMIT);
        assert_eq!(summarize_error_body(&body), body);
    }

    #[test]
    fn test_summarize_error_body_over_limit_is_truncated() {
        let body = "a".repeat(ERROR_BODY_LIMIT + 1);
        let expected = format!("{} (truncated)", "a".repeat(ERROR_BODY_LIMIT));
        assert_eq!(summarize_error_body(&body), expected);
    }

    #[test]
    fn test_summarize_error_body_cuts_on_char_boundary() {
        // Multi-byte characters must not be split.
        let body = "é".repeat(ERROR_BODY_LIMIT + 10);
        let summary = summarize_error_body(&body);
        let kept = summary.trim_end_matches(" (truncated)");
        assert_eq!(kept.chars().count(), ERROR_BODY_LIMIT);
        assert!(kept.chars().all(|c| c == 'é'));
        assert!(summary.ends_with(" (truncated)"));
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

    fn temp_quota_file(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("homer-{name}-{}", std::process::id()));
        std::fs::remove_dir_all(&dir).ok();
        dir.join("rapidapi-quota.json")
    }

    #[test]
    fn test_quota_headers_are_saved_from_mock_server() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("x-ratelimit-requests-limit", "100")
            .with_header("x-ratelimit-requests-remaining", "3")
            .with_header("x-ratelimit-requests-reset", "86400")
            .with_body("{}")
            .create();

        let path = temp_quota_file("quota-saved");
        let client = ZillowClient::with_base_url("test-key".to_string(), server.url())
            .with_quota_file(path.clone());
        client
            .property_details("123 Main St")
            .expect("request with quota left succeeds");

        let saved = quota::load(&path)
            .expect("quota file is readable")
            .expect("quota file was written");
        assert_eq!(saved.limit, Some(100));
        assert_eq!(saved.remaining, 3);
        assert!(saved.resets_at.is_some());
        assert!(saved.warning().is_some(), "3 of 100 left should warn");
        std::fs::remove_dir_all(path.parent().expect("temp dir")).ok();
    }

    #[test]
    fn test_used_up_quota_stops_before_the_next_request() {
        let mut server = mockito::Server::new();
        let first = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("x-ratelimit-requests-limit", "100")
            .with_header("x-ratelimit-requests-remaining", "0")
            .with_header("x-ratelimit-requests-reset", "86400")
            .with_body("{}")
            .expect(1)
            .create();

        let path = temp_quota_file("quota-used-up");
        let client = ZillowClient::with_base_url("test-key".to_string(), server.url())
            .with_quota_file(path.clone());
        client
            .property_details("123 Main St")
            .expect("the last free request still goes through");

        // A second run must stop without contacting the server.
        let client = ZillowClient::with_base_url("test-key".to_string(), server.url())
            .with_quota_file(path.clone());
        let err = client
            .property_details("456 Oak Ave")
            .expect_err("a used-up quota must stop the request");
        assert!(
            err.to_string().contains("used up"),
            "unexpected error: {err}"
        );
        first.assert();
        std::fs::remove_dir_all(path.parent().expect("temp dir")).ok();
    }

    #[test]
    fn test_quota_after_reset_allows_requests() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body("{}")
            .create();

        let path = temp_quota_file("quota-reset");
        let expired = QuotaState {
            limit: Some(100),
            remaining: 0,
            resets_at: Some(quota::now_secs().saturating_sub(1)),
        };
        quota::save(&path, &expired).expect("save expired quota");
        let client = ZillowClient::with_base_url("test-key".to_string(), server.url())
            .with_quota_file(path.clone());
        client
            .property_details("123 Main St")
            .expect("a quota that has reset allows requests");
        mock.assert();
        std::fs::remove_dir_all(path.parent().expect("temp dir")).ok();
    }

    #[test]
    fn test_request_without_quota_headers_counts_against_saved_quota() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body("{}")
            .expect(1)
            .create();

        let path = temp_quota_file("quota-no-headers");
        let last_one = QuotaState {
            limit: Some(100),
            remaining: 1,
            resets_at: Some(quota::now_secs() + 86_400),
        };
        quota::save(&path, &last_one).expect("save quota");
        let client = ZillowClient::with_base_url("test-key".to_string(), server.url())
            .with_quota_file(path.clone());
        client
            .property_details("123 Main St")
            .expect("the last free request still goes through");

        let saved = quota::load(&path)
            .expect("quota file is readable")
            .expect("quota file is kept");
        assert_eq!(saved.remaining, 0, "an unreported request still counts");
        assert_eq!(saved.limit, last_one.limit);
        assert_eq!(saved.resets_at, last_one.resets_at);

        // The next run must stop without contacting the server.
        let client = ZillowClient::with_base_url("test-key".to_string(), server.url())
            .with_quota_file(path.clone());
        let err = client
            .property_details("456 Oak Ave")
            .expect_err("the count went to zero, so the guard stops");
        assert!(err.to_string().contains("used up"), "{err}");
        mock.assert();
        std::fs::remove_dir_all(path.parent().expect("temp dir")).ok();
    }

    #[test]
    fn test_request_without_quota_headers_and_no_saved_quota_writes_nothing() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_body("{}")
            .create();

        let path = temp_quota_file("quota-none-yet");
        let client = ZillowClient::with_base_url("test-key".to_string(), server.url())
            .with_quota_file(path.clone());
        client
            .property_details("123 Main St")
            .expect("request succeeds");
        assert_eq!(quota::load(&path).expect("readable"), None);
    }

    #[test]
    fn test_quota_from_429_response_is_saved() {
        let mut server = mockito::Server::new();
        let _mock = server
            .mock("GET", "/property")
            .match_query(mockito::Matcher::Any)
            .with_status(429)
            .with_header("x-ratelimit-requests-remaining", "0")
            .with_body("quota exceeded")
            .create();

        let path = temp_quota_file("quota-429");
        let client = ZillowClient::with_base_url("test-key".to_string(), server.url())
            .with_quota_file(path.clone());
        client
            .property_details("123 Main St")
            .expect_err("a 429 response is an error");
        let saved = quota::load(&path)
            .expect("quota file is readable")
            .expect("quota from the 429 was saved");
        assert_eq!(saved.remaining, 0);
        std::fs::remove_dir_all(path.parent().expect("temp dir")).ok();
    }
}
