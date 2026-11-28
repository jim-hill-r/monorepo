use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use homer::evaluate::{HomePreferences, evaluate_property, parse_bathroom_count};
use homer::quota;
use homer::report::{format_json, format_property};
use homer::zillow::{DEFAULT_BASE_URL, ZillowClient};

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

fn main() -> Result<()> {
    let args = Args::parse();

    let prefs = HomePreferences {
        min_bedrooms: args.min_bedrooms,
        max_bedrooms: args.max_bedrooms,
        min_bathrooms: args.min_bathrooms,
        max_price: args.max_price,
        min_sqft: args.min_sqft,
    };
    prefs.validate()?;

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
