# 0031. Every web app lands on a jimhillr.com subdomain, with DNS on a free Cloudflare zone

- Status: Proposed (waiting on Jim)
- Date: 2026-11-28
- Asked by: Jim
- Plan: [2026-11-jimhillr-com.md](../plans/2026-11-jimhillr-com.md)
- Builds on: [0021](0021-tenant-scoped-product-data.md) (products are
  separate companies), 0026 (shared live resources get a migration guide)
- Replaces: [bear_down 0010](../../../bear_down/docs/decisions/0010-custom-domain.md)'s
  per-project choice

## Context

Jim (2026-11-28): "I have a jimhillr.com domain where all these pages apps
will eventually need to land. Make a plan for this. I think it lives in
square space but might be in aws."

Today every site is on its own `jimhillr-<project>.pages.dev` address, and
pane and the telemetry endpoint are on `*.jim-hill-r.workers.dev`. The
Cloudflare account has no zone (checked through the Cloudflare API on
2026-11-28). Where `jimhillr.com` is registered and whose nameservers
answer for it isn't recorded anywhere, and the agents' sandbox can't run
DNS or RDAP lookups, so the plan's first step finds out from a GitHub
Actions run.

## Decision (proposed)

1. **One subdomain per site**, named after its Pages project or Worker
   with the `jimhillr-` prefix dropped: `cookbook.jimhillr.com`,
   `bear-down.jimhillr.com`, `sparkhill-webapp.jimhillr.com`,
   `pane.jimhillr.com`, `telemetry.jimhillr.com`. Each is its own origin, so products stay
   separate companies (0021): no cookie is ever set on `.jimhillr.com`,
   and a product promoted to its own domain moves there behind a
   redirect. Jim (2026-11-28): "All projects will start out as a
   subdomain of jimhillr.com with a promotion path to a fully fledged
   product of its own with its own domain." New projects start on their
   subdomain; the promotion criteria belong to the product lifecycle
   (thread "Define the product promotion path to its own domain"), not to
   this decision. The apex and `www` stay as they are until Jim decides
   what goes there.
2. **`jimhillr.com`'s DNS moves to a free Cloudflare zone.** The domain
   stays registered where it is (Squarespace or AWS), so its renewal and
   billing don't change; only the nameservers do. Cloudflare imports the
   current records first, so email and anything else on the domain keeps
   working.
3. **DNS is code.** After the zone exists, OpenTofu in `infra`'s
   cloudflare stack owns every record and custom domain; nobody edits DNS
   by hand.
4. **`pages.dev` and `workers.dev` addresses keep working** after each
   move, and pages point search engines at the new address with a
   canonical link. Nothing is deleted.

## Options

| Option | Money | Free tier | Jim's part | Claude Code effort |
| --- | --- | --- | --- | --- |
| **A. Free Cloudflare zone, nameservers changed at the registrar (recommended)** | $0 more; saves $0.50 a month if the zone is on AWS Route 53 today | One zone on the free plan; Pages custom domains are free (100 per project) and Universal SSL is free | Once: add the zone, check the imported records, change the nameservers (about 20 minutes, runbook) | About 12 small PRs over the phases, a few dollars of the $25 budget |
| B. Keep DNS where it is and add one `CNAME` per site there | $0 more (Route 53 keeps its $0.50 a month) | Same Pages limits | A `CNAME` by hand for every site, now and for each new one | About 9 small PRs, plus a human task per site |
| C. Stay on `pages.dev` | $0 | No change | None | None |

Option B can't serve the Workers (pane, telemetry): a Worker custom domain
needs the domain's zone on Cloudflare. It also leaves DNS outside the
repo, so nothing checks it. Option A has one risk: a record the import
misses breaks what used it (most likely email). The runbook compares the
records before switching, and switching back is the same nameserver
change.

## Consequences

- One more shared live resource, the `jimhillr.com` zone (owner `infra`).
  Each project that uses a subdomain lists it in its `docs/isolation.md`
  with a migration guide: point a new domain at the same Pages project or
  Worker and redirect the old subdomain.
- Sign-in (Auth0 callback and logout URLs, allowed origins) and PostHog's
  allowed hosts change for cookbook and cahokia. Those are credential
  changes and apply only after Jim approves the `infra-credentials` run
  (decision 0010).
- The deploy token gains DNS edit on this one zone through Infra Mint, a
  credential change.
- The dangling DNS check
  (`issues/resource-cleanup-phase-4-dangling-dns.md`) gets a zone to check.
