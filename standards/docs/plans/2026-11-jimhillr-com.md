# Plan: move every web app onto jimhillr.com

Started 2026-11-28 from Jim's request:

> I have a jimhillr.com domain where all these pages apps will eventually
> need to land. Make a plan for this. I think it lives in square space but
> might be in aws.

Status: **proposed**. The decision is
[0031](../decisions/0031-jimhillr-com.md); the work items are issue files
in the root [issues/](../../../issues/) with `epic: jimhillr.com`, one
group per phase, and the one step only Jim can take is in
[issues/human/](../../../issues/human/). Agents leave the epic alone
(`agent-ignore`) until Jim accepts 0031.

Jim then added (2026-11-28): "All projects will start out as a subdomain
of jimhillr.com with a promotion path to a fully fledged product of its own
with its own domain." This plan covers the subdomain hosting side; the
promotion path and its criteria are in the product lifecycle standard
(decision 0029, `standards/docs/product-lifecycle.md`, its "Promoting"
section).

## Goal

Every site and public Worker is served at `<name>.jimhillr.com`, where
`<name>` is its Pages project or Worker name without `jimhillr-`. DNS
for `jimhillr.com` is code in `infra`, checked like everything else, and
the move costs nothing more than the domain renewal Jim already pays.

## Where things are today

| Site | Today | Lands at |
| --- | --- | --- |
| bear_down | `jimhillr-bear-down.pages.dev` | `bear-down.jimhillr.com` |
| cahokia (web) | `jimhillr-cahokia.pages.dev` | `cahokia.jimhillr.com` |
| controlled_chaos | `jimhillr-controlled-chaos.pages.dev` | `controlled-chaos.jimhillr.com` |
| cookbook | `jimhillr-cookbook.pages.dev` | `cookbook.jimhillr.com` |
| design_system preview | `jimhillr-design-system.pages.dev` | `design-system.jimhillr.com` |
| sparkhill website | `jimhillr-sparkhill.pages.dev` | `sparkhill.jimhillr.com` |
| sparkhill webapp | `jimhillr-sparkhill-webapp.pages.dev` | `sparkhill-webapp.jimhillr.com` |
| pane (Worker) | `jimhillr-pane.jim-hill-r.workers.dev` | `pane.jimhillr.com` |
| telemetry (Worker) | `jimhillr-telemetry.jim-hill-r.workers.dev` | `telemetry.jimhillr.com` |

Not in scope: `blue.eel.education` (the live Blue Eel app, not ours to
move), the `jimhillr-tofu-state` Worker (internal, no public address
needed), and the apex `jimhillr.com` and `www`, which keep whatever they
serve today until Jim decides.

The Cloudflare account has no zone (Cloudflare API, 2026-11-28). Where the
domain is registered and whose nameservers answer for it is unknown: Jim
thinks Squarespace, possibly AWS. The agents' sandbox blocks DNS and RDAP
lookups, so phase 1 answers it from GitHub Actions instead of asking Jim.

## Phases

### Phase 1: find out where the domain lives (agent)

A `chores domains` command and a weekly `all-domains.yml` workflow (plus
`workflow_dispatch`) look up every domain in a new
`standards/domains.toml`: registrar and expiry from RDAP, nameservers,
`MX`, `TXT` and `CAA` from DNS. It writes a run summary and fails when a
listed fact (registrar, nameservers) changes, so a lapse or a hijack shows
up. Read-only, public data, no secret. Its first run tells us whether the
DNS is at Squarespace, Route 53 or elsewhere, and which records (email in
particular) the move must keep. The same check later feeds the dangling
DNS check and the Squarespace domain costs.

### Phase 2: put the zone on Cloudflare (Jim, about 20 minutes)

The one step that needs a person, with a runbook that covers both
registrars:

1. Add `jimhillr.com` to the Cloudflare account on the **Free** plan.
   Cloudflare scans and imports the existing records.
2. Compare the imported records with phase 1's list; add any it missed
   (above all `MX`, SPF/DKIM `TXT` and verification records).
3. At the registrar (Squarespace > Domains > DNS > Nameservers, or AWS >
   Route 53 > Registered domains > Edit name servers), replace the
   nameservers with the two Cloudflare gives.
4. Wait for Cloudflare to show the zone Active (minutes to 24 hours).
5. Only if the old DNS was a Route 53 hosted zone: delete that hosted zone
   a week later to stop its $0.50 a month. IaC never deletes, so this
   stays a person's step.

Why a person: adding a zone and changing nameservers are account-level
actions on a domain Jim pays for, the registrar has no API agents may use,
and agents never hold a credential that can move a domain.

### Phase 3: DNS as code (agent, credential change)

- `infra`'s cloudflare stack reads the zone and owns its records, starting
  with the ones imported in phase 2, so the plan shows no change on its
  first apply.
- Infra Mint adds DNS edit on this one zone to the deploy token's scopes
  (`standards/settings.toml`, `secret-rotation.md`); Jim approves the
  `infra-credentials` run.
- `standards/radar/limits.toml` gets a `cloudflare_zones` limit (budget 1)
  and a `pages_custom_domains_per_project` limit; `infra/docs/cost.toml`
  counts the zone.
- `standards/resources.toml` registers the zone; the isolation standard's
  shared resource table lists it, owner `infra`.
- The dangling DNS check (resource-cleanup phase 4) starts checking it.

### Phase 4: move each site (agent, one PR per site)

Each move, in its project:

1. Add the custom domain in infra (`pages_domain` for a Pages project, a
   Worker custom domain for pane and telemetry) and its `CNAME`.
2. Change `DEPLOY_URL`, the threat model's `[[surface]] url`, the user
   guide, the README and the evals to the new address; keep an eval that
   the old `pages.dev` address still answers.
3. Add a canonical link to the new address.
4. Add the zone to the project's `docs/isolation.md` under
   `## Shared resources`, with a migration guide.
5. cookbook and cahokia also add the new origin to their Auth0
   application's callback, logout and web origin URLs, and cookbook to
   PostHog's allowed hosts: credential changes, applied after Jim
   approves.
6. telemetry first, because bear_down and controlled_chaos post to it:
   its move adds the new origin to their CSP `connect-src` and keeps the
   old one until both have shipped.

Order: telemetry, then bear_down (no sign-in, quickest win), the other
static sites, pane (its Cloudflare Access application moves with it), and
cookbook and cahokia last because of sign-in.

### Phase 5: tidy up (agent)

- Naming standard: a section for `jimhillr.com` subdomains next to the
  Pages names, saying every new project starts on its subdomain, and an audit rule that a site's `DEPLOY_URL` is on
  `jimhillr.com` once it has moved.
- The `pages.dev` and `workers.dev` addresses stay, since nothing breaks
  by keeping them. Retiring any of them later follows
  `standards/docs/resources.md`.

## Cost

| Item | Money | Free-tier use |
| --- | --- | --- |
| Cloudflare zone on the Free plan | $0 | 1 zone |
| Pages custom domains and Universal SSL | $0 | 7 domains across 7 projects, limit 100 per project |
| Worker custom domains | $0 | 2 |
| Domain renewal | unchanged (already paid at Squarespace or AWS) | none |
| Route 53 hosted zone, if that's where DNS is today | saves $0.50 a month once deleted | none |
| Claude Code | about 12 small PRs over the phases, a few dollars | counts against the $25 a month budget |

## Cost at scale

Projects now: 27 (`ls */docs/cost.toml | wc -l`, 2026-11-28). Implemented
everywhere means every project has a site on its own subdomain: one Pages
project (or Worker) and one DNS record each, plus about 5 records for
email and verification. Limits: [limits.toml](../radar/limits.toml); the
DNS record limit is Cloudflare's for free zones created since September
2024 (200), to add to limits.toml in phase 3.

| Scale | Projects | Money a month | Tightest free limit | Claude Code effort |
| --- | --- | --- | --- | --- |
| Now | 27 | $0: the zone, records, custom domains and certificates are free, and the renewal is already paid | Pages projects, 27 of 100 (27%), over the budget of 25, so Jim raises it; DNS records 32 of 200 (16%) | About 12 small PRs once (a few dollars), then about $0.10 a month for each new project's subdomain |
| 5x | 135 | $0, as above | Pages projects (and Workers scripts), past the account's 100. DNS records 140 of 200 (70%) | Same, plus moving sites into a second free Cloudflare account; DNS stays in one zone |
| 100x | 2,700 | $0 with a wildcard record and a router Worker, or about $25 a month for Cloudflare Pro (3,500 records) | DNS records, 2,705 of 200; a router Worker would then hit Workers requests (100,000 a day) | About 2 PRs for the router, then about $0.10 a month per new project, and the plan stops rather than pays |

What gives first as it grows: the Pages project limit, which is about
hosting, not DNS; a second free account handles it, since a Pages custom
domain only needs a `CNAME`, not a zone in the same account. DNS records
give out next, near 200 sites. Before paying for Pro we would use one
wildcard record and a routing Worker, and promoted products move to their
own domains anyway (decision 0029), which frees their records.

## Risks

- **A missed record breaks something on the domain** (email most likely).
  Phase 1 records every record first, phase 2 compares before switching,
  and switching back is the same nameserver change.
- **Sign-in breaks if a callback URL is missed.** Both old and new origins
  stay allowed until the move's evals pass.
- **One shared zone across separate companies (0021).** Each subdomain is
  its own origin and no cookie is set on the parent domain; the migration
  guide is the technical half of a promotion to its own domain (point the
  new domain at the same Pages project or Worker, redirect the subdomain).

## Questions for Jim

1. Accept decision 0031 (free Cloudflare zone, one subdomain per site)?
   Recommended: yes.
2. Keep the apex `jimhillr.com` and `www` as they are for now?
   Recommended: yes; decide later what lives there.
