---
name: code-review
description: Review one project in this monorepo against the standards in standards/docs and record the five most impactful findings as agent-generated issue files in its docs/issues/ folder. Use for periodic code quality reviews or when asked to review a project and file issues.
metadata:
  replaces: agent_copilot/prompts/code-review.md
---

# Code review

Review one project and turn the findings into issues that the
`start-a-new-task` skill can pick up later. Read `AGENTS.md` first.

## 1. Pick a project

1. Candidates are top-level directories containing a `Cargo.toml` or
   `package.json` (not meta folders such as `.github` or `standards`).
2. Drop any project listed under `# On Hold Projects` in the root
   `standards/docs/issues/README.md`.
3. Prefer the project with the fewest issue files tagged `agent-generated` in
   its `docs/issues/` folders (`mise run //:issues` lists them); a project with no
   `docs/issues/` folder counts as zero. Break ties alphabetically.

## 2. Read the standards

Before reading code, read the relevant files in `standards/docs/`: `rust.md`,
`testing.md`, `documentation.md`, `naming.md`, and `typescript.md` if the
project has TypeScript. For design and naming judgement calls, read the
notes in `athenaeum/resources/code_quality/` (Rust API Guidelines, Clippy,
Effective Rust, Clean Code, Ousterhout, Fowler's smells) and name each
finding with the source's own term, such as `C-GETTER` or "feature envy".

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
possible and says exactly what to change. Write each as its own issue file in
the project's `docs/issues/` folder (format in
`standards/docs/issue-management.md`), named by a short slug of the finding:

```markdown
---
kind: todo
priority: priority
tags: [agent-generated]
added: <today, YYYY-MM-DD>
---

<finding>
```

Run `mise run //:issues -- --check` to confirm all five are well formed, then
deliver with the `open-pull-request` skill.
