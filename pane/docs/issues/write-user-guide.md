---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write pane's user guide at `pane/docs/user/README.md`, for Jim, its one user. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (read each page (projects, work, human issues, reviews), acknowledge merged PRs, open the source behind a number), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from pane/README.md. In the same PR, remove `pane` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
