---
name: code-review
description: Review one project in this monorepo against the standards in standards/docs and record the five most impactful findings as TODO (agent-generated) items in its ISSUES.md. Use for periodic code quality reviews or when asked to review a project and file issues.
metadata:
  replaces: agent_copilot/prompts/code-review.md
---

# Code review

Review one project and turn the findings into issues that the
`start-a-new-task` skill can pick up later. Read `AGENTS.md` first.

## 1. Pick a project

1. Candidates are top-level directories containing a `Cargo.toml` or
   `package.json` (not meta folders such as `.github` or `standards`).
2. Drop any project listed under `# On Hold Projects` in the root `ISSUES.md`
   or its own `ISSUES.md`.
3. Prefer the project with the fewest `TODO (agent-generated):` items in its
   `ISSUES.md`; a project with no `ISSUES.md` counts as zero. Break ties
   alphabetically.

## 2. Read the standards

Before reading code, read the relevant files in `standards/docs/`: `rust.md`,
`testing.md`, `documentation.md`, `naming.md`, and `typescript.md` if the
project has TypeScript.

## 3. Inspect the code

Install the project's tooling per **Dev commands** in `AGENTS.md` if you need
to run anything. Look at:

- **Error handling**: `.unwrap()`/`.expect()` where errors should propagate.
- **Lints**: `Cargo.toml` sets `unwrap_used = "warn"`, `expect_used = "warn"`
  and `unsafe_code = "forbid"`.
- **Tests**: unit tests for non-trivial functions, edge cases, E2E where required.
- **Clarity**: long functions, duplication, magic numbers, unclear names.
- **Performance**: needless clones, repeated allocations, blocking in async code.
- **Docs**: public APIs and tricky logic documented; README meets the standard.

## 4. Record exactly five findings

Pick the five most impactful. Each one names a file and function or type where
possible, says exactly what to change, and starts with `TODO (agent-generated):`.
Add them under `# Priority Issues` in the project's `ISSUES.md`. If the file
does not exist, create it:

```markdown
# Priority Issues

- TODO (agent-generated): <finding 1>
- TODO (agent-generated): <finding 2>
- TODO (agent-generated): <finding 3>
- TODO (agent-generated): <finding 4>
- TODO (agent-generated): <finding 5>

# Backlog
```

Re-read the file to confirm all five are present and well formed, then deliver
with the `open-pull-request` skill.
