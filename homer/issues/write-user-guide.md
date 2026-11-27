---
kind: todo
priority: priority
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write homer's user guide at `homer/docs/user/README.md`, for people running the CLI. It needs the four sections in [User docs](../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (install it, set the RapidAPI key on the free plan, look up an address, score a property against preferences), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from homer/README.md. In the same PR, remove `homer` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0016](../../standards/docs/decisions/0016-user-docs.md).
