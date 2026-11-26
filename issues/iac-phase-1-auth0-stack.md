---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 20
added: 2026-11-26
---

Decision 0007, items 1 and 3: in `infra`, add the `WebApp` and `Auth0App` constructs and `cargo run -- synth`, which writes `infra/out/auth0.tf.json` with sorted keys. The stack imports the cookbook and cahokia Auth0 applications (`import` blocks) and sets production, preview (`https://*.<project>.pages.dev`) and localhost callback, web origin and logout URLs, and the Single Page Application type. `mise run ci` fails when the committed output differs from a fresh synth; add the committed-output exception to `standards/docs/mise.md`. Use the Pages project names on `main` at the time (the `jimhillr-` rename may have landed).
