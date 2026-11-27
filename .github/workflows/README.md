# GitHub Actions Workflows

## Repo health badges

Every workflow that runs on `main` by itself (a push to `main` or a
schedule) has a status badge in the "Repo health" table of the root
README, and PR triage checks them three times a day
([standards/docs/repo-health.md](../../standards/docs/repo-health.md)).
A new workflow like that adds its badge in the same PR;
`monorepo/workflow_tests/tests/readme_badges_tests.rs` fails otherwise.

## CI/CD with mise

Every project is a directory with a `mise.toml` that includes one shared task
file from `standards/tasks/`, so every project has the same tasks (`ci`, `fix`,
`build`, `test`, `dev`, `release`, `deploy`). The rules are in
[standards/docs/mise.md](../../standards/docs/mise.md).

### mise-projects.yml (reusable)

Called by the three workflows below.

1. **plan** runs `mise run //:audit` (the repo standards check), then lists
   the projects to run: those affected since the `base` input
   (`mise run //:affected`), or every project when `base` is empty
   (`mise run //:projects`). See `standards/scripts/affected.sh` for what
   counts as affected (files inside the project, shared task files it
   includes, Cargo path dependencies, and `# affected-by:` lines).
   Deploy runs also list every project marked `# deploy-on-every-push`
   (pane, whose snapshot reads the whole repo).
2. **ci (&lt;project&gt;)** runs once per project: installs the project's tools
   with `jdx/mise-action`, restores the Rust cache, runs `mise run ci`, and
   then `mise run evals` (stage `ci` eval results in the job summary and a
   14-day artifact), and `mise run deploy` when the `deploy` input is true.

### evals-store.yml (reusable)

Called by `push-main.yml` and `nightly-ci.yml` after `mise-projects.yml`,
never by pull requests. Its one job downloads every project's
`evals-results-<project>` artifact from the run and commits them to the
long-lived `evals-results` data branch (`standards/scripts/evals.sh store`,
see [standards/docs/evals.md](../../standards/docs/evals.md#results-and-verdicts)).
It is the only CI job with `contents: write`, granted on the calling job
only, and its only push is to `evals-results` (SEC-007). It never fails the
run. It is a separate workflow because a reusable workflow whose job asks
for write fails to start when a caller (pull requests) grants only read.

### pull-request-ci.yml

Runs on every pull request against the projects it changes. Never deploys.
`workflow_dispatch` runs every project on a branch.

A **Secret scan** job runs gitleaks over the PR's commits
(`standards/scripts/secrets-check.sh range`, SEC-001 in
[standards/docs/security.md](../../standards/docs/security.md)).

A **Dependency audit** job runs cargo-deny and `npm audit` for the lockfiles
the PR changes (`standards/scripts/deps-check.sh`, SEC-003 and SEC-004).
Warnings only for now, so CI result doesn't wait for it.

A **Cost summary** job runs `cost summary --base origin/<base>` (the `cost`
project, [standards/docs/cost.md](../../standards/docs/cost.md)) and writes
what the PR changes about cost to the job summary: cost files changed, and
services, usage, money and budgets that moved. It labels the PR `cost` when
anything moved, and `needs-human` when it adds a vendor or money to a project
or raises a budget. Informational, so CI result doesn't wait for it.

A last job, **CI result**, always runs after the projects and the secret scan
and passes only when every project passed and no secret was found. It is the one check the `main` ruleset should
require, so GitHub auto-merge waits for green CI (see the Auto-merge section
of `standards/docs/pull-requests.md`). Rename it only together with that
ruleset.

### push-main.yml

Runs on every push to `main`: `mise run ci` then `mise run deploy` in each
project the push changed. Only the cloudflare projects deploy anything; they
build their web app and publish it with `wrangler pages deploy`, which needs
the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` repository secrets.
Runs have no concurrency group, so every push gets its own CI and deploy;
a group would let GitHub cancel queued runs and skip them.

A project with `DEPLOY_URL` in its `mise.toml` `[env]` deploys in the
`production` environment, so the deploy is listed under Deployments with its
URL, and the job summary shows the production URL, this deployment's URL and
the commit. `workflow_dispatch` with no inputs checks and deploys every
project; with `project` and `ref` it redeploys that one project at that ref,
which is how to roll back (see `standards/docs/mise.md`).

### nightly-ci.yml and nightly-cd.yml

**Nightly CI** runs `mise run ci` in every project at 2 AM Pacific, so new
toolchain or dependency releases show up within a day, even in quiet
projects. **Nightly CD** runs at 2:30 AM in every project that deploys (has
`DEPLOY_URL`), running `mise run ci` then `mise run deploy`, so expired
secrets and broken deploys show up within a day too. It needs the same
Cloudflare secrets as `push-main.yml`. They are two workflows so the README
has a separate badge for each.

Every deploy, here and in `push-main.yml`, runs only after `ci` passed in
the same job, and `standards/scripts/deploy-check.sh` skips it when a newer
commit on main changes the project, so an older run never overwrites a
newer deploy.

Tests: `monorepo/workflow_tests/tests/mise_workflow_tests.rs` and
`deploy_check_tests.rs`.

### nightly-downloads.yml

**Nightly downloads** runs at 2:30 AM Pacific, with Nightly CD, for every
game with `RELEASE_TAG` in its `mise.toml` (`mise run //:downloads`). It runs
`mise run ci`, then `mise run release:macos` on a `macos-15` runner and
`mise run release:android` on Ubuntu, and publishes both to the game's
GitHub prerelease with `standards/scripts/downloads.sh publish`, moving the
tag to the commit built and replacing the assets, so the download links never
change. Pull requests that change the game or its build run the same builds
without publishing; `workflow_dispatch` publishes on demand. Only the publish
job gets `contents: write`. The optional `ANDROID_KEYSTORE_BASE64` and
`ANDROID_KEYSTORE_PASSWORD` secrets sign the APK. See "Game builds" in
`standards/docs/mise.md`.

Tests: `monorepo/workflow_tests/tests/downloads_workflow_tests.rs`.


## Infrastructure as code

`infra-plan.yml` plans every infra stack on PRs that change `infra/`, with
read-only credentials and no lock. `infra-apply.yml` plans on `main` in the
`infra` environment and applies exactly that saved plan, in `infra`, or in
`infra-credentials` after Jim's approval when it changes a credential
(decision 0010). `infra-drift.yml` runs a refresh-only plan every Monday.
`tofu-state-deploy.yml` deploys the OpenTofu state Worker (`tofu_state`,
decision 0009) in `infra`. Every job that touches state on `main` shares the
`infra` concurrency group. Details are in `infra/README.md` and
`tofu_state/README.md`. Tests: `monorepo/workflow_tests/tests/infra_workflows_tests.rs`
and `tofu_state_deploy_tests.rs`.

## pull-request-review-decks.yml (Review decks)

Runs on pull requests that touch `docs/reviews/` (and on `workflow_dispatch`).
Builds every review deck to HTML and PDF with the Marp CLI through
`standards/scripts/review-decks.sh` (also `mise run //:review-decks`) and
uploads them as the `review-decks` artifact. See
[docs/reviews/README.md](../../docs/reviews/README.md).

## all-dependency-audit.yml

Nightly (2:30 AM Pacific) or by hand: runs `standards/scripts/deps-check.sh
--all`, cargo-deny over every `Cargo.lock` and `npm audit` over every
`package-lock.json` (SEC-003, SEC-004 in
[standards/docs/security.md](../../standards/docs/security.md)). Warnings
only for now, so it stays green; findings are in the log.

## all-free-tier-usage.yml

Weekly (Mondays, 7 AM Pacific), by hand, or called from another workflow:
runs `standards/scripts/free-tiers.sh usage`, which reads how many Cloudflare
Pages projects and Workers exist and how much Actions cache and artifact
storage the repo uses, writes a table to the run summary, and fails when
anything reaches its budget in
[standards/docs/free-tiers.md](../../standards/docs/free-tiers.md). It only
reads.

A second job, **Secret scan (full history)**, runs gitleaks over every commit
(`standards/scripts/secrets-check.sh history`), so a new rule catches an old
commit within a week.

Tests: `monorepo/workflow_tests/tests/free_tier_tests.rs`.

## all-codeql.yml

Advanced CodeQL setup ([decision 0008](../../standards/docs/decisions/0008-codeql-rust-weekly.md)),
which replaces GitHub's default setup:

| Language | When | Time |
| --- | --- | --- |
| JavaScript/TypeScript | every PR, every push to `main`, weekly | about 1 minute |
| GitHub Actions | every PR, every push to `main`, weekly | about 40 seconds |
| Rust (`build-mode: none`) | weekly (Sundays 2 AM UTC) and by hand (`workflow_dispatch`) | about 26 minutes |

Rust stays off PRs because a whole-repo Rust scan took about 26 minutes
(PR #382, 2026-01), and the merge rule waits for every check. On PRs, Rust
is covered by clippy (`mise run ci`) and cargo-deny (SEC-003 in
[standards/docs/security.md](../../standards/docs/security.md)). All results
go to the repository's Security tab.

Default setup must stay **off** in Settings > Code security while this file
exists: GitHub runs only one of the two. The workflow needs `contents: read`,
`security-events: write` (to upload results) and `actions: read`.

Tests: `monorepo/workflow_tests/tests/codeql_workflow_tests.rs`.

## all-delete-branches.yml

Deletes remote branches without needing a local clone. Run it manually from the Actions tab (or the `workflow_dispatch` API) with:

- `branches`: branch names separated by spaces or newlines.
- `dry_run`: defaults to `true`, which only reports what would be deleted. Set it to `false` to delete.

The workflow never deletes the default branch and skips any branch that is the head of an open pull request. Branches that no longer exist are reported as not found. The run summary lists how many branches were deleted, skipped, missing, or failed.

### Permissions

- `contents: write` - To delete branch refs
- `pull-requests: read` - To find branches with open pull requests

## pull-request-agent-skills.yml

Validates the agent skills in `.agents/skills/` against the
[Agent Skills specification](https://agentskills.io/specification) on pull
requests that touch `.agents/`, `.claude/`, `AGENTS.md` or `CLAUDE.md`. It uses
the pinned `skills-ref` reference validator and also checks that
`.claude/skills` is still a symlink to `../.agents/skills`. Can also be run
manually via `workflow_dispatch`.

Tests: `monorepo/workflow_tests/tests/agent_skills_workflow_tests.rs`.
