---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write athenaeum's user guide at `athenaeum/docs/user/README.md`, for engineers and agents who read the knowledge base. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (find a note, cite a source, add a note the way CONTRIBUTING.md asks), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from athenaeum/README.md. In the same PR, remove `athenaeum` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
