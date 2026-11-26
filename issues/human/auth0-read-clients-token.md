---
added: 2026-11-26
unblocks: "the weekly Auth0 orphan URL check (resource cleanup phase 4, standards/docs/plans/2026-11-resource-cleanup.md)"
---

In the Auth0 dashboard, create a machine-to-machine application authorized for the Management API with only the `read:clients` scope, and save its client ID and secret as the GitHub Actions secrets `AUTH0_SWEEP_CLIENT_ID` and `AUTH0_SWEEP_CLIENT_SECRET`. Weekly use is about 4 of the free plan's 1,000 machine tokens a month.
