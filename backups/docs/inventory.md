# What holds state

Every place in the repo's architecture where losing data would hurt, its
recovery tier and how it comes back. The tiers and targets are in
[decision 0030](../../standards/docs/decisions/0030-backups-and-disaster-recovery.md).
A PR that adds a stateful resource (a D1 database, a KV namespace, a data
branch, a new vendor that keeps user data) adds its row here.

| Tier | RPO | RTO |
| --- | --- | --- |
| 0 | 0 while GitHub is up; up to 90 days if it is lost | 1 day to read, 3 days to work |
| 1 | 24 hours | 4 hours |
| 2 | 0 (built from the repo) | 1 day |
| 3 | none kept (re-minted) | 2 days |
| 4 | accepted loss | none |

## GitHub

| What | Owner | Tier | Backup | Restore |
| --- | --- | --- | --- | --- |
| `jimhillr/monorepo`, `main` and every branch: code, docs, issues, and the cookbook content (`cookbook/content/`, the product) | all | 0 | Git history on GitHub; offline copy with Jim every 90 days | From the offline copy into a new repository, then [disaster-recovery.md](../../standards/docs/runbooks/disaster-recovery.md) |
| `feedback-data` branch (weekly fan feedback summaries) | telemetry, bear_down | 0 | In the offline copy if Jim uses `git clone --mirror`; a ZIP of `main` leaves it out | From the copy |
| `evals-results`, `coverage-results` branches | evals | 4 | None needed | Rerun the nightly workflows |
| Repo settings, rulesets, environments, labels | infra | 2 | The `github` IaC stack; by-hand ones in `standards/settings.toml` | `infra-apply`, then the settings runbooks |
| GitHub secrets and variables | infra | 3 | Never backed up | Bootstrap credentials by hand, then Infra Mint ([infra-credentials-environment.md](../../standards/docs/runbooks/infra-credentials-environment.md)) |
| Actions artifacts (D1 exports, review decks) | backups | 1 | They are the backup | - |

## Cloudflare

| What | Owner | Tier | Backup | Restore |
| --- | --- | --- | --- | --- |
| D1 `jimhillr-tofu-state` (OpenTofu state, already encrypted) | tofu_state | 1 | 10 versions per stack in the Worker; Time Travel 7 days; nightly encrypted export (phase 1) | Roll back a version (tofu_state README), Time Travel, or import the export |
| Product D1 data (flags, feedback, support; decisions 0018, 0020, 0021) | each product | 1 | Planned: nightly encrypted export from the day it exists (phase 1) | Time Travel, or import the export |
| Pages projects, Workers, routes, bindings | each project | 2 | The repo (`wrangler.toml`, deploy workflows) and the `cloudflare` stack | Nightly CD and `infra-apply` |
| Analytics Engine (telemetry events), Workers Logs | telemetry, observability | 4 | None | Accept the loss |

## Other vendors

| What | Owner | Tier | Backup | Restore |
| --- | --- | --- | --- | --- |
| Auth0 tenant config (apps, URLs, grants) | infra | 2 | The `auth0` IaC stack | `infra-apply` |
| Auth0 users | auth_sdk | 4 | None (decision 0030, part 6) | Users sign up again |
| PostHog session replays and settings | each product | 4 | Settings in `standards/settings.toml` | Re-enter the settings |
| Squarespace domains | Jim | 2 | `list-squarespace-domains.md` keeps the list; auto-renew and lock by hand | The registrar's own recovery |
| Claude project, connectors, scheduled tasks | jimhillr | 2 | Prompts in `standards/docs/scheduled-tasks/`; [claude-setup.md](../../standards/docs/runbooks/claude-setup.md) | Follow the runbook |

## Accounts

The accounts (GitHub `jimhillr`, `jimhillr-agt`, `jimhillr-reviewer`,
Cloudflare, Auth0, PostHog, Squarespace, Claude) are what every row above
stands on. Their recovery codes are kept by Jim offline
([account-recovery-codes.md](../../standards/docs/runbooks/account-recovery-codes.md)).
