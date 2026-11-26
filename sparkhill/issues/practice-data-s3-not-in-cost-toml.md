---
kind: todo
priority: backlog
tags: [agent-generated]
added: 2026-11-26
---

The cost check's unknown-host warning (COST-009) found that the webapp's
practice page fetches `https://eel3-data.s3.us-east-2.amazonaws.com/...`
(`webapp/src/components/practice.rs`). AWS S3 is not a vendor on the
[tech radar](../../standards/docs/tech-radar.md), and
[docs/cost.toml](../docs/cost.toml) doesn't list it. Find out who owns that
bucket and what it costs. Then either move the files into the repo or onto
Cloudflare (Pages serves static JSON for free), or propose AWS in a decision
record for Jim, with its cost, and add it to `docs/cost.toml`.
