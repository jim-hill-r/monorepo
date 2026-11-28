---
added: 2026-11-27
runbook: standards/docs/runbooks/squarespace-domains.md
---

## Requirement

List every domain on the Squarespace account: its name, yearly price, next renewal date, whether auto-renew is on, and what it is for (a project, a redirect, or nothing). Also give a Squarespace budget for the tech radar (the yearly total divided by 12, or a little above). Reply in the Claude project and an agent records them as yearly costs in the repo.

Cost tracking phase 4, standards/docs/plans/2026-11-cost-tracking.md. Unblocks `cost/docs/issues/cost-tracking-phase-4-domains-as-costs.md`.

## Why it can't be automated

Squarespace has no public API for domains or billing, and agents must never hold a credential with billing access, so only a person signed in to the account can read the list.

## Runbook

[List the Squarespace domains](../../../../standards/docs/runbooks/squarespace-domains.md)
