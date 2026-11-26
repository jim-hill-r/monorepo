---
kind: todo
priority: backlog
tags: [agent-generated]
epic: Cost check
order: 110
added: 2026-11-26
---

Billing checks in the weekly usage workflow, where the vendor's API allows it. Use a separate read-only secret for each: GitHub's billing usage API (fail on any non-zero net amount for the month), and Cloudflare's account subscriptions API with a Billing Read token (fail on any paid subscription, Workers Paid included). Record in `free-tiers.md` which vendors can't be checked by API (PostHog, Auth0, RapidAPI) so the monthly review asks Jim about them.
