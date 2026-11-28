---
kind: todo
priority: backlog
tags: [agent-generated]
epic: "Revenue / Phase 3: meter, shadow billing and public ledger"
order: 30
added: 2026-11-27
---

Revenue phase 3 (standards/docs/plans/2026-11-revenue-402.md), after phase 2. Add a Rust `meter` crate that writes one Analytics Engine data point per API call (project, route, caller `first_party`, `third_party` or `paid`, list price in micro-dollars), a `[[price]]` table in `docs/cost.toml` checked by `cost check`, monthly internal charges for first-party use on the statement (no money moves), and the public ledger page on a public `/open` path of the pane Worker (decision 0012; the Access bypass is a credential change through `infra-credentials`). Keep metering under 1% of the 100,000 Analytics Engine writes a day and list it in each project's `docs/cost.toml`. $0, no new vendor.
