# GitHub Workflow Conventions

## Keep logic out of workflow YAML

**IMPORTANT**: GitHub workflows should have as little logic as possible. They
should check out the repo, install tools with `jdx/mise-action`, and run mise
tasks. If a workflow needs real logic, put it in a script under
`standards/scripts/` exposed as a root task in `mise.toml` (like `//:audit`
and `//:affected`), not inline in YAML.

Why:

1. **Testability**: a script can be run and tested on its own; inline YAML
   logic cannot.
2. **Reusability**: the same task runs locally and in CI.
3. **Maintainability**: one place to change behaviour.

**Workflows should contain:** triggers, permissions, concurrency, checkout,
`jdx/mise-action`, caching, and `mise run <task>` steps.

**Scripts and tasks should contain:** project discovery, change detection,
standards checks, and anything with loops or conditionals.

**❌ Bad - logic in YAML:**
```yaml
- run: |
    for dir in */; do
      if [ -f "$dir/mise.toml" ] && git diff --name-only origin/main | grep -q "^$dir"; then
        echo "$dir"
      fi
    done
```

**✅ Good - call a task:**
```yaml
- run: mise run --quiet //:affected
```

## Shared CI workflow

Pull request, main and nightly CI all call the reusable
`.github/workflows/mise-projects.yml`, which audits the repo, lists projects
and runs `mise run ci` (and, on main, `mise run deploy`) in each one. Add new
per-project behaviour to the standard tasks in `standards/tasks/`, not to a
workflow. See `.github/workflows/README.md`.

## Quoting GitHub Actions expressions in Bash

When assigning a GitHub Actions expression to a bash variable, wrap it in
double quotes, or better, pass it through `env:`:

**❌ Bad:**
```yaml
- run: BASE_SHA=${{ github.event.pull_request.base.sha }}
```

**✅ Good:**
```yaml
- env:
    BASE_SHA: ${{ github.event.pull_request.base.sha }}
  run: echo "$BASE_SHA"
```

This avoids errors when the expression is empty, handles special characters,
and stops untrusted values being run as shell code.

## Authentication

- **GITHUB_TOKEN**: use for everything it can do.
- **Repository secrets** (for example `CLOUDFLARE_API_TOKEN`): pass them only
  to the step that needs them, and only on `main`, never on pull requests.
- **Custom PAT**: only for permissions GITHUB_TOKEN lacks.

## Testing workflows

Every workflow has Rust tests in `jimhillr/workflow_tests/tests/` (see its
README for the list). They check the file parses, its triggers and
permissions, and the behaviour that matters (for example that pull requests
never deploy). Run them with `mise run test` in `jimhillr/` before committing
workflow changes.

## Tools

Install tools through each project's `mise.toml` and `jdx/mise-action`, not
with separate setup actions (`actions-rust-lang/setup-rust-toolchain`,
`actions/setup-node`, ad hoc `cargo install`). Rust components such as
`rustfmt` and `clippy` are listed in the project's `rust` tool entry. See
[toolchain.md](toolchain.md).
