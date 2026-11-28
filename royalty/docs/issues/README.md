# Notes on royalty's issues

## Royalty

Royalty lets the people who build libraries and frameworks say how they
want to be paid, in a `royalty.toml`, and lets a consumer work out what
they owe under tiers and caps without touching their running program. The
design, the file format, the phases, the costs and the open questions are in
[docs/plans/2026-11-royalty-plan.md](../plans/2026-11-royalty-plan.md).
Work the phases in order; within a phase, by `order`.

The plan is **approved** (Jim, 2026-11-27, every recommended option in its
"Decisions for Jim"). An issue with a "Blocked on" or "Waits on" line
waits for what it names; every other issue is workable in phase order.

### Phase 0: agree the design

Jim has answered. Both issues are docs PRs.

### Phase 1: MVP scanner and report

A Rust CLI that scans a consumer's Cargo dependency graph for
`royalty.toml` files, works out what is owed, and writes a report for
manual payment. Needs Phase 0; Jim said go ahead (decision 11). The first issue
turns the project from a docs project into a Rust one. Each issue is one
PR.

### Phase 2: honor code and reports

Backlog. Needs Phase 1. Adds terms pinning, the payments ledger,
organization-wide caps, payee checks and the npm scanner.

### Phase 3: metered

Backlog. Needs Phase 2. Counting from CI, deploys and logs. The in-process
counter is built only if Jim says so (decision 9).

### Phase 4: live payment

Backlog. Needs Phase 3 and a payment vendor, which needs Jim's sign-off in
a decision record. Nothing here may add a vendor or a cost before that.
