---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 1: project and model"
order: 10
added: 2026-11-28
---

Create the `feedback` project (technical-platform tier): mise.toml, README, CONTRIBUTING, user guide, docs/cost.toml, docs/threat.toml, docs/requirements.md with FDBK-001 to FDBK-009 from [the plan](../standards/docs/plans/2026-11-user-feedback.md#requirements) (needs-human), and evals.toml. Add the `feedback/model` crate with the user widget's `UserFeedback` (project, build, page, topic, rating, note) and `Vote` types, validated and scrubbed like telemetry's notes, and the review toolbar's `Comment`, `Reply`, `Resolve` and `Anchor` types (URL and route, selector path, element text and role, offset, viewport; or scene, screen and world position and entity for games), the build info, the `feedback.jsonl` line format shared with pane's ratings, and the re-anchoring rules (selector, then text, then position; "moved" when the element is gone), all unit tested. This is also eval engine phase 6's model step, which it replaces.
