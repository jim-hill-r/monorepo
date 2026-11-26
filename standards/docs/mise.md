# Building with mise

Every project is built, tested and deployed with [mise](https://mise.jdx.dev).
It replaced Cast in November 2026. Shared task files live in
[`standards/tasks/`](../tasks/README.md), and the repo checks live in
`standards/scripts/`.

## Rules

- **No root Rust workspace.** Every project directory stands on its own. A
  project may be a Cargo workspace itself (cahokia, cookbook, pane, luggage),
  but the repo root never is.
- **Same commands everywhere.** Every project's `mise.toml` includes exactly
  one `standards/tasks/<type>.toml`, so `mise run ci`, `fix`, `build`, `test`,
  `dev`, `release` and `deploy` work in any project. `mise run //:audit`
  enforces this on every pull request.
- **Tools come from mise.** A project lists its own tools in `[tools]`. System
  packages that CI needs go in a `ci:deps` task override in the project's
  `mise.toml`.
- **No build artifacts in git.** Web apps deploy straight from CI. A binary
  that needs publishing goes to a GitHub Release.
- **Shared code between projects** uses relative path dependencies for now
  (`auth_sdk` in cahokia and cookbook).
- **Standards are checked on pull requests**, not by a nightly job that opens
  PRs.

## CI/CD

All three workflows call `.github/workflows/mise-projects.yml`, which runs
`mise run //:audit`, lists projects, and runs `mise run ci` in each one as its
own job.

| Workflow | Projects | Runs |
| --- | --- | --- |
| `pull-request-ci.yml` | changed since the PR base | `ci` |
| `push-main.yml` | changed by the push | `ci`, then `deploy` |
| `nightly-ci.yml` | all | `ci` |
| `nightly-cd.yml` | those with `DEPLOY_URL` | `ci`, then `deploy` |

`standards/scripts/affected.sh` decides what a change touches: the deepest
project holding a changed file; every project including a changed
`standards/tasks/` file; every project with a Cargo path dependency on the
changed directory (including the path dependencies of a Cargo workspace the
project sits in); and every project whose `mise.toml` has an
`# affected-by: <dir>` line naming a changed directory or an affected project.
The cloudflare projects use `# affected-by: ../web`, so a change to a web app
redeploys it.

### Deploys

A project that deploys sets its production URL as `DEPLOY_URL` in the
`[env]` table of its `mise.toml`. In `push-main.yml`, the job of each such
project runs in the `production` GitHub environment with that URL, so every
deploy shows on its commit and in the repo's Deployments list, and the job
summary lists the production URL, the URL of that exact deployment and the
commit. Smoke tests and PR comments read the same `DEPLOY_URL`.

A deploy runs only after `mise run ci` passed in the same job, and only for
the latest version of main: `standards/scripts/deploy-check.sh` skips it
(and says so in the job summary) when a newer commit on main changes the
project, because that commit's own run deploys it. "Changes" is decided by
`affected.sh`, so path dependencies and `# affected-by:` count. The nightly
run deploys every project this way, so each site is rebuilt and republished
from main at least once a day.

To roll back, or to redeploy one project, run **Push Main CI/CD** by hand
(Actions tab, or `workflow_dispatch` through the API) with `project` set to
the project directory, for example `cookbook/cloudflare`, and `ref` set to
the commit SHA or tag to deploy. The run checks out that ref, runs
`mise run ci` and then `mise run deploy` for that project only, without the
latest-version check. A ref from
before `DEPLOY_URL` existed still deploys, but without the environment link.

## Decisions

| Date | Decision |
| --- | --- |
| 2026-11-26 | Replace Cast with mise for tool installs and task running; rebuild CI/CD around it. |
| 2026-11-26 | Stop committing build artifacts. |
| 2026-11-26 | Keep `auth_sdk` as a path dependency rather than a tagged git dependency. |
| 2026-11-26 | Keep per-project Cargo workspaces; no root workspace. |
| 2026-11-26 | Replace the nightly standards audit (ISSUES.md PRs) with the `//:audit` PR check. |
| 2026-11-26 | Deploy from the `push-main.yml` run with the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` secrets, rather than from committed artifacts. |
| 2026-11-26 | Run CI and CD for every project every night, and deploy only the latest version of main that passed CI ([decision 0001](decisions/0001-deploy-feedback-loop.md)). |
| 2026-11-26 | Record each deploying project's production URL as `DEPLOY_URL` in its `mise.toml`, deploy in a `production` GitHub environment, and roll back by redeploying one project at a ref ([decision 0001](decisions/0001-deploy-feedback-loop.md)). |
