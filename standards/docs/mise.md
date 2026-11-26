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

`standards/scripts/affected.sh` decides what a change touches: the deepest
project holding a changed file; every project including a changed
`standards/tasks/` file; every project with a Cargo path dependency on the
changed directory (including the path dependencies of a Cargo workspace the
project sits in); and every project whose `mise.toml` has an
`# affected-by: <dir>` line naming a changed directory or an affected project.
The cloudflare projects use `# affected-by: ../web`, so a change to a web app
redeploys it.

## Decisions

| Date | Decision |
| --- | --- |
| 2026-11-26 | Replace Cast with mise for tool installs and task running; rebuild CI/CD around it. |
| 2026-11-26 | Stop committing build artifacts. |
| 2026-11-26 | Keep `auth_sdk` as a path dependency rather than a tagged git dependency. |
| 2026-11-26 | Keep per-project Cargo workspaces; no root workspace. |
| 2026-11-26 | Replace the nightly standards audit (ISSUES.md PRs) with the `//:audit` PR check. |
| 2026-11-26 | Deploy from the `push-main.yml` run with the `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` secrets, rather than from committed artifacts. |
