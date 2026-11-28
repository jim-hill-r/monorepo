---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write infra's user guide at `infra/docs/user/README.md`, for Jim and the agents that change infrastructure. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (read a plan on a PR, approve a credential run in infra-credentials, add a resource, recover from a failed apply), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from infra/README.md. In the same PR, remove `infra` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
