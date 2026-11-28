---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write design_system's user guide at `design_system/docs/user/README.md`, for developers building an app on it. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (add it to an app, pick a theme and mode, use the tokens, check contrast), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from design_system/README.md. In the same PR, remove `design_system` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
