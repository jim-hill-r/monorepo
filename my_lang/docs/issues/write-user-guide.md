---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write my_lang's user guide at `my_lang/docs/user/README.md`, for programmers writing my_lang code. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (install the compiler, write and run a first program, use pure functions and side effects, read a compile error), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from my_lang/README.md. In the same PR, remove `my_lang` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
