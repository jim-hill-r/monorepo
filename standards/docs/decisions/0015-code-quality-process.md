# 0015. A standing code quality process: one standard, a ratchet, triage into issues

- Status: Proposed
- Date: 2026-11-27
- Asked by: Jim
- Plan: [2026-11-code-quality.md](../plans/2026-11-code-quality.md)
- Builds on: [0003](0003-eval-engine.md) (the eval loop this copies) and
  [0006](0006-one-file-per-issue.md) (issue files)

## Context

Jim (2026-11-27): "We also need a process for ensuring maintainable code
base. It needs to be well designed, with high code quality." Today the repo
enforces two clippy rules (LIN-001, LIN-002), `rustfmt` and default clippy
lints on non-test code. Nothing limits complexity, nothing lints TypeScript,
shell, workflows or TOML, and design review happens only when someone runs
the `code-review` skill. Clippy `pedantic` gives roughly 100 warnings in a
typical project today, so turning it on everywhere at once would break CI
across the repo.

## Decision

1. **One written standard**, `standards/docs/code-quality.md`, with QUAL
   rule IDs, drawn from the Rust API Guidelines, Clippy, Effective Rust,
   Clean Code, A Philosophy of Software Design, Refactoring and Google's
   code review guide (what we take from each, and what we leave, is in the
   plan). Each rule names its enforcer: a tool or the review rubric.
2. **Machines enforce what they can**, in the hooks and PR CI: clippy
   (with `--all-targets`), a strict profile (`pedantic`, cognitive
   complexity 15, function body 60 lines, 5 arguments, nesting 4, files 800
   lines), Biome for TypeScript and CSS, ShellCheck and shfmt, actionlint
   and taplo. All open source, $0.
3. **A ratchet, not a big bang.** `chores quality` compares each project's
   strict findings with its committed `docs/quality.toml` baseline. A PR may
   lower the counts, never raise them. At 0 the project graduates: the
   strict lints become hard lints and the baseline goes away.
4. **Triage files the work**, like eval triage: a weekly free workflow
   writes issue files for the worst findings to a branch that PR triage
   opens as a draft PR. Design findings come from the `code-review` rubric,
   run inside the weekly deep project review. The normal agent loop works
   both.
5. **Visible**: a quality badge per project and one for the repo.

## Options considered

| Option | Money | Why not (or why) |
| --- | --- | --- |
| Ratchet per project (chosen) | $0 | Improves every week without breaking CI; costs a second cached clippy run per changed project |
| Pedantic as hard lints everywhere now | $0 | Likely over 1,000 warnings at once; dozens of parallel PRs that conflict |
| Complexity limits only | $0 | About half the work, but misses most idiom and API fixes |
| Hosted analysis (SonarCloud, Codacy) | $0 on public repos | A new vendor (needs sign-off), weak Rust support, results live outside the repo |

## Consequences

- Every project gets a `docs/quality.toml` in phase 1, and the audit checks
  each project has either a baseline or the hard strict lints.
- Agents leave no new findings in files they touch (the ratchet enforces
  it), so quality rises with normal work.
- New tools go on the tech radar in the PR that adopts each.
- Open questions (strictness, shell scripts) are in the plan; this record
  is updated with Jim's answers before it is accepted.
