---
name: code-review
description: Review one project in this monorepo against the standards in standards/docs and record the five most impactful findings as agent-generated issue files in its issues/ folder. Use for periodic code quality reviews or when asked to review a project and file issues.
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
   `issues/README.md`.
3. Prefer the project with the fewest issue files tagged `agent-generated` in
   its `issues/` folders (`mise run //:issues` lists them); a project with no
   `issues/` folder counts as zero. Break ties alphabetically.

## 2. Read the standards

Before reading code, read the relevant files in `standards/docs/`:
`code-quality.md` (the QUAL rules this review checks), `rust.md`,
`testing.md`, `documentation.md`, `naming.md`, and `typescript.md` if the
project has TypeScript. For design and naming judgement calls, read the
notes in `athenaeum/resources/code_quality/` (Rust API Guidelines, Clippy,
Effective Rust, Clean Code, Ousterhout, Fowler's smells) and name each
finding with its QUAL ID and the source's own term, such as
"QUAL-301, `C-GETTER`" or "QUAL-103, shallow module".

## 3. Inspect the code

Install the project's tooling per **Dev commands** in `AGENTS.md` if you need
to run anything. Review in Google's order, design first (QUAL-506), and
look at:

- **Design** (QUAL-101 to QUAL-106): one purpose per module, a pure core
  with I/O at the edges, deep modules with small interfaces, types that
  make bad states unrepresentable, no needless `pub`.
- **Size and complexity** (QUAL-202 to QUAL-206): functions over 60 lines,
  cognitive complexity over 15, more than 5 arguments, nesting deeper than
  4, files over 800 lines. Generated code is excluded (QUAL-407).
- **Error handling** (QUAL-207, QUAL-208): `.unwrap()`/`.expect()` where
  errors should propagate (LIN-001), panics in library paths, errors
  swallowed silently.
- **Clarity** (QUAL-201, QUAL-209 to QUAL-211): unclear names, comments
  that say what instead of why, commented-out or dead code, `dbg!`,
  duplication past the rule of three, magic numbers.
- **API** (QUAL-301 to QUAL-305), for crates other crates depend on: Rust
  API Guidelines naming, common traits, error types, builders, and
  `# Errors`/`# Panics` docs.
- **Lints** (QUAL-401, QUAL-402, QUAL-504): `Cargo.toml` sets LIN-001 and
  LIN-002; waivers are `#[expect(..., reason = "...")]`.
- **Tests**: unit tests for non-trivial functions, edge cases, E2E where
  required ([testing.md](../../../standards/docs/testing.md)).
- **Performance**: needless clones, repeated allocations, blocking in async code.
- **Docs**: public APIs and tricky logic documented; README meets the standard.

## 4. Record exactly five findings

Pick the five most impactful. Each one cites its rule ID (a QUAL ID, or
LIN, DOC or another standard's ID), names a file and function or type where
possible and says exactly what to change. Write each as its own issue file in
the project's `issues/` folder (format in
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
