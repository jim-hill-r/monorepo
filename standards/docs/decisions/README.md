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
| [0012](0012-revenue-and-http-402.md) | Revenue: a public financial ledger and GitHub Sponsors first, HTTP 402 (x402) staged | Accepted |
| [0013](0013-credit-window-scheduler.md) | Start work by the Claude usage window, checked every hour | Accepted |
| [0014](0014-faster-pr-approvals.md) | Faster PR approvals: an Approve page in pane, one approval per stack, a `jimhillr-reviewer` account for low-risk PRs and post-merge acknowledgement | Accepted |
| [0015](0015-code-quality-process.md) | A standing code quality process: one standard, a ratchet, triage into issues | Accepted |
| [0016](0016-holding-and-handling-money.md) | Holding and handling money: a separate free account, accounting books in the repo, a merchant of record for card sales | Accepted |
| [0017](0017-user-docs.md) | Every project has a user guide, checked by the audit with a shrinking baseline | Accepted |
| [0018](0018-feature-flags.md) | Feature flags: our own flags Worker on D1, with a management app per project (pane for platform work) where app managers set modes per user without git | Accepted |
| [0019](0019-review-roles.md) | Review roles: who may approve what, as a matrix of domains (cost, threat, requirements, credentials...) and project tiers, in one file | Accepted |
| [0020](0020-user-feedback-and-support.md) | User feedback, support and in-app user guides as platform libraries: each product its own endpoint, data and inbox | Accepted |
| [0021](0021-tenant-scoped-product-data.md) | Product runtime data in one shared D1 database, in tenant-scoped tables with per-tenant budgets and a one-command move out | Accepted |
| [0023](0023-observability.md) | Observability as one project and review domain: Cloudflare logs, a watchdog Worker for checks and alert rules, GitHub Mobile to page Jim's phone (0022 is taken by an open PR) | Accepted |
| [0025](0025-review-deck-demo-videos.md) | Narrated, captioned demo videos in the review decks, recorded from a script with Playwright, Kokoro and ffmpeg | Proposed |
