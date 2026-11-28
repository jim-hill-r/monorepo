---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write telemetry's user guide at `telemetry/docs/user/README.md`, for developers sending events from an app. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (send an event to POST /event, its payload, the rate limit and what is stored), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from telemetry/README.md. In the same PR, remove `telemetry` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
