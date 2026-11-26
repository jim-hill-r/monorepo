# Decisions

Repo-wide decisions, one per file: the context, what was decided, and what
follows from it. Records are numbered and never rewritten. A later record can
supersede an earlier one. Decisions that belong to one project live in that
project's own `docs/decisions/` folder (for example
[bear_down](../../../bear_down/docs/decisions/README.md)).

Earlier repo-wide decisions about the move to mise are listed in
[../mise.md](../mise.md#decisions).

| # | Decision | Status |
| --- | --- | --- |
| [0001](0001-deploy-feedback-loop.md) | Preview builds, recorded smoke UAT and real-user review on free tiers | Accepted |
| [0002](0002-human-issues.md) | Manual tasks for a human live in HUMAN_ISSUES.md, with a weekly digest | Accepted |
| [0003](0003-eval-engine.md) | Evaluate product requirements at every stage (CI, CD, observability, chaos, judge, human) on free tiers | Accepted |
| [0004](0004-database-engine.md) | Files first, committed SQLite next, D1 when a hosted database is needed | Accepted |
| [0005](0005-browser-testing.md) | Keep Playwright, but run it on a browser we already have | Accepted |
| [0007](0007-infrastructure-as-code.md) | Infrastructure as code: Rust synthesizes Terraform JSON, OpenTofu applies it | Proposed |
