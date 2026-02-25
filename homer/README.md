# Homer

A CLI tool for looking up and evaluating house details from Zillow. Provide an address to get
property details, or provide filtering criteria to score how well a property matches your preferences.

## Usage

### Look up a property

```bash
homer --address "123 Main St, Seattle, WA 98101" --api-key YOUR_API_KEY
```

### Score a property against your preferences

```bash
homer --address "123 Main St, Seattle, WA 98101" \
  --min-bedrooms 3 \
  --min-bathrooms 2 \
  --max-price 800000 \
  --api-key YOUR_API_KEY
```

## Options

| Option | Description |
|--------|-------------|
| `--address` | The property address to look up (required) |
| `--api-key` | RapidAPI key for Zillow API access (or set `RAPIDAPI_KEY` env var) |
| `--min-bedrooms` | Minimum number of bedrooms |
| `--max-bedrooms` | Maximum number of bedrooms |
| `--min-bathrooms` | Minimum number of bathrooms |
| `--max-price` | Maximum listing price |
| `--min-sqft` | Minimum square footage |

## API Key

Homer uses the Zillow API via [RapidAPI](https://rapidapi.com/apimaker/api/zillow-com1).
Sign up for a free or paid plan and set your API key via:

- The `--api-key` flag
- The `RAPIDAPI_KEY` environment variable
