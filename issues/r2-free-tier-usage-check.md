---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 70
added: 2026-11-26
---

Decision 0009, item 6: add R2 to the weekly `all-free-tier-usage.yml` check: storage of the `jimhillr-infra-state` bucket and Class A and B operations this month against `r2_storage_bytes` in `standards/radar/limits.toml`, and fail if any bucket other than `jimhillr-infra-state` exists. Needs a read-only token that can list R2 usage; add the human issue for it first.
