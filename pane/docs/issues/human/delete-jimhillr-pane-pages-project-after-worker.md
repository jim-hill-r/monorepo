---
added: 2026-11-27
unblocks: "frees one of the 25 budgeted Pages projects and stops jimhillr-pane.pages.dev serving a stale build now that pane runs as the jimhillr-pane Worker"
runbook: standards/docs/runbooks/delete-old-pages-projects.md
---

## Requirement

Pane moved from the Cloudflare Pages project `jimhillr-pane` to the Worker `jimhillr-pane` with static assets (Phase 3 of [pane's plan](../../plans/2026-11-review-dashboard.md)). Once `https://jimhillr-pane.jim-hill-r.workers.dev` serves pane (the first push-main deploy after the PR merges), delete the Pages project `jimhillr-pane` (`jimhillr-pane.pages.dev`) in the Cloudflare dashboard. Delete the Pages project only, never the Worker of the same name.

## Why it can't be automated

It deletes something, and IaC never deletes (decision 0007, item 8); agents also have no Cloudflare write token.

## Runbook

[Delete old Pages projects](../../../../standards/docs/runbooks/delete-old-pages-projects.md), section "Pane's Pages project, after the move to a Worker".
