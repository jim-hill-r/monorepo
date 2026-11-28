---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write tofu_state's user guide at `tofu_state/docs/user/README.md`, for operators running OpenTofu for infra. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (point OpenTofu at the backend, what a lock means and how to clear a stale one, recover state), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from tofu_state/README.md. In the same PR, remove `tofu_state` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
