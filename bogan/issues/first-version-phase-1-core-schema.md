---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Bogan first version / Phase 1: schema and listings"
order: 20
added: 2026-11-28
---

Create `bogan/core` (Rust crate) with the listing and seller schema from the [product brief](../docs/plans/2026-11-product-brief.md): hops with stage, who, where, evidence and `checked`/`verified` records (never proof files), the two chain rules (a named `making` hop, no hidden hops: the chain ends at the hop that reaches the buyer, unknowns written as `unknown`), and the transparency summary. Validate every file in `bogan/listings/` in `mise run ci`, add three sample listings clearly marked as examples, write the curator's checklist in `bogan/docs/curation.md`, and add the seller-data surface (published names and towns, consent, removal) to `docs/threat.toml`. Tests for each rule are cargo evals for the phase 1 requirements.
