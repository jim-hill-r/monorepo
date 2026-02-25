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
#[derive(Debug, Deserialize, Serialize, Default)]
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
}

/// Zillow API client backed by RapidAPI
pub struct ZillowClient {
    client: Client,
    api_key: String,
}

impl ZillowClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Look up property details for a given address
    pub fn property_details(&self, address: &str) -> Result<PropertyDetails> {
        let url = "https://zillow-com1.p.rapidapi.com/property";
        let response = self
            .client
            .get(url)
            .header("x-rapidapi-host", "zillow-com1.p.rapidapi.com")
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

/// Parse property details from the raw Zillow API JSON response
pub fn parse_property_details(raw: &serde_json::Value, address: &str) -> Result<PropertyDetails> {
    let price = raw
        .get("price")
        .and_then(|v| v.as_u64())
        .or_else(|| raw.get("zestimate").and_then(|v| v.as_u64()));

    let bedrooms = raw
        .get("bedrooms")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let bathrooms = raw
        .get("bathrooms")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);

    let living_area = raw
        .get("livingArea")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

    let year_built = raw
        .get("yearBuilt")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);

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
            None => {}
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

    let meets_preferences = unmet_criteria.is_empty();

    PropertyEvaluation {
        details: PropertyDetails {
            address: details.address.clone(),
            price: details.price,
            bedrooms: details.bedrooms,
            bathrooms: details.bathrooms,
            living_area: details.living_area,
            year_built: details.year_built,
            home_type: details.home_type.clone(),
            description: details.description.clone(),
            zillow_url: details.zillow_url.clone(),
        },
        meets_preferences,
        unmet_criteria,
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
}
