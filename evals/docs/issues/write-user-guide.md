---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write evals's user guide at `evals/docs/user/README.md`, for project owners. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (write requirements and evals, run them, read a verdict and the eval coverage badge, act on a red result), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from evals/README.md. In the same PR, remove `evals` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
