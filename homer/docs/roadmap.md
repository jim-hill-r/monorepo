# Homer roadmap

Written 2026-11-28 from Jim's ask:

> Can we add to homers product roadmap a web ui? As a cli I don't think
> it's very useful

The mission and vision are in [mission.md](mission.md). The work is issue
files in [`../issues/`](../issues/), with the epic notes in
[`../issues/README.md`](../issues/README.md). How the web UI gets listing
data without costing money or breaking the RapidAPI rule is
[decision 0001](decisions/0001-web-ui-data-source.md) (proposed).

## Where it stands (2026-11-28)

homer is a Rust CLI: give it an address and your preferences and it prints
the house's details from Zillow (through RapidAPI's free plan), a match
score and each criterion it misses, as text or JSON. It runs by hand only;
a quota file stops it before the free plan's limit. `src/main.rs` was split
into `zillow`, `evaluate` and `report` modules (#960); table-driven scoring
(#962) and showing the address Zillow matched (#921) are open PRs. It has
requirements HOME-001 to HOME-006, evals against a mock of the API, and a
user guide. It has no users besides Jim and no web presence.

## Product roadmap

### Now

The web UI, epic "Web UI", phases 1 and 2. Both work without any lookup,
so they don't wait on decision 0001.

1. **A shared core** ([issue](../issues/web-ui-core-crate.md)): scoring and
   the report move into `homer_core`, which the CLI and the browser share.
2. **Score a house by hand on the web**
   ([issue](../issues/web-ui-manual-scorer.md)): `homer.jimhillr.com` (the
   Pages project `jimhillr-homer`), where you set your must-haves once,
   type a house's facts and see what it meets and misses. Preferences stay
   in your browser.

### Next

1. **Look a house up with your own key**
   ([CORS check](../issues/web-ui-cors-check.md),
   [lookup](../issues/web-ui-bring-your-own-key.md)): type an address and
   homer fills in the facts from Zillow, using your own free RapidAPI key,
   kept in your browser. Every listing links back to Zillow.
2. **A shortlist to compare houses** ([issue](../issues/web-ui-shortlist.md)):
   save the houses you score and compare them side by side, sorted by
   match.
3. **Feedback and a findable site**
   ([issue](../issues/web-ui-feedback-and-site-checks.md)): the in-app
   feedback and support path (decision 0020) and the web standard's search,
   agent and speed rules (decision 0028).

### Later

- Weighted preferences ("price matters more than square feet") and a score
  out of 100 instead of a count (no issue yet).
- More criteria Zillow already returns: lot size, year built, HOA fees, days
  on market (no issue yet).
- A shareable link that carries someone's preferences, so a couple can use
  the same must-haves (no issue yet).
- A promotion review toward homer's own domain once it meets the product
  lifecycle gates (decision 0029, proposed) (no issue yet).

## Technical roadmap

### Now

- **Workspace with `homer_core`, `cli` and `web`**
  ([issue](../issues/web-ui-core-crate.md)): `homer_core` has no network or
  file code and builds for `wasm32-unknown-unknown`; the quota rules take
  plain header values so the browser can apply them too.
- **Finish the open CLI work first**: the
  [table-driven criteria](../issues/table-driven-evaluate-property.md)
  (#962) and the [matched address](../issues/show-zillow-matched-address.md)
  (#921), so the core crate moves code that is already in its final shape.

### Next

- **Evals for the web app** ([issue](../issues/web-ui-manual-scorer.md)):
  web journeys as requirements, Playwright tests in CI and a production
  eval of the live page, as cookbook has. Lookup evals use a mock of the
  API; no workflow ever uses a real key (FT-002).
- **A pass-through Function only if the CORS check needs it**
  ([issue](../issues/web-ui-cors-check.md)): it stores and logs nothing,
  forwards only to RapidAPI's Zillow host, and is rate limited.
- **Public-site security**: `_headers` with a strict CSP, the threat model
  updated from "local CLI" to a public static site
  ([issue](../issues/web-ui-manual-scorer.md)).

### Later

- Move to `homer.jimhillr.com` and add its DNS record in `infra` when
  decision 0031's phase 4 reaches homer (no issue yet; that plan files one
  per project).
- Drop the CLI's separate copy of anything the web app proves out, so
  there is one scoring path (no issue yet).

## Cost

Money is the monthly cost to us. 1x is today (Jim, a handful of lookups a
month); 5x and 100x are that many regular users, or that many projects
sharing the Cloudflare account. Limits are in
`standards/radar/limits.toml`.

| Item | 1x | 5x | 100x | Free-tier use |
| --- | --- | --- | --- | --- |
| Cloudflare Pages static site | $0 | $0 | $0 | Static requests are free and unlimited; direct upload uses none of the 500 builds a month |
| Pages Function, only if the CORS check needs one | $0 | $0 | $0 | About 3 Worker requests a lookup: under 1% of the account's 100,000 a day even at 100x users |
| RapidAPI (Zillow data) | $0 | $0 | $0 | Each user spends their own free quota; Jim's key is never used by the site |
| Custom domain | $0 | $0 | $0 | `homer.jimhillr.com` is a subdomain of a domain we already pay for (decision 0031) |
| Claude Code build effort | 6 issues, about 6 to 8 small PRs over several weeks | same | same | A few dollars of the $25 a month budget |

With one shared key instead (decision 0001's option B), 100x is impossible
on the free plan: the quota would run out in days and the only fix is a
paid plan, which AGENTS.md forbids.

## Risks

- **Zillow's terms.** The data comes from a third-party RapidAPI listing,
  not Zillow, and Zillow forbids scraping and reuse. homer keeps the risk
  small (no stored data, no sharing between users, attribution and a link
  back, each user is the API subscriber) and keeps manual entry so the app
  still works if lookup has to go.
- **Users need a RapidAPI account** for lookups. Manual entry and the user
  guide keep that from blocking first use.
- **A forwarding Function could become an open proxy.** It forwards to one
  host only and is rate limited per IP.
