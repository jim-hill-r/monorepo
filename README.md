# Monorepo

Monorepo for all projects

## Repo health

Each badge shows the latest run on `main` of a check that guards the whole
repo. The "PR triage" scheduled task checks them three times a day and
tries to fix any that is red with a pull request. Badges are served by
GitHub for free and add no workflow runs, so they cost $0. See
[standards/docs/repo-health.md](standards/docs/repo-health.md).

| Area | Status | Runs | Checks |
| --- | --- | --- | --- |
| Main CI/CD | [![Main CI/CD](https://github.com/jimhillr/monorepo/actions/workflows/push-main.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/push-main.yml) | Every push to `main` | Repo audit (`mise run //:audit`: standards, issue files, human-task runbooks, cost check, eval check, security rules, threat check), then CI, evals and deploy for each changed project |
| Nightly CI | [![Nightly CI](https://github.com/jimhillr/monorepo/actions/workflows/nightly-ci.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/nightly-ci.yml) | Daily | Audit, CI and evals in every project, so toolchain and dependency drift shows within a day |
| Nightly CD | [![Nightly CD](https://github.com/jimhillr/monorepo/actions/workflows/nightly-cd.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/nightly-cd.yml) | Daily | CI and deploy of every site, so expired secrets and broken deploys show |
| Nightly downloads | [![Nightly downloads](https://github.com/jimhillr/monorepo/actions/workflows/nightly-downloads.yml/badge.svg?branch=main)](https://github.com/jimhillr/monorepo/actions/workflows/nightly-downloads.yml?query=branch%3Amain) | Daily | macOS and Android builds of the games |
| Code coverage | [![Coverage](https://raw.githubusercontent.com/jimhillr/monorepo/coverage-results/badge.svg)](https://github.com/jimhillr/monorepo/tree/coverage-results) [![Nightly coverage](https://github.com/jimhillr/monorepo/actions/workflows/nightly-coverage.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/nightly-coverage.yml) | Daily | Line coverage of every Rust project with cargo-llvm-cov, total over all projects (COV-001: under 80% is a warning for now) |
| Eval coverage | [![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badge.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json) | Each push to `main` and nightly | Share of every project's requirements met on production evidence, mean over all projects (a project with no requirements counts as 0%), with `must` requirements shown separately. Under 80% of musts met is a warning ([evals.md](standards/docs/evals.md#eval-coverage)) |
| Threats: CodeQL | [![CodeQL](https://github.com/jimhillr/monorepo/actions/workflows/all-codeql.yml/badge.svg?branch=main)](https://github.com/jimhillr/monorepo/actions/workflows/all-codeql.yml?query=branch%3Amain) | Push to `main`, weekly | CodeQL for JavaScript/TypeScript and Actions, plus the weekly Rust scan (decision 0008) |
| Threats: advisories | [![Dependency audit](https://github.com/jimhillr/monorepo/actions/workflows/all-dependency-audit.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/all-dependency-audit.yml) | Daily | cargo-deny and `npm audit` over every lockfile (SEC-003, SEC-004). Warnings only for now, so read the log for advisories |
| Threats: Dependabot | [![Dependabot](https://github.com/jimhillr/monorepo/actions/workflows/dynamic/dependabot/dependabot-updates/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/dynamic/dependabot/dependabot-updates) | Dependabot's schedule | Dependabot's update runs (a red run means a manifest it can't update) |
| PR approvals | [![All Auto Review](https://github.com/jimhillr/monorepo/actions/workflows/all-auto-review.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/all-auto-review.yml) | Every 15 minutes | `jimhillr-reviewer` approves the green agent PRs its rules allow (PR-003, decision 0014) |
| Merge conflicts | [![Merge conflicts](https://github.com/jimhillr/monorepo/actions/workflows/all-merge-conflicts.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/all-merge-conflicts.yml) | Push to `main`, every 3 hours | Labels open PRs that conflict with `main` `merge-conflict` and comments once on an agent's so its thread merges `main` ([pull-requests.md](standards/docs/pull-requests.md#keep-it-mergeable)) |
| Secrets and free tiers | [![Free tier usage](https://github.com/jimhillr/monorepo/actions/workflows/all-free-tier-usage.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/all-free-tier-usage.yml) | Weekly | gitleaks over the full git history (SEC-001), usage against every free-tier limit, and the live threat check |
| Temporary resources | [![All Retire Resources](https://github.com/jimhillr/monorepo/actions/workflows/all-retire-resources.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/all-retire-resources.yml) | Weekly | Deletes Pages previews older than 14 days, old production deployments and branches 7 days after their PR closed ([resources.md](standards/docs/resources.md#temporary-resources)) |
| Eval triage | [![Evals triage](https://github.com/jimhillr/monorepo/actions/workflows/evals-triage.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/evals-triage.yml) | Daily | Files a `FIX` or `TODO` issue for each requirement that is not green and has no open issue, on the `evals-triage` branch for PR triage to open ([evals.md](standards/docs/evals.md#triage)) |
| Fan feedback export | [![Telemetry feedback export](https://github.com/jimhillr/monorepo/actions/workflows/telemetry-feedback-export.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/telemetry-feedback-export.yml) | Weekly | bear_down's fan votes and A/B exposures from Analytics Engine to the [`feedback-data`](https://github.com/jimhillr/monorepo/tree/feedback-data) branch, with a summary per arm |
| Infra apply | [![Infra apply](https://github.com/jimhillr/monorepo/actions/workflows/infra-apply.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/infra-apply.yml) | Push to `main` | OpenTofu plan with the plan guards (IAC-001 to IAC-005), then apply (decision 0007) |
| Infra drift | [![Infra drift](https://github.com/jimhillr/monorepo/actions/workflows/infra-drift.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/infra-drift.yml) | Weekly | Refresh-only plan of every stack: red when a live setting differs from state |
| Infra mint | [![Infra mint](https://github.com/jimhillr/monorepo/actions/workflows/infra-mint.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/infra-mint.yml) | Quarterly, after Jim approves | Rotates the minted infra passwords and tokens ([rotation plan](standards/docs/secret-rotation.md)) |
| tofu_state | [![tofu_state deploy](https://github.com/jimhillr/monorepo/actions/workflows/tofu-state-deploy.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/tofu-state-deploy.yml) | Push to `main` | Deploy of the OpenTofu state Worker (decision 0009) |
| Pane redeploy | [![Pane redeploy](https://github.com/jimhillr/monorepo/actions/workflows/pane-redeploy.yml/badge.svg)](https://github.com/jimhillr/monorepo/actions/workflows/pane-redeploy.yml) | After each eval run on `main` | Redeploys pane so its Requirements section shows the eval results just stored |

# Getting Started

Only [mise](https://mise.jdx.dev/getting-started.html) needs to be installed
system wide. Each project lists its own tools (Rust, Node, dx, wrangler, ...)
in its `mise.toml`, and every project has the same tasks:

```bash
cd <project>
mise install      # install the project's tools
mise run ci       # format check, lint, build and test
mise run fix      # apply formatting and lint fixes
mise run dev      # run it locally
mise run release  # release build
mise run deploy   # deploy, for projects that deploy
```

From anywhere in the repo, `mise run //:audit` checks the repo standards and
`mise run '//...:ci'` runs CI in every project. See
[standards/docs/mise.md](standards/docs/mise.md).

# Contribute

- All issues are contained within the codebase as TODO's. Use todo tree extension to find work.

# Working with Git LFS Files

This repository uses Git LFS for large binary files (PDFs, archives, etc.). By default, LFS files are **not** downloaded automatically during clone to save bandwidth and storage.

To download LFS files when needed:
```bash
# Download all LFS files
git lfs fetch --all
git lfs checkout

# Download specific files
git lfs fetch --include="path/to/file.pdf"
git lfs checkout "path/to/file.pdf"

# Download files matching a pattern
git lfs fetch --include="*.pdf"
git lfs checkout
```
