---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 50
added: 2026-11-27
---

Add the Cargo scanner behind `royalty scan`: run `cargo metadata --locked --offline` in the consumer's project, list every package with whether it is direct or transitive and whether it is reached only through dev or build dependencies, and read a `royalty.toml` from beside each package's manifest in the local package sources. Group packages by `family`, use the newest version's terms, and apply `reach`. Make no network call. Test against small fixture workspaces committed under `cli/tests/fixtures/` with path dependencies that ship a `royalty.toml`.
