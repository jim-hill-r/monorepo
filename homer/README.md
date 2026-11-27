# Homer

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/homer.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

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

Homer prints the property details, a match score (how many of the criteria you
set the property meets) and each criterion it misses:

```text
Address:    123 Main St, Seattle, WA 98101
Type:       SINGLE_FAMILY
Price:      $750,000
Bedrooms:   3
Bathrooms:  2.5
Sqft:       1850
Year Built: 1998

Match:      2 of 3 criteria met
✗ This property does not meet all your preferences:
  - Square footage: 1850 sqft (need at least 2000 sqft)
```

Prices print with thousands separators. A value Zillow doesn't report counts
as not met. With no criteria set, it leaves the score out.

If Zillow has no property for the address (RapidAPI answers with no property
fields), homer stops with an error that says so, quoting RapidAPI's message
if it sent one, rather than printing an empty report.

### Print the result as JSON

Add `--json` to print the property details and the evaluation as one JSON
object on stdout instead of the text report, for scripts and evals:

```bash
homer --address "123 Main St, Seattle, WA 98101" --min-bedrooms 3 --json
```

```json
{
  "details": {
    "address": "123 Main St, Seattle, WA 98101",
    "price": 750000,
    "bedrooms": 3,
    "bathrooms": 2.5,
    "living_area": 1850,
    "year_built": 1998,
    "home_type": "SINGLE_FAMILY",
    "description": null,
    "zillow_url": "https://www.zillow.com/homedetails/123-Main-St-Seattle-WA-98101/1_zpid/"
  },
  "meets_preferences": true,
  "unmet_criteria": [],
  "criteria_checked": 1,
  "criteria_met": 1
}
```

A value Zillow doesn't report is `null`. Warnings, such as a low RapidAPI
quota, go to stderr, so stdout is always just the JSON.

## Options

| Option | Description |
|--------|-------------|
| `--address` | The property address to look up (required) |
| `--api-key` | RapidAPI key for Zillow API access (or set `RAPIDAPI_KEY` env var) |
| `--quota-file` | Where homer keeps the RapidAPI quota it last saw (or set `HOMER_QUOTA_FILE`); see [Free quota](#free-quota) |
| `--min-bedrooms` | Minimum number of bedrooms |
| `--max-bedrooms` | Maximum number of bedrooms |
| `--min-bathrooms` | Minimum number of bathrooms (a non-negative number, such as 1.5) |
| `--max-price` | Maximum listing price |
| `--min-sqft` | Minimum square footage |
| `--json` | Print the details and evaluation as one JSON object instead of the text report |

## API Key

Homer uses the Zillow API via [RapidAPI](https://rapidapi.com/apimaker/api/zillow-com1).
Sign up for the free plan and set your API key via:

- The `--api-key` flag
- The `RAPIDAPI_KEY` environment variable

When RapidAPI answers with an error status, homer stops with that status and
the start of the response body, whitespace collapsed and cut to about 300
characters, so a long gateway error page doesn't flood the terminal.

## Free quota

homer must stay on RapidAPI's free plan, so it never makes a request past the
plan's monthly quota. Every RapidAPI response reports the quota in the
`x-ratelimit-requests-limit`, `x-ratelimit-requests-remaining` and
`x-ratelimit-requests-reset` headers. homer saves the latest values to
`$XDG_STATE_HOME/homer/rapidapi-quota.json` (or
`~/.local/state/homer/rapidapi-quota.json`; change it with `--quota-file`) and:

- prints a warning when 10% of the quota or less is left (5 requests if
  RapidAPI doesn't send the limit),
- stops with an error, without contacting RapidAPI, once the quota is used up,
  until the reset time RapidAPI gave has passed,
- counts a request whose response reports no quota as one request used, so
  the saved count never goes stale.

homer replaces the quota file in one step (it writes a temporary file and
renames it), so an interrupted run can't leave a half-written file behind.

If the quota file is ever corrupt, homer stops rather than guessing; delete the
file to start over.
