---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 35
added: 2026-11-26
---

Decision 0009: build the `jimhillr-tofu-state` Worker (Rust, `workers-rs`, as its own project) that serves OpenTofu's `http` backend at `/states/<stack>` with state in the D1 database `jimhillr-tofu-state`: `GET` (read password or write password), `POST` (write password, and only by the current lock holder when locked), `LOCK` and `UNLOCK` (write password; `409` with the holder's lock info when held; unlock only by the same lock ID), keep the last 10 versions per stack, constant-time password checks, and no state contents in logs. Deploy with wrangler through `free-tiers.sh` (one Worker of the budget), copying `TOFU_STATE_READ_PASSWORD` and `TOFU_STATE_WRITE_PASSWORD` into Worker secrets from GitHub. Test it end to end with real `tofu init`, `plan` and `apply` against `wrangler dev`, including two concurrent applies where one gets the lock error. Add the project's `docs/cost.toml` (Cloudflare Workers and D1) and Workers and D1 radar exemplars. Needs `issues/human/tofu-state-passwords.md` before its first deploy, not to build it.
