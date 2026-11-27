---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Royalty / Phase 1: MVP scanner and report"
order: 10
added: 2026-11-27
---

Make royalty a Rust project with a `royalty_model` crate (`model/`): serde types for schema 1 of `royalty.toml` (`[project]`, `[license]`, `[[payee]]`, `[terms]`, `[[tier]]`, `[caps]`, `[income]`, `[measure]`, `[consumer]`), a parser that applies the defaults, and validation that returns findings (unknown `schema` is an error, unknown fields are warnings, `terms` required unless `binding` is `voluntary`, at least one method per payee, no value that looks like a secret). No I/O. Tests: every `royalty.toml` in the monorepo parses as "no payment terms", and each worked example in the [plan](../docs/plans/2026-11-royalty-plan.md#worked-examples) parses. Switch `mise.toml` to the Rust task file, and update `README.md`, `CONTRIBUTING.md` and `docs/cost.toml`.

Blocked on: decisions 1 to 5 and 11 in the plan.
