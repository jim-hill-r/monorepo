---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 36
added: 2026-11-27
---

Deploy the state Worker from GitHub, never by hand: a `workflow_dispatch` job (and on push to `main` when `tofu_state/` changes) in the `infra` environment that runs `mise run deploy` in `tofu_state` with `CLOUDFLARE_API_TOKEN`, `CLOUDFLARE_ACCOUNT_ID`, `TOFU_STATE_READ_PASSWORD` and `TOFU_STATE_WRITE_PASSWORD` from GitHub secrets, then checks `GET /states/<stack>` answers `401` without a password and `204` or `200` with the read password. Then add the Worker's URL to the infra workflows as the backend origin. Test the workflow in `monorepo/workflow_tests`. Needs `issues/human/tofu-state-passwords.md` and `issues/human/cloudflare-api-token-workers-d1-edit.md` first.
