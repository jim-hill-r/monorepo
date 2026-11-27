---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Resource cleanup / Phase 1: claims, guards and a read-only sweep"
order: 20
added: 2026-11-26
---

Resource cleanup phase 1 (see the [plan](../standards/docs/plans/2026-11-resource-cleanup.md)). Add `standards/scripts/resources.sh check` with RES-001 (a claimed resource that disappears between the PR base and head needs a `[[retired]]` or `[[keep]]` entry; error from day one, in pre-push and PR CI), RES-002 (no tracked file outside `standards/resources.toml` and dated history mentions a retired `url` or `name`; in `//:audit`) and RES-003 (every `upload-artifact` has `retention-days` of 7 or less, except review decks with a comment). Add a PR CI step that labels PRs adding `[[retired]]` or `[[keep]]` entries `needs-human`. Tests in `monorepo/workflow_tests`.
