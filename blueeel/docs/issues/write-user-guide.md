---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write blueeel's user guide at `blueeel/docs/user/README.md`, for learners and teachers using the reading app. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (open the app, start and finish a lesson, track progress), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from blueeel/README.md. In the same PR, remove `blueeel` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
