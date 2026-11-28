---
kind: todo
priority: backlog
tags: [agent-generated, agent-ignore]
epic: "jimhillr.com / Phase 1: find where the domain lives"
order: 10
added: 2026-11-28
---

Add `standards/domains.toml` (starting with jimhillr.com), a `chores domains` command and a weekly `all-domains.yml` workflow with workflow_dispatch that look up each domain's registrar and expiry (RDAP) and its NS, MX, TXT and CAA records, write a run summary, and fail when a recorded registrar or nameserver changes. Read-only public data, no secret. Its first run answers whether the DNS is at Squarespace or AWS. Plan: [2026-11-jimhillr-com.md](../standards/docs/plans/2026-11-jimhillr-com.md), decision [0031](../standards/docs/decisions/0031-jimhillr-com.md). Remove agent-ignore from the epic once Jim accepts 0031.
