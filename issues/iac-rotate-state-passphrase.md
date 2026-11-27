---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 70
added: 2026-11-27
---

Make `TOFU_STATE_PASSPHRASE` rotatable (standards/docs/secret-rotation.md). OpenTofu's state encryption takes a `fallback` method: synth a second key provider from a `tofu_state_passphrase_old` variable as the fallback. Then the rotation goes like this: Infra Mint moves the current value to `TOFU_STATE_PASSPHRASE_OLD` and generates a new one, the next apply on `main` re-encrypts every stack with the new key, and a later run deletes the old secret. Add it to `ROTATE_AUTO` in `infra/scripts/mint.sh` once a test shows a plan and apply read old-key state and write new-key state. Until then the passphrase is rotated only after a leak, by hand.
