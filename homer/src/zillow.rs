//! Look a property up on Zillow through RapidAPI and read the details homer
//! uses from the response.
//!
//! [`ZillowClient`] sends one request per lookup and keeps the RapidAPI
//! free-plan quota with [`crate::quota`], stopping before a request the plan
//! can't cover. [`parse_property_details`] turns the raw JSON into
//! [`PropertyDetails`], and fails when the response holds no property.

use std::path::PathBuf;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::quota::{self, QuotaState};

/// Property details returned by the Zillow API
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
pub struct PropertyDetails {
    /// The address as the user typed it.
    pub address: String,
    /// The address of the property Zillow matched, which can differ from
    /// `address` when Zillow matches loosely.
    pub matched_address: Option<String>,
    /// Zillow's property id.
    pub zpid: Option<u64>,
    pub price: Option<u64>,
    pub bedrooms: Option<u32>,
    pub bathrooms: Option<f32>,
    pub living_area: Option<u32>,
    pub year_built: Option<u32>,
    pub home_type: Option<String>,
    pub description: Option<String>,
    pub zillow_url: Option<String>,
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
pub(crate) fn bathroom_count(value: f64) -> Option<f32> {
    // Narrowing is intended: bathroom counts are small, and the finiteness
    // check below catches values that overflow to infinity.
    #[allow(clippy::cast_possible_truncation)]
    let count = value as f32;
    (count.is_finite() && count >= 0.0).then_some(count)
}

/// The response fields that show the Zillow API found a property: `zpid`
/// and every field homer reads.
const PROPERTY_FIELDS: [&str; 10] = [
    "zpid",
    "price",
    "zestimate",
    "bedrooms",
    "bathrooms",
    "livingArea",
    "yearBuilt",
    "homeType",
    "description",
    "hdpUrl",
];

/// Fail when a successful response holds no property.
///
/// RapidAPI can answer 200 with `{}` or `{"message": "..."}` when Zillow has
/// no match for the address. Without this check homer would print just the
/// typed address and call every criterion "unknown", as if it had found a
/// house. A field that is present but has a value homer can't use still
/// counts as a property, so the report shows it as unknown.
fn ensure_property_found(raw: &serde_json::Value, address: &str) -> Result<()> {
    let found = raw
        .as_object()
        .is_some_and(|fields| PROPERTY_FIELDS.iter().any(|key| fields.contains_key(*key)));
    if found {
        return Ok(());
    }
    let reason = ["message", "error"]
        .iter()
        .find_map(|key| raw.get(*key).and_then(|v| v.as_str()))
        .map(|text| format!(" Zillow API said: {}", summarize_error_body(text)))
        .unwrap_or_default();
    anyhow::bail!("Zillow found no property for \"{address}\".{reason}")
}

/// The address Zillow matched, from its `address` object
/// (`streetAddress`, `city`, `state`, `zipcode`), as
/// "street, city, state zipcode". Missing or blank parts are left out.
fn matched_address(raw: &serde_json::Value) -> Option<String> {
    let part = |key: &str| {
        raw.get("address")
            .and_then(|a| a.get(key))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
    };
    let state_zip = [part("state"), part("zipcode")]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    let parts: Vec<&str> = [part("streetAddress"), part("city")]
        .into_iter()
        .flatten()
        .chain((!state_zip.is_empty()).then_some(state_zip.as_str()))
        .collect();
    (!parts.is_empty()).then(|| parts.join(", "))
}

/// Zillow's property id, sent as a number or a string of digits.
fn zpid(raw: &serde_json::Value) -> Option<u64> {
    let value = raw.get("zpid")?;
    json_whole_number(value).or_else(|| value.as_str()?.trim().parse().ok())
}

/// Whether two addresses are the same once case, spacing and punctuation
/// are ignored, so "123 main st." matches "123 Main St".
#[must_use]
pub fn same_address(a: &str, b: &str) -> bool {
    let key = |s: &str| {
        s.chars()
            .filter(char::is_ascii_alphanumeric)
            .map(|c| c.to_ascii_lowercase())
            .collect::<String>()
    };
    key(a) == key(b)
}

/// Parse property details from the raw Zillow API JSON response.
///
/// Fails when the response holds no property (see [`ensure_property_found`]).
pub fn parse_property_details(raw: &serde_json::Value, address: &str) -> Result<PropertyDetails> {
    ensure_property_found(raw, address)?;

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
        matched_address: matched_address(raw),
        zpid: zpid(raw),
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;

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
    fn test_parse_property_details_empty_is_not_found() {
        let err = parse_property_details(&json!({}), "789 Oak Ave")
            .expect_err("an empty response holds no property");
        assert_eq!(
            err.to_string(),
            "Zillow found no property for \"789 Oak Ave\"."
        );
    }

    #[test]
    fn test_parse_property_details_message_only_is_not_found() {
        let raw = json!({ "message": "  No   property found " });
        let err = parse_property_details(&raw, "789 Oak Ave")
            .expect_err("a message-only response holds no property");
        assert_eq!(
            err.to_string(),
            "Zillow found no property for \"789 Oak Ave\". Zillow API said: No property found"
        );
        let raw = json!({ "error": "Invalid address" });
        let err = parse_property_details(&raw, "a").expect_err("error-only response");
        assert!(
            err.to_string()
                .ends_with("Zillow API said: Invalid address"),
            "{err}"
        );
    }

    #[test]
    fn test_parse_property_details_non_object_is_not_found() {
        for raw in [json!([]), json!(null), json!("text"), json!(42)] {
            assert!(parse_property_details(&raw, "a").is_err(), "{raw}");
        }
    }

    #[test]
    fn test_parse_property_details_zpid_only_is_found() {
        let details = parse_property_details(&json!({ "zpid": 12345 }), "789 Oak Ave")
            .expect("a zpid means Zillow found the property");
        assert_eq!(details.address, "789 Oak Ave");
        assert!(details.price.is_none());
        assert!(details.bedrooms.is_none());
    }

    pub(crate) fn zillow_address(street: &str) -> serde_json::Value {
        json!({
            "zpid": 2077,
            "bedrooms": 3,
            "address": {
                "streetAddress": street,
                "city": "Seattle",
                "state": "WA",
                "zipcode": "98101"
            }
        })
    }

    #[test]
    fn test_parse_property_details_reads_matched_address_and_zpid() {
        let typed = "123 Main St, Seattle, WA 98101";
        let details = parse_property_details(&zillow_address("123 Main St"), typed).expect("found");
        assert_eq!(details.address, typed);
        assert_eq!(
            details.matched_address.as_deref(),
            Some("123 Main St, Seattle, WA 98101")
        );
        assert_eq!(details.zpid, Some(2077));
    }

    #[test]
    fn test_parse_property_details_zpid_as_string() {
        let details = parse_property_details(&json!({ "zpid": "2077" }), "a").expect("found");
        assert_eq!(details.zpid, Some(2077));
    }

    #[test]
    fn test_parse_property_details_partial_or_missing_address_object() {
        let no_object = parse_property_details(&json!({ "bedrooms": 2 }), "a").expect("found");
        assert_eq!(no_object.matched_address, None);
        assert_eq!(no_object.zpid, None);

        let street_only = json!({ "bedrooms": 2, "address": { "streetAddress": "5 Pine Rd" } });
        let details = parse_property_details(&street_only, "a").expect("found");
        assert_eq!(details.matched_address.as_deref(), Some("5 Pine Rd"));

        let blank = json!({ "bedrooms": 2, "address": { "streetAddress": " ", "city": "" } });
        let details = parse_property_details(&blank, "a").expect("found");
        assert_eq!(details.matched_address, None);
    }

    #[test]
    fn test_same_address_ignores_case_spacing_and_punctuation() {
        assert!(same_address("123 Main St., Seattle", "123 main st seattle"));
        assert!(!same_address("123 Main St", "1230 Main St"));
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
            .with_body(r#"{"zpid": 1}"#)
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
            .with_body(r#"{"zpid": 1}"#)
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
            .with_body(r#"{"zpid": 1}"#)
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
            .with_body(r#"{"zpid": 1}"#)
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
            .with_body(r#"{"zpid": 1}"#)
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
            .with_body(r#"{"zpid": 1}"#)
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
