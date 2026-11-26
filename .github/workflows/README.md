# GitHub Actions Workflows

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
2. **ci (&lt;project&gt;)** runs once per project: installs the project's tools
   with `jdx/mise-action`, restores the Rust cache, runs `mise run ci`, and
   then `mise run deploy` when the `deploy` input is true.

### pull-request-ci.yml

Runs on every pull request against the projects it changes. Never deploys.
`workflow_dispatch` runs every project on a branch.

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


## pull-request-review-decks.yml (Review decks)

Runs on pull requests that touch `docs/reviews/` (and on `workflow_dispatch`).
Builds every review deck to HTML and PDF with the Marp CLI through
`standards/scripts/review-decks.sh` (also `mise run //:review-decks`) and
uploads them as the `review-decks` artifact. See
[docs/reviews/README.md](../../docs/reviews/README.md).

## all-free-tier-usage.yml

Weekly (Mondays, 7 AM Pacific), by hand, or called from another workflow:
runs `standards/scripts/free-tiers.sh usage`, which reads how many Cloudflare
Pages projects and Workers exist and how much Actions cache and artifact
storage the repo uses, writes a table to the run summary, and fails when
anything reaches its budget in
[standards/docs/free-tiers.md](../../standards/docs/free-tiers.md). It only
reads.

Tests: `monorepo/workflow_tests/tests/free_tier_tests.rs`.

## codeql.yml

This workflow runs CodeQL security scanning on the codebase, analyzing JavaScript/TypeScript and GitHub Actions workflows but **not Rust**.

### Why Rust is Excluded

- **Performance**: Rust CodeQL analysis takes ~26 minutes per scan
- **Scope**: The monorepo contains ~20+ Rust projects
- **Efficiency**: Most PRs don't modify all Rust code
- **Alternative**: Rust security is checked by `clippy` in `mise run ci`

### How It Works

1. **Trigger**: The workflow runs on:
   - Push to `main` branch
   - Pull requests to `main` branch
   - Weekly schedule (Sundays at 2 AM UTC)

2. **Language Scanning**: 
   - **JavaScript/TypeScript**: ~1 minute scan time
   - **GitHub Actions**: ~40 seconds scan time
   - **Rust**: Excluded (would take ~26 minutes)

3. **Analysis**:
   - Uses GitHub's CodeQL action
   - Automatically builds the code (autobuild)
   - Uploads results to GitHub Security tab

### Relationship to GitHub's Default CodeQL

This custom workflow **replaces** GitHub's default CodeQL setup. When a custom `.github/workflows/codeql.yml` exists:
- GitHub's automatic default CodeQL is disabled
- This custom workflow takes over
- You have full control over which languages are scanned

### Re-enabling Rust Scanning

If Rust security scanning is needed:

**Option 1: Enable in this workflow**
- Edit `.github/workflows/codeql.yml`
- Add `'rust'` to the `matrix.language` array
- Note: This will add ~26 minutes to each PR/push

**Option 2: Create a separate scheduled workflow**
- Create a new workflow that only runs weekly
- Only scans Rust
- Doesn't block PRs with long scan times

### Permissions

The workflow requires:
- `contents: read` - To checkout the repository
- `security-events: write` - To upload CodeQL results
- `actions: read` - To read workflow information


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
