# homer user guide

## Who it's for

homer is for anyone house hunting who wants a quick, repeatable check of a
listing from the command line. Give it an address and it looks the property
up on Zillow (through RapidAPI) and prints the price, bedrooms, bathrooms,
size, year built and a Zillow link. Give it your preferences as well, such as
at least 3 bedrooms and at most $800,000, and it tells you how many of them
the house meets and which ones it misses.

homer is free to use. It runs on RapidAPI's free plan and stops itself
before it could ever go past that plan's monthly quota.

## Getting started

1. Install Rust if you don't have it: follow <https://rustup.rs>.
2. Install homer:

   ```bash
   cargo install --git https://github.com/jimhillr/monorepo homer
   ```

   This builds homer and puts it on your `PATH` (in `~/.cargo/bin`). From a
   clone of the repository, `cargo install --path homer` does the same.
3. Get a RapidAPI key on the free plan (see
   [How to set your RapidAPI key](#how-to-set-your-rapidapi-key)).
4. Look up your first address:

   ```bash
   homer --address "123 Main St, Seattle, WA 98101"
   ```

`homer --help` lists every option.

## How to set your RapidAPI key

1. Sign in or sign up at <https://rapidapi.com>.
2. Open the [Zillow API](https://rapidapi.com/apimaker/api/zillow-com1) and
   subscribe to its **free** plan. Never pick a paid plan: homer is
   meant to cost nothing.
3. Copy your key (`X-RapidAPI-Key`) from the API's page.
4. Give it to homer in one of two ways:
   - set it once in your shell: `export RAPIDAPI_KEY=your-key` (add the line
     to `~/.bashrc` or `~/.zshrc` to keep it), or
   - pass it on each run: `homer --api-key your-key --address "..."`.

homer never prints your key, not even in `--help`.

## How to look up an address

Run homer with the full address, in quotes:

```bash
homer --address "123 Main St, Seattle, WA 98101"
```

homer prints what Zillow knows about the property:

```text
Address:    123 Main St, Seattle, WA 98101
Type:       SINGLE_FAMILY
Price:      $750,000
Bedrooms:   3
Bathrooms:  2.5
Sqft:       1850
Year Built: 1998
```

When there is no list price, the price shown is Zillow's estimate. Each
lookup uses one request of your free quota.

## How to score a property against your preferences

Add any of these options to the lookup:

| Option | Meaning |
| --- | --- |
| `--min-bedrooms 3` | at least 3 bedrooms |
| `--max-bedrooms 5` | at most 5 bedrooms |
| `--min-bathrooms 1.5` | at least 1.5 bathrooms |
| `--max-price 800000` | price at most $800,000 (no `$` or commas) |
| `--min-sqft 2000` | at least 2,000 square feet |

```bash
homer --address "123 Main St, Seattle, WA 98101" \
  --min-bedrooms 3 --min-bathrooms 2 --min-sqft 2000
```

Under the details, homer adds a match score and each preference the house
misses:

```text
Match:      2 of 3 criteria met
✗ This property does not meet all your preferences:
  - Square footage: 1850 sqft (need at least 2000 sqft)
```

A value Zillow doesn't report counts as not met. If you set both
`--min-bedrooms` and `--max-bedrooms`, the minimum can't be above the
maximum: homer stops and tells you, without using a request.

## How to save results for a script or spreadsheet

Add `--json` to get the details and the score as one JSON object instead of
the text report:

```bash
homer --address "123 Main St, Seattle, WA 98101" --min-bedrooms 3 --json > house.json
```

Warnings go to the screen (stderr), so the file holds only the JSON. A
value Zillow doesn't report is `null`.

## Getting help

Common problems:

- **"the following required arguments were not provided: --api-key"**: homer
  has no key. Set `RAPIDAPI_KEY` or pass `--api-key` (see above).
- **"Zillow found no property for ..."**: Zillow couldn't match the address.
  Check the spelling and include the city, state and ZIP code.
- **"warning: only N RapidAPI free requests left this period"**: you are
  close to the free plan's monthly limit. homer still works until it runs
  out.
- **"The RapidAPI free quota ... is used up"**: homer has stopped so it never
  goes past the free plan. Wait until the time it gives. If RapidAPI gave no
  reset time, check your usage on RapidAPI's dashboard, then delete the
  quota file (`~/.local/state/homer/rapidapi-quota.json`, or the path you
  set with `--quota-file`) to try again.
- **"Zillow API returned status 401" or "403"**: the key is wrong, or you
  aren't subscribed to the Zillow API's free plan. Copy the key again and
  check the subscription.
- **"Zillow API returned status 429"**: RapidAPI is limiting requests. Wait a
  minute and try again.
- **"invalid value ... for '--min-bathrooms'"**: use a plain number of zero
  or more, such as `1.5`.
- **The quota file is corrupt**: delete it, and homer starts a fresh count.

homer has no feedback button. To report a problem or ask for a feature, tell
Jim with the command you ran and what homer printed (leave your API key
out), and it will be recorded as an issue in `homer/docs/issues/`.
