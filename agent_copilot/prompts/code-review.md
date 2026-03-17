# Code Review prompt

## Task
Perform a general code quality review of one project in this monorepo and record the findings as actionable TODO items.

## Instructions
1. Select a project to review per the Select Project section below.
2. Setup the dev environment per the Setup Environment section below.
3. Perform the review and record findings per the Review section below.

## Select Project

1. List all top-level directories in the repository root that contain a `Cargo.toml` or `package.json` (i.e., they are actual projects, not meta-directories like `.github` or `standards`).
2. Exclude any project that is listed in the `# On Hold Projects` section of the root `ISSUES.md` or of its own `ISSUES.md`.
3. From the remaining projects, prefer the one whose `ISSUES.md` has the fewest existing `TODO (agent-generated):` items. If multiple projects tie, pick the first alphabetically.
4. If a project does not have an `ISSUES.md` at all, treat it as having zero TODO items (highest priority for review).

## Setup Environment
1. Run `./cast/cli/artifacts/x86_64-unknown-linux-gnu/cast install` in the root directory.
2. Run `cast install` in the root directory.
3. Run `cast install` inside the selected project directory before beginning work.
4. Use `cast -h` to explore available commands such as `build`, `test`, and `ci`.

## Review

Perform a thorough code quality review of the selected project using the steps below.

### 1. Read the Standards

Read the relevant standards documents in `standards/docs/` before inspecting the code:
- `standards/docs/rust.md` – Rust coding standards (clippy lints, error handling, unsafe code rules)
- `standards/docs/testing.md` – Testing standards (unit test coverage, E2E test requirements)
- `standards/docs/documentation.md` – Documentation standards
- `standards/docs/naming.md` – Naming conventions
- `standards/docs/typescript.md` – TypeScript standards (if the project has TypeScript)

### 2. Inspect the Code

Review the source files of the selected project, focusing on:
- **Error handling**: Are `.unwrap()` / `.expect()` calls used where proper error propagation should be used instead? (See `standards/docs/rust.md`)
- **Clippy lints**: Does `Cargo.toml` configure the required clippy lints (`unwrap_used = "warn"`, `expect_used = "warn"`, `unsafe_code = "forbid"`)?
- **Test coverage**: Are there unit tests for all non-trivial functions? Are edge cases covered? Does the project meet the testing standards in `standards/docs/testing.md`?
- **Code clarity**: Are there long functions, duplicated logic, magic numbers, or unclear variable names that could be refactored?
- **Performance**: Are there obvious inefficiencies such as unnecessary cloning, repeated allocations, or blocking calls in async contexts?
- **Documentation**: Are public APIs and complex logic documented with comments? Does the project have a README that matches the standards in `standards/docs/documentation.md`?

### 3. Produce Exactly 5 Findings

Select the 5 most impactful improvements and record each as a TODO item. Each finding must:
- Be specific (reference a file and function/struct name where possible)
- Be actionable (describe exactly what should be changed)
- Be marked with `TODO (agent-generated):`

### 4. Record the Findings

Add the 5 TODO items to the selected project's `ISSUES.md` under `# Priority Issues`. If the project does not have an `ISSUES.md`, create one with the following structure:

```markdown
# Priority Issues

- TODO (agent-generated): <finding 1>
- TODO (agent-generated): <finding 2>
- TODO (agent-generated): <finding 3>
- TODO (agent-generated): <finding 4>
- TODO (agent-generated): <finding 5>

# Backlog
```

### 5. Verify

After recording the findings, re-read the project's `ISSUES.md` to confirm all 5 TODO items are present and correctly formatted.

## Context

This prompt is used to automatically perform periodic code quality reviews of projects in the monorepo. The output (the 5 TODO items in ISSUES.md) will be picked up by future agent runs via the `start-a-new-task` prompt.
