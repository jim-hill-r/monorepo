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
  `dev`, `release`, `deploy` and `evals` work in any project. `mise run //:audit`
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
- **Checks run before code leaves the machine.** lefthook git hooks
  (`mise run //:hooks`) run `//:audit` and `mise run ci` in every affected
  project on each commit and push, so PR CI is a double check. See
  [git-hooks.md](git-hooks.md).

## CI/CD

All three workflows call `.github/workflows/mise-projects.yml`, which runs
`mise run //:audit`, lists projects, and runs `mise run ci` in each one as its
own job, then `mise run evals`, which turns the test reports `ci` saved into
eval results in the job summary and a 14-day `evals-results-<project>`
artifact ([evals.md](evals.md#in-ci)).

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

A change to the root `mise.toml`, `standards/scripts/` or the mise CI
workflows makes `push-main.yml` check every project. Pull requests (and the
git hooks) pass `--changed-only` instead, so such a change checks only
`standards`, `monorepo` (whose workflow tests read those files) and, for a
script, the projects whose `mise.toml` or included task file names it
(Jim, 2026-11-26: CI runs only for the projects a change touches). The
nightly CI run still checks every project.

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

### Game builds

Bevy games (`standards/tasks/bevy-game.toml`) have three extra tasks that
build the game for players, through `standards/scripts/bevy-release.sh`:
`release:web` (WebAssembly in `dist/web`), `release:macos` (a universal
`.app`, zipped, macOS only) and `release:android` (an APK, with cargo-apk).

- **Web.** A game with a `wrangler.toml` deploys its web build to Cloudflare
  Pages like any other site: it sets `DEPLOY_URL`, and `mise run deploy`
  builds `dist/web` and runs `wrangler pages deploy`, so `push-main.yml` and
  Nightly CD publish it. A game without one deploys nothing.
- **macOS and Android.** A game with `RELEASE_TAG` in its `[env]` gets
  both built every night by `nightly-downloads.yml` (after `mise run ci`
  passes) and published to that GitHub Release, a prerelease whose tag is
  moved to the commit built and whose assets are replaced in place. The
  links never change:
  `https://github.com/jimhillr/monorepo/releases/download/<RELEASE_TAG>/<package>-macos.zip`
  and `.../<package>-android.apk`.
- **Finding them.** `mise run //:downloads` (`standards/scripts/downloads.sh
  list`) prints every game with downloads as JSON: project, release tag and
  web URL. Tools such as pane's Deliverables page read that list, or the
  `RELEASE_TAG` and `DEPLOY_URL` lines in each `mise.toml`, instead of
  hard-coding links.
- **Signing.** The macOS app has only an ad-hoc signature, because Developer
  ID signing and notarization need a paid Apple account. The APK is signed
  with the `ANDROID_KEYSTORE_BASE64` and `ANDROID_KEYSTORE_PASSWORD` secrets
  when they exist, otherwise with a throwaway debug key.

## Decisions

| Date | Decision |
| --- | --- |
| 2026-11-26 | Replace Cast with mise for tool installs and task running; rebuild CI/CD around it. |
| 2026-11-26 | Stop committing build artifacts. |
| 2026-11-26 | Keep `auth_sdk` as a path dependency rather than a tagged git dependency. |
| 2026-11-26 | Keep per-project Cargo workspaces; no root workspace. |
| 2026-11-26 | Replace the nightly standards audit (ISSUES.md PRs) with the `//:audit` PR check. |
| 2026-11-26 | Run the CI checks locally in lefthook pre-commit and pre-push hooks, installed through mise ([git-hooks.md](git-hooks.md)). |
| 2026-11-26 | Deploy from the `push-main.yml` run with the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` secrets, rather than from committed artifacts. |
| 2026-11-26 | Run CI and CD for every project every night, and deploy only the latest version of main that passed CI ([decision 0001](decisions/0001-deploy-feedback-loop.md)). |
| 2026-11-26 | Publish game builds for playtesting: the web build to Cloudflare Pages, and macOS and Android builds to a nightly GitHub prerelease per game (`RELEASE_TAG`). |
| 2026-11-26 | Record each deploying project's production URL as `DEPLOY_URL` in its `mise.toml`, deploy in a `production` GitHub environment, and roll back by redeploying one project at a ref ([decision 0001](decisions/0001-deploy-feedback-loop.md)). |
| 2026-11-26 | Pull request CI checks only the projects a change touches; shared scripts and root config no longer run every project (`affected.sh --changed-only`). |
