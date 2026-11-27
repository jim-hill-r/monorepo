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
| [0002](0002-human-issues.md) | Manual tasks for a human live in HUMAN_ISSUES.md, with a weekly digest | Accepted (file layout superseded by 0006) |
| [0003](0003-eval-engine.md) | Evaluate product requirements at every stage (CI, CD, observability, chaos, judge, human) on free tiers | Accepted |
| [0004](0004-database-engine.md) | Files first, committed SQLite next, D1 when a hosted database is needed | Accepted |
| [0005](0005-browser-testing.md) | Keep Playwright, but run it on a browser we already have | Accepted |
| [0006](0006-one-file-per-issue.md) | One file per issue in `issues/` folders, with a generated list | Accepted |
| [0007](0007-infrastructure-as-code.md) | Infrastructure as code: Rust synthesizes Terraform JSON, OpenTofu applies it | Accepted (state location changed by 0009) |
| [0008](0008-codeql-rust-weekly.md) | Keep the custom CodeQL workflow and add a weekly Rust scan | Accepted |
| [0009](0009-opentofu-state-http-backend.md) | OpenTofu state in a Cloudflare Worker with D1 | Accepted |
| [0010](0010-credential-second-approval.md) | A second approval for credential changes: a GitHub environment only Jim can approve | Accepted |
| [0011](0011-mint-secrets-by-workflow.md) | Mint infra secrets with a workflow, outside OpenTofu | Accepted |
| [0012](0012-revenue-and-http-402.md) | Revenue: open books and GitHub Sponsors first, HTTP 402 (x402) staged | Proposed |
