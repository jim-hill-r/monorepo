---
kind: todo
priority: priority
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write sparkhill's user guide at `sparkhill/docs/user/README.md`, for Blue Eel's learners and teachers and the website's visitors. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (use Blue Eel (start reading, follow a lesson, review), find things on the website; one guide can cover both with a page each beside README.md), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from sparkhill/README.md. In the same PR, remove `sparkhill` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
