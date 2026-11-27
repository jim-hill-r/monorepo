# Get the RapidAPI Zillow key

- **Kind:** one-time (done; keep for a rebuild)
- **Status:** done. The `rapidapi-free-plan` settings check re-checks the
  plan every 30 days.
- **Time:** about 5 minutes
- **Cost:** $0, free plan only. It must never cost money (AGENTS.md):
  no paid plan, no scheduled runs, nothing that could pass the free quota.

## What it's for

`homer` looks up houses through the [Zillow API on
RapidAPI](https://rapidapi.com/apimaker/api/zillow-com1). It tracks the
quota itself and stops before the free limit.

## Why a person

Signing up and subscribing to a plan needs a person, and the key belongs
to whoever runs homer.

## Before you start

- A RapidAPI account.

## Steps

1. Open the Zillow API page above > Pricing > subscribe to the Basic
   (free) plan. Add no card.
2. Copy the key from the API's Endpoints page (`X-RapidAPI-Key`).
3. Set it where you run homer: `RAPIDAPI_KEY` in your shell, or
   `--api-key`.

## Where secrets go

Only in the environment of whoever runs homer. No workflow uses it, so
there is no GitHub secret, and there must not be one (it would invite a
scheduled run).

## Verify

`homer` returns a house's details and prints the quota left.

## Undo or rotate

rapidapi.com > My Apps > the app > Authorization > add a new key and
delete the old one.
