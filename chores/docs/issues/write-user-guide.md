---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write chores's user guide at `chores/docs/user/README.md`, for agents and maintainers who run its commands. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (run each command, read its output and exit codes, know which skill calls it), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from chores/README.md. In the same PR, remove `chores` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
