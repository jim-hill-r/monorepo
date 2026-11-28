# Plan: a standing code quality process

Started 2026-11-27 from Jim's request:

> We also need a process for ensuring maintainable code base. It needs to
> be well designed, with high code quality. Not sure exactly what resources
> to base this on but things like clean code or rust best practices would
> be a good place to start.

Status: **approved** by Jim (decision cards, 2026-11-27), with the
recommended answer to both questions (see [Decisions](#decisions)). The
decision is [0015](../decisions/0015-code-quality-process.md). The work items are issue files in the root
[issues/](../issues/) with `epic: Code quality`.

This plan is the code twin of the [eval engine](2026-11-eval-engine.md):
evals keep *what we ship* green, this keeps *how it is built* green, with
the same loop (written standard, a deterministic check, a ratchet, a free
workflow that files issues, the normal agent loop that works them).

## Goal

The codebase stays easy to change. Concretely:

1. **One written standard** says what good code looks like here, drawn from
   well-known sources and cut to rules a tool or a reviewer can check.
2. **Machines enforce everything they can.** Lints, formatting and
   complexity limits run in the git hooks and PR CI, in every language the
   repo uses, so review time goes to design.
3. **Quality only goes up.** Each project has a committed baseline of its
   quality findings. A PR may lower it, never raise it (a ratchet, like
   code coverage).
4. **Gaps become issues on their own.** A weekly free workflow files issue
   files for the worst findings, and a design review rubric turns judgment
   findings into issues too. Nothing waits for someone to notice.
5. **It is visible.** Each project gets a quality badge next to its eval
   coverage badge.

Not in scope: rewriting working code for style alone, or a hosted
"code quality" SaaS (SonarCloud, Codacy, CodeClimate). Every tool below is
open source and runs in GitHub Actions or the hooks, so the money cost is
$0.

## Sources

Each source is used for what it is good at. Where they disagree, the
standard picks one and says why.

| Source | What we take | What we leave |
| --- | --- | --- |
| [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html) | The checklist, by its IDs: naming (C-CASE, C-CONV `as_`/`to_`/`into_`, C-GETTER no `get_`), common traits (C-COMMON-TRAITS, C-DEBUG), errors (C-GOOD-ERR), newtypes and custom types over `bool`s (C-NEWTYPE, C-CUSTOM-TYPE), builders (C-BUILDER), docs with examples and `# Errors`/`# Panics` (C-EXAMPLE, C-FAILURE) | Crates.io publishing items (C-METADATA, C-STABLE) for crates we don't publish |
| [Clippy](https://doc.rust-lang.org/clippy/) lint groups | `pedantic` as the target level; the thresholds `too_many_lines`, `cognitive_complexity`, `too_many_arguments`, `excessive_nesting`; a few `restriction` lints (`dbg_macro`, `todo`, `allow_attributes_without_reason`, `undocumented_unsafe_blocks`) | `nursery` as a group (unstable), `restriction` as a group (contradictory by design), `cargo` (we don't publish) |
| [Effective Rust](https://effective-rust.com/) (Drysdale) and the [Rust Design Patterns](https://rust-unofficial.github.io/patterns/) book | Use the type system to make bad states unrepresentable, prefer `Option`/`Result` transforms, newtype, RAII, avoid `clone` to satisfy the borrow checker, the anti-patterns list | Patterns for problems we don't have |
| *Clean Code* (Martin) | Intention-revealing names, functions that do one thing at one level of abstraction, comments that say *why*, no dead code, the boy scout rule | The "functions of 2 to 4 lines" and "no comments" extremes, and its class-heavy OO advice, which don't fit Rust |
| *A Philosophy of Software Design* (Ousterhout) | Deep modules with small interfaces, information hiding, "define errors out of existence", design it twice for anything public | (Used as the counterweight to Clean Code's tiny functions) |
| *Refactoring* (Fowler) | The code smell list as the review vocabulary (long function, duplicated code, feature envy, primitive obsession, shotgun surgery), and the rule of three for duplication | |
| [Google's code review guide](https://google.github.io/eng-practices/review/) | What a reviewer looks at first (design, then function, complexity, tests, naming, comments), and "approve once it improves the code health of the system, even if it isn't perfect" | |
| TypeScript: [Biome](https://biomejs.dev/) recommended rules and `tsc --strict` | Lint and format for our 42 `.ts` files (Playwright tests), and CSS | typescript-eslint's type-aware rule sets: deeper, but a Node toolchain and config per project for 42 test files |
| Shell: [ShellCheck](https://www.shellcheck.net/) and [shfmt](https://github.com/mvdan/sh) | For the shell scripts that remain (see Question 2) | |
| Workflows: [actionlint](https://github.com/rhysd/actionlint) | Type and expression errors in `.github/workflows/` (zizmor already covers their security, SEC-005) | |
| TOML: [taplo](https://taplo.tamasfe.dev/) | Format check for our 233 TOML files (a Rust tool) | |

Our own existing rules stay: [rust.md](../rust.md) (LIN-001, LIN-002),
[testing.md](../testing.md), [documentation.md](../documentation.md),
[naming.md](../naming.md), anyhow for apps and thiserror for libraries.

## Where we are (measured 2026-11-27)

- Every Rust package already has LIN-001 and LIN-002, and CI runs
  `cargo fmt --check` and `cargo clippy -- -D warnings` (default lints only,
  and not on test code: no `--all-targets`).
- No complexity limits, no `pedantic`, no TypeScript, shell, workflow or
  TOML linting. One `clippy.toml` (cahokia, for Dioxus await rules).
- Running `cargo clippy --all-targets -- -W clippy::pedantic` today gives
  about 140 warnings in `cost`, 120 in `evals`, 100 in `threat` and 5 in
  `telemetry` (lib and test targets counted separately). The top lints are
  `must_use_candidate`, `format_push_string`, `expect_used` in tests,
  `missing_errors_doc` and `too_many_lines`. So a big-bang switch would
  break CI in nearly every project; a ratchet doesn't.
- Long files: `cookbook/web/src/main.rs` (2,054 lines),
  `controlled_chaos/src/game.rs` (1,828), `cookbook/data_md/src/lib.rs`
  (1,826), `cahokia/core/src/lib.rs` (1,770). Generated code
  (`infra/src/tfschema/`) is excluded from every check.
- [README.md](../README.md) says "no shell scripts", but the repo has 53
  `.sh` files, most in `standards/scripts/`. The standard and the repo
  disagree (Question 2).
- The `code-review` skill already files five findings per project, but its
  checklist is six bullets and nothing schedules it.

## Design

### 1. The written standard

A new `standards/docs/code-quality.md`, with stable IDs like the other
standards so the audit, the review rubric and issue files can cite them:

- **QUAL-1xx design**: module boundaries (a crate or module has one
  purpose; I/O at the edges and a pure core, the shape our `core`/`model`
  crates already have), deep modules, make bad states unrepresentable,
  no `pub` that nothing outside uses.
- **QUAL-2xx code**: names, function size and nesting (with the numbers
  below), errors, no panics in library paths, comments say why, no
  commented-out code, duplication (rule of three).
- **QUAL-3xx API**: the Rust API Guidelines items above, for any crate
  other crates in the repo depend on.
- **QUAL-4xx per language**: Rust lints, TypeScript, shell, workflows, TOML.
- **QUAL-5xx process**: the ratchet, the badge, the review rubric, when a
  waiver (`#[expect(clippy::..., reason = "...")]`) is fine.

Each rule says whether a tool enforces it (and which) or a reviewer does.

### 2. Rust enforcement: a strict profile and a ratchet

Two lint levels:

- **Hard** (in each `Cargo.toml`, fails CI through `-D warnings`): today's
  LIN-001 and LIN-002, plus `dbg_macro`, `todo` and
  `allow_attributes_without_reason`. CI's clippy gains `--all-targets`,
  with `allow-unwrap-in-tests` and `allow-expect-in-tests`, so test code is
  linted too.
- **Strict** (ratcheted): `clippy::pedantic`, `clippy::cognitive_complexity`
  and the thresholds, from one shared `standards/quality/clippy.toml` that
  the ratchet run points at with `CLIPPY_CONF_DIR`:

  | Threshold | Clippy default | Ours |
  | --- | --- | --- |
  | `too-many-lines-threshold` (function body) | 100 | 60 |
  | `cognitive-complexity-threshold` | 25 | 15 |
  | `too-many-arguments-threshold` | 7 | 5 |
  | `excessive-nesting-threshold` | off | 4 |
  | File length (our check, not clippy) | none | 800 lines |

  Allowed from `pedantic`, with reasons in the standard:
  `must_use_candidate` (noise on every getter) and
  `module_name_repetitions` (fights our crate naming).

**The ratchet** is a new `chores quality` command (a deterministic Rust
command, per the baked-skills plan, so no tokens are spent on it). For one
project it runs clippy with the strict profile and `--message-format=json`,
adds the file-length check, and compares the counts per lint with the
project's committed `docs/quality.toml`:

```toml
# Written by `chores quality --update`. Lower is better; never raise it by hand.
[baseline]
updated = 2026-11-27
total = 118

[baseline.lints]
"clippy::format_push_string" = 30
"clippy::too_many_lines" = 8
"jimhillr::file_too_long" = 1
```

- `chores quality --check` (in the hooks and PR CI, affected projects only)
  fails when any count goes **up**, naming the new findings with file and
  line, and passes when counts fall, asking for `--update` so the baseline
  follows (QUAL-501).
- **Graduation**: when a project's total reaches 0, the next PR moves
  `pedantic` and the thresholds into its hard lints and deletes its
  baseline; from then on plain clippy keeps it clean (QUAL-502). The audit
  checks every project has one or the other.
- cahokia's own `clippy.toml` keeps its Dioxus rules and copies the shared
  thresholds; the audit checks they match.

### 3. Other languages

All in `mise` tools, pinned, run by the shared tasks so every project gets
them through `mise run ci` and the hooks:

- **TypeScript and CSS**: `biome ci` (lint and format) plus `tsc --noEmit`
  under the existing strict `tsconfig`. Biome is one binary with no Node
  plugins, so it adds no `package.json` dependencies.
- **Shell**: `shellcheck` and `shfmt -d` on every tracked `.sh` file, from
  the audit (see Question 2 for what happens to the scripts themselves).
- **Workflows**: `actionlint`, next to zizmor in `workflows-check.sh`.
- **TOML**: `taplo fmt --check`.

New ones start as warnings for one week (the way SEC-005 did), then fail.
Each goes on the [tech radar](../tech-radar.md) in the PR that adopts it.

### 4. The feedback loop

The same shape as [eval triage](../evals.md#triage), so it costs nothing on
a quiet week:

- **Quality triage** (`quality-triage.yml`, weekly, free Actions minutes):
  runs `chores quality --report` over every project, and for each project
  with findings and no open `quality-*` issue file, writes one issue file
  for its worst file or lint (`todo`, backlog, or priority for a file over
  1,500 lines or a function over cognitive complexity 30). It writes at most
  10 a run, force-pushes them to the `quality-triage` branch, and PR triage
  opens the draft PR, exactly as eval triage does.
- **Agents fix while they work** (the boy scout rule, QUAL-503): an agent
  changing a file leaves no new findings in it, and the ratchet holds it to
  that.
- **Design review** (judgment the tools can't make): the `code-review`
  skill gets a real rubric (the QUAL-1xx and QUAL-3xx rules, the Fowler
  smells, Google's review order) and files its findings citing rule IDs.
  The weekly `deep-project-review` already rotates through projects; it
  runs the `code-review` rubric as one section, so there is no new
  scheduled task.
- **Badge**: each project README gets a quality badge (findings left, green
  at 0 once graduated) next to eval coverage, and the README's repo health
  section gets one for the whole repo.

### 5. In the PR process

`open-pull-request` gains one checklist line: "the quality ratchet passes
and no new waivers lack a reason". The PR CI **Quality** job posts the
counts that changed, so a reviewer sees "cookbook: 118 → 109".

## Phases

| Phase | What | PRs (about) |
| --- | --- | --- |
| 1. Standard and ratchet | `code-quality.md`; `standards/quality/clippy.toml`; `chores quality` with `--check`, `--update`, `--report`; baselines for every project; `--all-targets` and the new hard lints in `rust.toml`; hooks and PR CI | 3 |
| 2. Other languages | Biome, ShellCheck and shfmt, actionlint, taplo, each with its radar entry; warnings for a week, then failing | 2 to 3 |
| 3. Loop | `quality-triage.yml`; badges; the `code-review` rubric and its use in `deep-project-review` | 2 |
| 4. Burn-down | Issue files from triage, worked by the normal agent loop; each project graduates at 0 | ongoing |

## Cost

- **Money: $0.** Every tool is open source. All jobs run on standard
  GitHub-hosted runners, unlimited on a public repo
  ([free-tiers.md](../free-tiers.md)). The ratchet's second clippy run
  reuses the build cache, adding roughly a minute per changed Rust project
  to PR CI. Triage stores nothing new (no artifacts, one branch).
- **Free-tier headroom:** no change to Actions cache use beyond the
  existing per-project clippy caches.
- **Claude Code effort:** phases 1 to 3 are about 7 to 8 PRs of thread
  work, similar in size to the threat-check phases. Burn-down is one small
  issue per project per week through the normal loop, which is work the
  loop would otherwise spend on other backlog, so it adds no new spend
  against the $25 a month budget; it changes what the budget is spent on.

## Questions for Jim

1. **How strict, and how fast?** Recommended: pedantic and complexity
   limits through the per-project ratchet above, graduating each project at
   0. Lighter: only the complexity limits, no pedantic (about half the
   burn-down, misses many idiom fixes). Faster: turn pedantic on as hard
   lints everywhere now and fix it in one sweep (around 1,000+ warnings at
   once across ~58 crates, many parallel PRs that conflict). All three cost
   $0 in money.
2. **What happens to shell scripts?** The standard says "no shell
   scripts", but there are 53. Recommended: keep the Rust-only rule, lint
   the existing scripts with ShellCheck now, block new `.sh` files in the
   audit, and port the scripts into `chores` commands over time as backlog
   issues (fits the baked-skills direction; about 1 to 2 scripts per PR).
   Alternative: allow shell for short glue (under ~100 lines) with
   ShellCheck and shfmt, and change the standard to say so (less porting
   work, two languages for scripts forever).

## Decisions

Jim, 2026-11-27, on decision cards:

1. **Strictness:** "Pedantic via per-project ratchet". Pedantic and the
   complexity limits go in through each project's baseline, and each
   project graduates to hard lints at 0.
2. **Shell scripts:** "Keep Rust-only, port over time". The "no shell
   scripts" rule stays. Phase 2 lints the existing scripts with ShellCheck
   and shfmt, the audit fails any new tracked `.sh` file (QUAL-404), and
   the scripts are ported into `chores` commands as backlog issues, one or
   two per PR, starting with the ones the hooks and CI run most.
