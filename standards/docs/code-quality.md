# Code quality

What good code looks like in this repo, as rules a tool or a reviewer can
check. The rules come from the sources in
[athenaeum/resources/code_quality/](../../athenaeum/resources/code_quality/README.md)
(what we take from each, and what we leave, is in the
[plan](plans/2026-11-code-quality.md#sources)), and the process that keeps
them green is [decision 0015](decisions/0015-code-quality-process.md).

Every rule has a stable ID so the audit, the review rubric and issue files
can cite it, and names its **enforcer**: a tool that fails CI, the
**ratchet** (`chores quality`, which may only see a project's findings go
down, QUAL-501), or **review** (the `code-review` rubric and the PR
reviewer). Where a tool lands in a later phase of the plan, the rule says
so; until then review enforces it.

Older rules stay where they are and still apply: [rust.md](rust.md)
(LIN-001, LIN-002), [testing.md](testing.md),
[documentation.md](documentation.md) and [naming.md](naming.md).

## Summary

| ID | Rule | Enforcer |
| --- | --- | --- |
| QUAL-101 | One purpose per crate and module | Review |
| QUAL-102 | Pure core, I/O at the edges | Review |
| QUAL-103 | Deep modules with small interfaces | Review |
| QUAL-104 | Make bad states unrepresentable | Review |
| QUAL-105 | No `pub` that nothing outside uses | Review |
| QUAL-106 | Design public interfaces twice | Review |
| QUAL-201 | Names reveal intent | Review |
| QUAL-202 | Function bodies at most 60 lines | Ratchet (`too_many_lines`) |
| QUAL-203 | Cognitive complexity at most 15 | Ratchet (`cognitive_complexity`) |
| QUAL-204 | At most 5 arguments | Ratchet (`too_many_arguments`) |
| QUAL-205 | Nesting at most 4 deep | Ratchet (`excessive_nesting`) |
| QUAL-206 | Files at most 800 lines | Ratchet (`jimhillr::file_too_long`) |
| QUAL-207 | Errors propagate with types that say what went wrong | Clippy (LIN-001), review |
| QUAL-208 | No panics in library paths | Clippy (`todo`, LIN-001), ratchet (`missing_panics_doc`), review |
| QUAL-209 | Comments say why; no commented-out code | Review |
| QUAL-210 | No dead or debug code | rustc `dead_code`, clippy `dbg_macro` |
| QUAL-211 | Remove duplication on the third copy | Review |
| QUAL-301 | API names follow the Rust API Guidelines | Review, ratchet (pedantic) |
| QUAL-302 | Public types implement the common traits | Review |
| QUAL-303 | Error types are meaningful and well behaved | Review |
| QUAL-304 | Builders for many optional parts | Review |
| QUAL-305 | Public items document examples, errors and panics | Ratchet (`missing_errors_doc`, `missing_panics_doc`), review |
| QUAL-401 | Rust: rustfmt and clippy hard lints | `mise run ci` |
| QUAL-402 | Rust: the strict profile | Ratchet |
| QUAL-403 | TypeScript and CSS: Biome and `tsc --strict` | Biome, tsc (phase 2) |
| QUAL-404 | Shell: no new scripts; existing ones linted | Audit, ShellCheck, shfmt (phase 2) |
| QUAL-405 | Workflows: actionlint | actionlint (phase 2) |
| QUAL-406 | TOML: taplo format | taplo (phase 2) |
| QUAL-407 | Generated code is excluded | Every check |
| QUAL-501 | The ratchet: findings only go down | `chores quality --check` |
| QUAL-502 | Graduation at zero | Audit |
| QUAL-503 | Leave no new findings in files you touch | Ratchet, review |
| QUAL-504 | Waivers are `#[expect]` with a reason | Clippy (`allow_attributes_without_reason`) |
| QUAL-505 | A quality badge per project | README (phase 3) |
| QUAL-506 | Review in Google's order and cite rule IDs | Review |
| QUAL-507 | New tools warn for a week before they fail | CI |

## QUAL-1xx: design

These are judgment rules. Review enforces them, with the
[code-review](../../.agents/skills/code-review/SKILL.md) rubric naming each
finding by its ID and the source's own term.

- **QUAL-101: one purpose per crate and module.** A crate or module can be
  described in one sentence without "and". When a module grows a second
  job, split it. Source: Clean Code (single responsibility), Ousterhout.
- **QUAL-102: pure core, I/O at the edges.** Logic that decides things is
  plain functions over plain data, so it is unit tested without a network,
  a clock or a file system. Reading and writing (HTTP, files, Workers
  bindings, the terminal) lives in a thin outer layer. The `core` and
  `model` crates many projects already have are this shape; keep it.
- **QUAL-103: deep modules with small interfaces.** A module's interface
  should be much simpler than what it hides. Prefer one function that does
  the whole job over several that callers must call in the right order,
  and hide details callers don't need ("information hiding"). This is the
  counterweight to tiny functions: don't split a function so far that the
  reader has to jump between many shallow ones. Source: Ousterhout.
- **QUAL-104: make bad states unrepresentable.** Use enums instead of
  flags that can't all be true at once, newtypes instead of bare `String`
  or `u64` for ids, money and units (primitive obsession), and parse input
  into a validated type once at the edge instead of checking it
  everywhere. "Define errors out of existence" where an API can. Sources:
  Effective Rust, Ousterhout, C-NEWTYPE and C-CUSTOM-TYPE.
- **QUAL-105: no `pub` that nothing outside uses.** An item is `pub` only
  when another crate or module uses it; otherwise `pub(crate)` or private.
  A smaller surface is easier to change.
- **QUAL-106: design public interfaces twice.** For a new public API (a
  crate other projects depend on, a Worker endpoint, a file format),
  sketch at least two shapes and say in the PR or plan why this one won.
  Source: Ousterhout.

## QUAL-2xx: code

- **QUAL-201: names reveal intent.** A name says what a thing is or does
  in the domain's words, without needing a comment. No abbreviations
  beyond the common ones (`id`, `url`, `db`), no `data`, `info`, `manager`
  or `helper` alone, and booleans read as questions (`is_ready`). Naming
  conventions are in [naming.md](naming.md). Source: Clean Code.
- **QUAL-202: function bodies at most 60 lines.** Clippy's
  `too_many_lines` with `too-many-lines-threshold = 60` (clippy's default
  is 100). A long function is usually several steps that each deserve a
  name.
- **QUAL-203: cognitive complexity at most 15.** Clippy's
  `cognitive_complexity` with `cognitive-complexity-threshold = 15`
  (default 25).
- **QUAL-204: at most 5 arguments.** Clippy's `too_many_arguments` with
  `too-many-arguments-threshold = 5` (default 7). Group related arguments
  into a struct (QUAL-304 for optional ones).
- **QUAL-205: nesting at most 4 deep.** Clippy's `excessive_nesting` with
  `excessive-nesting-threshold = 4` (off by default). Return early, use
  `?`, `let ... else` and iterator chains.
- **QUAL-206: files at most 800 lines.** Our own check in `chores quality`,
  reported as `jimhillr::file_too_long`. Test modules count; generated code
  doesn't (QUAL-407). A long file is usually several modules.
- **QUAL-207: errors propagate with types that say what went wrong.** Use
  `?` and return `Result`; applications use `anyhow` with `.context(...)`
  saying what was being done, libraries use `thiserror` enums callers can
  match on. No `.unwrap()` or `.expect()` outside tests (LIN-001, a hard
  lint today). Never swallow an error silently: handle it, return it or
  log it with `tracing` and say why it is safe to continue.
- **QUAL-208: no panics in library paths.** Library code returns errors
  instead of panicking: no `todo!` (clippy `todo`, a hard lint from phase
  1), `unimplemented!`, `panic!` on bad input, or unchecked indexing on
  data from outside. A panic that can only mean a bug is fine, documented
  under `# Panics` (clippy `missing_panics_doc`, in the strict profile).
- **QUAL-209: comments say why; no commented-out code.** The code says
  what; a comment explains a reason the code can't show (a constraint, a
  workaround and its link, a choice between alternatives). Delete
  commented-out code: git keeps it. `TODO`/`FIX` comments follow
  [issue-management.md](issue-management.md). Source: Clean Code.
- **QUAL-210: no dead or debug code.** rustc's `dead_code` warning fails
  CI through `-D warnings`; don't silence it with an `#[allow]` (QUAL-504).
  No `dbg!` (clippy `dbg_macro`, a hard lint from phase 1) and no stray
  `println!` in libraries; use `tracing`.
- **QUAL-211: remove duplication on the third copy.** Two similar pieces
  of code may stay; the third time, extract the shared part (Fowler's rule
  of three). Duplicated knowledge (a limit, a URL, a rule) is extracted on
  the second copy, since the copies drift apart.

## QUAL-3xx: API

For any crate that other crates in the repo depend on (the technical
platform crates, a project's `core` or `model`). Items are named by their
[Rust API Guidelines](../../athenaeum/resources/code_quality/rust-api-guidelines.md)
IDs. Crates.io publishing items (C-METADATA, C-STABLE) don't apply to
crates we don't publish.

- **QUAL-301: names follow the guidelines.** C-CASE (casing), C-CONV
  (`as_` cheap borrow, `to_` costly conversion, `into_` consuming), C-GETTER
  (getters have no `get_` prefix), C-ITER (`iter`, `iter_mut`,
  `into_iter`). Several pedantic lints catch parts of this.
- **QUAL-302: public types implement the common traits.** C-COMMON-TRAITS
  (`Clone`, `PartialEq`, `Eq`, `Hash`, `Default` where they make sense),
  C-DEBUG (every public type is `Debug`), C-SERDE where the type is data.
- **QUAL-303: error types are meaningful and well behaved.** C-GOOD-ERR:
  error types implement `std::error::Error`, `Send` and `Sync`, their
  `Display` is a lowercase message without trailing punctuation, and `()`
  is never an error type.
- **QUAL-304: builders for many optional parts.** C-BUILDER: a type with
  more than a few optional settings gets a builder or a settings struct
  with `Default`, not a constructor with many `Option` arguments.
- **QUAL-305: public items document examples, errors and panics.**
  C-EXAMPLE (a crate's main items have a runnable example), C-FAILURE
  (`# Errors` and `# Panics` sections, clippy `missing_errors_doc` and
  `missing_panics_doc` in the strict profile). The general rules are in
  [documentation.md](documentation.md).

## QUAL-4xx: per language

- **QUAL-401: Rust hard lints.** Every Rust project passes `cargo fmt
  --check` and `cargo clippy -- -D warnings` with LIN-001 and LIN-002,
  through `mise run ci`, today. Phase 1 adds `--all-targets` (test code is
  linted too, with `allow-unwrap-in-tests` and `allow-expect-in-tests`)
  and the hard lints `dbg_macro`, `todo` and
  `allow_attributes_without_reason` to every `Cargo.toml` and
  `standards/tasks/rust.toml`.
- **QUAL-402: the Rust strict profile.** `clippy::pedantic`,
  `clippy::cognitive_complexity` and the thresholds of QUAL-202 to
  QUAL-205, from one shared `standards/quality/clippy.toml` that the
  ratchet run uses through `CLIPPY_CONF_DIR`. Two pedantic lints are
  allowed: `must_use_candidate` (it fires on nearly every getter, noise
  that hides real findings) and `module_name_repetitions` (it fights our
  crate and type naming, such as `cost::CostReport`). A project with its
  own `clippy.toml` (cahokia, for its Dioxus rules) copies the shared
  thresholds, and the audit checks they match. Until a project graduates
  (QUAL-502), the ratchet enforces this profile, not CI's `-D warnings`.
- **QUAL-403: TypeScript and CSS.** `biome ci` (Biome's recommended lint
  rules and formatter) and `tsc --noEmit` under the strict `tsconfig`.
  [typescript.md](typescript.md) has the rest. Phase 2.
- **QUAL-404: shell.** No new tracked `.sh` file: the audit fails one
  (scripting is Rust, see [README.md](README.md#languages)). The existing
  scripts pass ShellCheck and `shfmt -d` until they are ported into
  `chores` commands. Phase 2.
- **QUAL-405: workflows.** `actionlint` on `.github/workflows/`, next to
  zizmor, which already checks their security (SEC-005). Phase 2.
- **QUAL-406: TOML.** `taplo fmt --check` on every tracked TOML file.
  Phase 2.
- **QUAL-407: generated code is excluded.** Code a tool writes, such as
  `infra/src/tfschema/`, is left out of every quality check and the file
  length limit. Fix the generator, not its output.

## QUAL-5xx: process

- **QUAL-501: the ratchet.** Each project not yet graduated commits
  `docs/quality.toml`, its strict-profile findings counted per lint,
  written by `chores quality --update`. `chores quality --check` runs in
  the git hooks and the PR CI **Quality** job for affected projects, fails
  when any count goes up (naming the new findings by file and line), and
  asks for `--update` when counts fall so the baseline follows. Never
  raise a baseline by hand. Phase 1.
- **QUAL-502: graduation at zero.** When a project's baseline total
  reaches 0, the next PR moves the strict profile into its hard lints and
  deletes `docs/quality.toml`; from then on plain clippy keeps it clean.
  The audit checks every project has one or the other.
- **QUAL-503: leave no new findings in files you touch.** The boy scout
  rule: a change leaves each file it touches no worse, and better where
  it's cheap. The ratchet holds the counts; review holds the judgment
  rules. Rewriting working code for style alone, in files a change doesn't
  otherwise touch, belongs in its own issue.
- **QUAL-504: waivers are `#[expect]` with a reason.** When a lint is
  wrong for one item, waive it there with
  `#[expect(clippy::lint_name, reason = "why it is fine here")]`. Prefer
  `expect` to `allow`, since it warns once the waiver is no longer needed,
  and never waive without a `reason` (clippy
  `allow_attributes_without_reason`, a hard lint from phase 1). A waiver
  for a whole crate or module needs the PR to say why.
- **QUAL-505: a quality badge per project.** Each project README shows its
  findings left (green once graduated) next to its eval coverage badge,
  and the README's repo health section has one for the whole repo.
  Phase 3.
- **QUAL-506: review in Google's order and cite rule IDs.** A reviewer
  looks at design first, then function, complexity, tests, naming and
  comments ([Google's code review
  guide](https://google.github.io/eng-practices/review/)), and approves a
  change that improves the code's health even if it isn't perfect.
  Findings cite a QUAL ID and the source's own term (`C-GETTER`, "feature
  envy", "shallow module"), so the fix is easy to look up. The
  `code-review` skill files them as issue files; the weekly
  `deep-project-review` runs that rubric as one section from phase 3.
- **QUAL-507: new tools warn for a week before they fail.** A newly
  adopted check runs as a warning for one week, then fails CI, as SEC-005
  did. Each tool goes on the [tech radar](tech-radar.md) in the PR that
  adopts it.
