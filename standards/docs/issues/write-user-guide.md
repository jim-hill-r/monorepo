---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write standards's user guide at `standards/docs/user/README.md`, for contributors and agents. It needs the four sections in [User docs](../documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (find the rule that applies, run the audit and the hooks, read an error id such as DOC-005 and fix it), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from standards/README.md. In the same PR, remove `standards` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../decisions/0017-user-docs.md).
