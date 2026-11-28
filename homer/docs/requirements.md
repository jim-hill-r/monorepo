# Homer requirements

What homer promises the person who runs it to check a house. The format and
the rules are in [standards/docs/evals.md](../../standards/docs/evals.md);
the evals that check each requirement are in [evals.toml](../evals.toml).

Drafted by an agent on 2026-11-27 from the README, `docs/cost.toml`,
`docs/threat.toml`, the issue files and the RapidAPI rules in `AGENTS.md`.
homer is a CLI run by hand on Jim's machine: it has no deploy, and no
workflow may use the RapidAPI key, so its evals run against a mock of the
Zillow API, never the live one.

## HOME-001: An address looks up the property's details

- Priority: must
- Why: Jim wants the key facts of a house he is looking at without opening
  Zillow.
- Accept: `homer --address "<address>"` with an API key prints the address,
  type, price (the Zestimate when there is no list price), bedrooms,
  bathrooms (keeping halves, such as 2.5), square footage, year built and a
  Zillow link; a value Zillow doesn't report is left out of the text and is
  `null` in JSON, never a wrong or truncated number.

## HOME-002: A property is scored against Jim's preferences

- Priority: must
- Why: The point of homer is to say quickly whether a house fits what Jim
  is looking for.
- Accept: with any of `--min-bedrooms`, `--max-bedrooms`,
  `--min-bathrooms`, `--max-price` and `--min-sqft`, homer prints how many
  of the criteria set are met (`Match: 2 of 3 criteria met`) and each one
  missed with the property's value and the limit; a value Zillow doesn't
  report counts as not met; with no criteria set, the score is left out.

## HOME-003: Results can be read by scripts

- Priority: should
- Why: Scripts and evals compare houses from homer's output.
- Accept: with `--json`, stdout is exactly one JSON object with `details`
  and the evaluation (`meets_preferences`, `unmet_criteria`,
  `criteria_checked`, `criteria_met`); unknown values are `null`; warnings
  go to stderr, never stdout.

## HOME-004: homer never costs money

- Priority: must
- Why: RapidAPI is approved only on its free plan (AGENTS.md); going past
  the quota could bill or block the account.
- Accept: homer saves the quota RapidAPI reports after each request; warns
  when 10% or less is left (5 requests when the limit is unknown); refuses
  to send a request once the quota is used up until the reset time passes;
  counts a request with no quota headers as one used; stops on a corrupt
  quota file instead of guessing; and the RapidAPI subscription stays on
  the free plan with no payment method.

## HOME-005: The API key is never shown

- Priority: must
- Why: The key is Jim's own credential; printing it would leak it into
  terminals, logs and screenshots.
- Accept: `--help` and every error message leave out the key's value, even
  when it is set in `RAPIDAPI_KEY`.

## HOME-006: Mistakes and API failures give a clear message

- Priority: should
- Why: Jim runs homer now and then and shouldn't have to read code to see
  what went wrong.
- Accept: a missing API key or an invalid option (such as a negative or
  `NaN` `--min-bathrooms`) exits with status 2 and says what to fix;
  preferences no property can meet (`--min-bedrooms` above
  `--max-bedrooms`) exit with status 1, name both flags and make no
  RapidAPI request; an
  error status from RapidAPI shows the status and the start of the body,
  whitespace collapsed and cut to about 300 characters; a reply that isn't
  JSON is an error, not a crash.
