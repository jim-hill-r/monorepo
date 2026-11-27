---
kind: todo
priority: priority
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write bear_down's user guide at `bear_down/docs/user/README.md`, for Bears fans using the site. It needs the four sections in [User docs](../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (read the week's rooting list and what the ranking means, use the playoffs tab, vote on games), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from bear_down/README.md. In the same PR, remove `bear_down` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../standards/docs/decisions/0017-user-docs.md).
