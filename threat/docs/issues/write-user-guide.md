---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write threat's user guide at `threat/docs/user/README.md`, for project owners. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (write a project's docs/threat.toml, run the threat check and read its THR findings), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from threat/README.md. In the same PR, remove `threat` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
