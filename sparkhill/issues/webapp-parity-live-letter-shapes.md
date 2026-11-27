---
kind: todo
priority: priority
tags: [agent-generated]
epic: Blue Eel parity
order: 10
added: 2026-11-27
---

Draw the live app's own letter shapes. blue.eel.education traces
hand-recorded patterns, one `patterns/<letter>/master.json` per letter in its
public `eel3-data` S3 bucket; `src/patterns.json` holds Hershey font
approximations instead (PR #709), so letters look different from the live
app. Copy the 26 lowercase `master.json` files into `src/patterns.json` once
(same format: `path` points with `type` start/end, and `dimensions` with the
guide lines in pixels), so the app still fetches nothing at runtime, and
rescale them to the canvas guide lines if their `dimensions` differ. Keep the
`tracing_check` test that no letter passes as another.
