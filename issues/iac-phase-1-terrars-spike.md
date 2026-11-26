---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 10
added: 2026-11-26
---

Decision 0007, item 2: spike `terrars` on the Auth0 provider. Create the `infra` project (Rust, with `opentofu` pinned in its `mise.toml`), generate typed bindings for `auth0/auth0` with `terrars-generate` (only the resource types we use if the whole provider is too big), and record build time in CI and in a cloud sandbox and whether `tofu validate` accepts the output. Adopt `terrars` if it passes, else fall back to hand-written serde structs (A2); write the result into the decision folder as a short note linked from 0007. Bugs found in `terrars` are fixed upstream, with a git dependency on the fixed commit meanwhile (Jim, 2026-11-26).
