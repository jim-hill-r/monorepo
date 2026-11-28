---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write bogan's user guide at `bogan/docs/user/README.md`, for the sellers and buyers it is built for. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (list an item and show where it came from (sellers), browse and read a listing's supply chain (buyers)), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from bogan/README.md. In the same PR, remove `bogan` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
