---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write jimhillr's user guide at `jimhillr/docs/user/README.md`, for contributors whose workflows it tests. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (see what workflow_tests checks, read a failure, add a test for a new workflow), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from jimhillr/README.md. In the same PR, remove `jimhillr` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
