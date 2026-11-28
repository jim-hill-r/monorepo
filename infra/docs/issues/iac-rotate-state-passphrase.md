---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 70
added: 2026-11-27
---

Rotate `TOFU_STATE_PASSPHRASE` with Infra Mint (standards/docs/secret-rotation.md). The OpenTofu side is done: passphrase generations, the fallback to the previous key, and `tofu.sh rekey` (see "Rotating the state passphrase" in `infra/README.md`). A plain fallback under fixed names doesn't work, because OpenTofu keys each salt by the key provider's name, and a no-op apply doesn't re-encrypt.

What is left:

1. Pass `TOFU_STATE_PASSPHRASE_OLD` (repository secret) and `TOFU_STATE_KEY_GEN` (repository variable) to `tofu.sh` in `infra-plan.yml`, `infra-apply.yml` and `infra-drift.yml`.
2. In `infra/scripts/mint.sh`, rotating `TOFU_STATE_PASSPHRASE` (pass the current value into the job) writes it to `TOFU_STATE_PASSPHRASE_OLD`, puts a new one in `TOFU_STATE_PASSPHRASE`, and bumps `TOFU_STATE_KEY_GEN`, in that order. Refuse to rotate while any stack's state (the raw `meta` key from `GET /states/<stack>` with the read password) isn't on the current generation's key, so an un-rekeyed stack never falls two generations behind.
3. After a rotation, queue a rekey run in the `infra` environment (it has the write password) that runs `tofu.sh rekey` for every stack in `infra/out/`.
4. Keep `TOFU_STATE_PASSPHRASE_OLD` until the next rotation overwrites it: the state Worker keeps 10 versions, and rolling back past a rotation needs the old key.
5. Add a rekey case to `tofu_state/tests/e2e.sh` against `wrangler dev`, then add `TOFU_STATE_PASSPHRASE` to `ROTATE_AUTO` and update its row in `standards/docs/secret-rotation.md`.
