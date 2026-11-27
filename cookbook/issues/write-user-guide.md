---
kind: todo
priority: priority
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write cookbook's user guide at `cookbook/docs/user/README.md`, for cooks using the site. It needs the four sections in [User docs](../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (find a recipe, follow the week's plan, use the shopping list on a phone), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from cookbook/README.md. In the same PR, remove `cookbook` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../standards/docs/decisions/0017-user-docs.md).
