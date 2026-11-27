---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write psy_ents's user guide at `psy_ents/docs/user/README.md`, for readers: children and the adults reading with them. It needs the four sections in [User docs](../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (find and read the book, and know what age it suits), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from psy_ents/README.md. In the same PR, remove `psy_ents` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0016](../../standards/docs/decisions/0016-user-docs.md).
