---
kind: todo
priority: priority
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write controlled_chaos's user guide at `controlled_chaos/docs/user/README.md`, for players. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (start a game, play a turn with keyboard, mouse or touch, build a deck; link the rulebook rather than copying it), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from controlled_chaos/README.md. In the same PR, remove `controlled_chaos` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
