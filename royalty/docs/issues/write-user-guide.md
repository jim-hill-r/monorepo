---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write royalty's user guide at `royalty/docs/user/README.md`, for library authors who want to be paid and the projects that use their code. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (write a royalty.toml, run the royalty command (check, scan, report), read the report), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from royalty/README.md. In the same PR, remove `royalty` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
