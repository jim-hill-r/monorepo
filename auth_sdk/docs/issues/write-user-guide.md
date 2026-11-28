---
kind: todo
priority: backlog
tags: [agent-generated]
epic: User docs
added: 2026-11-27
---

Write auth_sdk's user guide at `auth_sdk/docs/user/README.md`, for developers adding sign-in to an app. It needs the four sections in [User docs](../../../standards/docs/documentation.md#user-docs): `## Who it's for`, `## Getting started`, a `## How to ...` section (install it, configure an Auth0 tenant, run the sign-in flow on each supported platform, handle errors), and `## Getting help` (common problems and how to report one). Check each step against what ships today and link the guide from auth_sdk/README.md. In the same PR, remove `auth_sdk` from `standards/user-docs.toml` and delete this file; `chores user-docs` (DOC-008) fails until both are done. Why: [decision 0017](../../../standards/docs/decisions/0017-user-docs.md).
