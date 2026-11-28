---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Backups / Phase 1: the repo outside GitHub"
order: 10
added: 2026-11-28
---

Build the nightly repo backup from [decision 0030](../../standards/docs/decisions/0030-backups-and-disaster-recovery.md), part 2. Add the Workers KV namespace `jimhillr-backups` to the `cloudflare` IaC stack, a KV-write token scoped to that account to `infra/scripts/mint.sh` (a credential change: Jim approves the `infra-credentials` run), and a `backups` GitHub environment holding it. Add `.github/workflows/backups-nightly.yml`: a full clone, `git bundle create --all`, chunks under 25 MiB written as `repo/<date>/<n>` (35-day expiry) and `repo/latest/<n>` (no expiry) through a `chores backup` command (Rust, no new .sh). Guard writes with `free-tiers.sh` against `kv_writes_per_day` and `kv_storage_bytes`. Update backups/docs/cost.toml, threat.toml, the user guide, the radar exemplar for Cloudflare Workers, and add the BACK-001 evals. Turn `backups` from a docs project into a Rust project if the command lives here rather than in chores.
