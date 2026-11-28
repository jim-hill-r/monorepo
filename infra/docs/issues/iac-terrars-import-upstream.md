---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Infrastructure as code / Phase 1: crate, Auth0 and pipeline"
order: 60
added: 2026-11-26
---

When the terrars `import` blocks PR (`infra/upstream/terrars-import-blocks.patch`) has merged upstream, pin `terrars` in `infra/Cargo.toml` to the fixed commit (or the release with it), replace `Synth::import` in `infra/src/synth.rs` with `Stack::add_import`, check `out/` is unchanged, and delete the patch files that are upstream. See `standards/docs/decisions/0007-terrars-spike.md`.
