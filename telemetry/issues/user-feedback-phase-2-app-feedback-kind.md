---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 2: telemetry app_feedback"
order: 20
added: 2026-11-28
---

Add an `app_feedback` event kind (project, build, page at most 100 characters with no query string, topic, optional rating 1 to 5, optional note at most 500 characters, scrubbed) as [the user feedback plan](../../standards/docs/plans/2026-11-user-feedback.md#what-is-stored-and-where-it-ends-up) says. Keep it when DNT or Sec-GPC is set, since the user pressed Send (TELE-003 wording change, needs-human). Extend telemetry-feedback-export.yml to write `<project>/<yyyy>-week-<nn>.jsonl` for it to the feedback-data branch.
