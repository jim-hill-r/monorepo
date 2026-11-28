---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "User feedback / Phase 3: in-app user guides"
order: 30
added: 2026-11-28
---

Add `feedback/help`: a build-time helper that compiles a project's docs/user/*.md into the app (pulldown-cmark, raw HTML and remote images dropped, a broken internal link fails the build), and Dioxus `HelpPanel` and `HelpLink` in `feedback/dioxus` (features dioxus06 and dioxus07) with `## Getting help` first, heading search and section deep links. Then add Help to Blue Eel's practice menu showing sparkhill/docs/user. Sends nothing, works offline (FDBK-005, FDBK-009). See [the plan](../plans/2026-11-user-feedback.md#in-app-user-guides).
