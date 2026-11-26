# 0008. Keep the custom CodeQL workflow and add a weekly Rust scan

- Status: Proposed (accepted when Jim merges the PR that adds it)
- Date: 2026-11-26
- Asked by: Jim ("I have dependabot and I think I have a custom GitHub
  workflow for security scanning. One of the human issues was to turn on
  security scanning but previously rust was not working. Research this and
  determine if I should remove the custom workflow or if we need to update
  it.")

## Context

Four security tools run on this repo today. They cover different things:

| Tool | Finds | Where it runs |
| --- | --- | --- |
| `.github/workflows/all-codeql.yml` (CodeQL, advanced setup) | Bugs in **our code**: injection, unsafe data flows, workflow script injection | JavaScript/TypeScript and Actions on every PR, push to `main` and weekly |
| Dependabot (`.github/dependabot.yml`) | **Out-of-date dependencies**, as weekly version bump PRs | GitHub, weekly |
| Threat check SEC-001 (gitleaks) | **Committed secrets** | Hooks, PR CI, weekly full history |
| Threat check SEC-003/004 (cargo-deny, npm audit) | **Known advisories in dependencies** | Hooks and PR CI when a lockfile changes, nightly |

The custom CodeQL workflow (PR #382, 2026-01-24) replaced GitHub's default
setup to drop Rust, because a Rust scan took about 26 minutes. That was
already after CodeQL's Rust support became generally available, with no
build needed, in October 2025
([changelog](https://github.blog/changelog/2025-10-14-codeql-scanning-rust-and-c-c-without-builds-is-now-generally-available)).
So Rust "works" in CodeQL. The problem is time, not support. Since then
nobody scans our Rust code (as opposed to our Rust dependencies) for
security bugs. Clippy is a lint, not a security scanner.

The human task in `issues/human/github-settings-code-security-turn-secret.md`
is about different switches: **secret scanning** with push protection,
**Dependabot alerts and security updates**, and **private vulnerability
reporting**. None of them is CodeQL or depends on Rust support. That task
still stands as written.

## Options

All options cost **$0**. This is a public repo, so CodeQL, code scanning and
GitHub-hosted Actions minutes are free and don't count against a quota
([free-tiers.md](../free-tiers.md)). The only cost is wall-clock time, and
Claude Code effort, which is small for every option (one workflow edit, well
under $1 of the $25 a month budget).

1. **Remove the custom workflow and turn on default setup with Rust.**
   - Every PR and push to `main` runs a whole-repo Rust scan of about 26
     minutes.
   - The merge rule waits for every check, so every agent PR would wait
     about 26 minutes longer to merge.
   - Default setup can't be scoped to the projects a PR changes.
   - It also needs Jim to flip the setting by hand.
2. **Remove the custom workflow and use default setup without Rust.** This
   is the same coverage as today, but in a dashboard setting instead of a
   reviewed file. Rust code stays unscanned.
3. **Keep the custom workflow and add Rust as a weekly-only job
   (recommended).**
   - PRs stay as fast as today.
   - Rust code gets a full CodeQL scan every Sunday, about 26 minutes a
     week (around 110 Actions minutes a month, free). It can also be run by
     hand.
   - Findings land in the Security tab with the others.
   - Nothing to change in the dashboard.
4. **Keep it as it is.** $0 and no change, but our Rust code is never
   scanned, which is the gap the threat check exists to close.

## Decision

Option 3. Keep `all-codeql.yml` and add an `analyze-rust` job. It runs on
the weekly schedule and on `workflow_dispatch` only, with
`build-mode: none` and a 60-minute timeout. JavaScript/TypeScript and Actions
keep running on every PR. Default code scanning setup stays off: GitHub
runs only one of the two.

## Consequences

- A Rust CodeQL finding can show up as late as a week after the PR that
  caused it. PR-time Rust checks stay clippy and cargo-deny.
- The first weekly run will show how long a Rust scan takes now and how many
  alerts it finds. If it is well under 10 minutes, revisit running it on PRs
  that change Rust code. If it finds a backlog, file the real ones as fix
  issues.
- The threat check's phase 3 (pin third-party actions to commits, zizmor)
  applies to this workflow too.
