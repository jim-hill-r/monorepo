# 0001. A static web UI where each person brings their own RapidAPI key

- Status: Proposed (waiting on Jim)
- Date: 2026-11-28
- Asked by: Jim
- Roadmap: [roadmap.md](../roadmap.md)

## Context

Jim (2026-11-28): "Can we add to homers product roadmap a web ui? As a cli
I don't think it's very useful."

homer today is a CLI that looks a house up through RapidAPI's
`zillow-com1` API on its **free plan only** (AGENTS.md: it "must never cost
money"; no scheduled runs; no workflow may use the key, FT-002). The free
plan gives one subscriber a small monthly request quota (the number is on
the API's pricing page and in the `x-ratelimit-requests-limit` header), and
`src/quota.rs` stops homer before it goes past it.

A web UI changes who sends those requests. Three things constrain it:

1. **The quota is per key.** If the site used one key (Jim's), every
   visitor would spend the same small quota, and since a paid plan is
   forbidden, the site would stop working for everyone within days of any
   real use.
2. **Zillow's terms.** `zillow-com1` is a third-party API on RapidAPI, not
   Zillow's own. Zillow's Terms of Use forbid scraping and reusing its data,
   and its official data feeds (Bridge Interactive, the Zestimate API) are
   approval-only and require attribution and forbid storing or
   redistributing the data. We can't make that risk go away, but we can
   keep it small: never store Zillow data on a server we run, never show
   one person's lookups to another, always attribute and link back to the
   listing on Zillow, and let each person be the RapidAPI subscriber who
   accepts that API's terms for their own lookups.
3. **Products are separate companies** (decision 0021) and every site lands
   on its own `<name>.jimhillr.com` origin (decision 0031, proposed), so
   homer's site has its own Pages project and origin, and no request passes
   through another product.

## Options

Money is the monthly cost to us. "1x" is today's use (one person, Jim, a
handful of lookups a month); 5x and 100x are that many regular users.
Cloudflare's numbers are in `standards/radar/limits.toml`.

| Option | 1x | 5x | 100x | Free-tier pressure | Build effort |
| --- | --- | --- | --- | --- | --- |
| **A. Bring your own key (recommended)**: a static Dioxus site; scoring runs in the browser; a person types listing facts by hand or looks an address up with their own RapidAPI key, kept only in their browser | $0 | $0 | $0 | Pages static requests are free and unlimited. If RapidAPI refuses browser calls (CORS), a Pages Function on homer's own origin forwards each call with the caller's key, about 3 Worker requests a lookup: under 1% of the account's 100,000 a day even at 100x. Each user spends only their own RapidAPI quota. | 6 issue files, about 6 to 8 small agent PRs, a few dollars of the $25 a month Claude budget |
| B. One shared key in a Worker secret | $0 | $0, but the quota runs out about 5 times as fast | Not possible on the free plan: the quota is gone in days and the only fix is a paid plan, which is forbidden | RapidAPI quota hits 100% at 5x to 100x; Workers stay low | About the same as A, plus a secret, its rotation row and a rate limiter |
| C. Manual entry only, no lookup | $0 | $0 | $0 | None | 3 to 4 PRs; the least useful, since typing every fact is the chore homer exists to remove |

At 5x or 100x as many *projects* on the same Cloudflare account, A still
stays free: static assets don't count against any limit, and its
Function, if it needs one, is a small share of the shared 100,000 Worker
requests a day. The one real shared limit is Pages' 500 builds a month,
which homer doesn't use (it deploys by direct upload).

## Decision (proposed)

Option A, built in this order (the phases in the
[roadmap](../roadmap.md)):

1. Move scoring and the report into a `homer_core` crate with no network
   code, so the CLI and the web app share it.
2. Ship a static web app, `homer/web` (Dioxus, like cookbook and cahokia),
   deployed as the Pages project `jimhillr-homer`, on
   `homer.jimhillr.com` once decision 0031 lands. It starts with manual
   entry (option C) so it is useful before any lookup works.
3. Add lookup with the person's own key: the key is held in the browser
   (session only, or remembered on that device if they tick a box), sent
   only to RapidAPI, and guarded by the same quota rules as the CLI.
4. Then the shortlist and comparison.

Rules that come with it:

- No server-side RapidAPI key, ever. Option B needs a new decision record.
- No Zillow data stored on a server or in the repo beyond test fixtures;
  every listing shown says "Data from Zillow via RapidAPI" and links to
  Zillow.
- If a forwarding Function is needed, it is a pass-through on homer's own
  origin: it stores and logs nothing (no key, no address), forwards only to
  `zillow-com1.p.rapidapi.com`, and is rate limited per IP so it can't be
  used as an open proxy.
- No accounts and no database; preferences and the shortlist live in the
  browser and can be exported as JSON.

## Consequences

- homer costs $0 at any number of users, and the RapidAPI rule in
  AGENTS.md holds without change: Jim's key is never used by the site.
- Each user has to sign up at RapidAPI for a free key before lookups work.
  The manual entry path and a short guide keep that from blocking anyone.
- Until Zillow or RapidAPI change their terms, the legal risk sits with
  the third-party API; if either objects, lookup is removed and manual
  entry stays.
- A switch to option B later means a Worker secret, a rotation row, a
  `docs/cost.toml` change and Jim's approval, and still can't scale past
  the free quota.
