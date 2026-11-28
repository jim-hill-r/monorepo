---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 1: project and model"
order: 10
added: 2026-11-28
---

Create the `feedback` project (technical-platform tier): mise.toml, README, CONTRIBUTING, user guide, docs/cost.toml, docs/threat.toml, docs/requirements.md with FDBK-001 to FDBK-011 from [the plan](../plans/2026-11-user-feedback.md#requirements) (needs-human), and evals.toml. Add `feedback/model` with `UserFeedback`, `Vote`, `SupportRequest` and `SupportReply`, the email and phone scrubbing moved from telemetry/src/event.rs into a shared function, and the review toolbar's `Comment`, `Reply`, `Resolve` and `Anchor` types with the build info, the `feedback.jsonl` line format shared with pane's ratings and the re-anchoring rules (selector, then text, then position; "moved" when the element is gone), all unit tested. Replaces eval engine phase 6's model issue.
