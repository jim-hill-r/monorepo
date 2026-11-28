---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Resource cleanup / Phase 4: Auth0 and DNS"
order: 20
added: 2026-11-26
---

Resource cleanup phase 4 (see the [plan](../plans/2026-11-resource-cleanup.md)). Once a DNS zone exists on the Cloudflare account, make the sweep fail on any record pointing at a `*.pages.dev` or `*.workers.dev` name we don't own (subdomain takeover), and add DNS records and custom domains as inbound references in the safe order.
