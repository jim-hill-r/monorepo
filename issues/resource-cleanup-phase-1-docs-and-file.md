---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 1: claims, guards and a read-only sweep"
order: 10
added: 2026-11-26
---

Resource cleanup phase 1 (see the [plan](../standards/docs/plans/2026-11-resource-cleanup.md)). Add `standards/docs/resources.md` with the rules (claims, retiring with `[[retired]]`, keeping with `[[keep]]`, the safe deletion order, temporary resources) and a short "renaming a resource" checklist, linked from `standards/docs/README.md` and `free-tiers.md`. Add `standards/resources.toml` with empty `[[keep]]` and `[[retired]]` lists, and an AGENTS.md rule: renaming or removing a resource retires the old one in `standards/resources.toml` in the same PR.
