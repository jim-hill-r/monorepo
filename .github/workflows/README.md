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
Runs are queued, never cancelled, so a deploy is not cut short.

### nightly-ci.yml

Runs `mise run ci` in every project at 2 AM Pacific, so new toolchain or
dependency releases that break a quiet project show up.

Tests: `monorepo/workflow_tests/tests/mise_workflow_tests.rs`.

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
