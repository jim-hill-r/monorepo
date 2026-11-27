---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Eval engine: requirements and feedback"
order: 280
added: 2026-11-26
---

For in-app feedback (eval engine phase 6), add `/feedback/sign-in` (behind Access; returns a signed 12-hour token to the opener by `postMessage`, or to a deep link for desktop and mobile), `POST /api/feedback` (checks the token, validates the comment with `feedback/model`, writes the `feedback.jsonl` line and the screenshot into the open `pane-edit` PR) and `GET /api/feedback?project=&url=` (open pins for a page from the snapshot plus the open `pane-edit` PR), with CORS limited to the projects' `DEPLOY_URL`s and preview hosts. Name `evals/feedback/*.webp` as safe for PR triage alongside `feedback.jsonl`.
