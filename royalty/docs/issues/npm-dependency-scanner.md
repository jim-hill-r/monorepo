---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Royalty / Phase 2: honor code and reports"
order: 60
added: 2026-11-27
---

Add an npm scanner beside the Cargo one: read `package-lock.json`, tell direct from transitive and `devDependencies` from the rest, and read `royalty.toml` from each package in `node_modules`. Document that a producer must list `royalty.toml` in the package's `files`.
