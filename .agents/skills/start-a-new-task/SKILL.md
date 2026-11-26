---
name: start-a-new-task
description: Pick one TODO or FIX from this monorepo's ISSUES.md files, following its priority rules, and complete it end to end with tests and a draft pull request. Use when asked to "start a new task", "work the next issue", or for scheduled autonomous work on the repo.
metadata:
  replaces: agent_copilot/prompts/start-a-new-task.md
---

# Start a new task

Complete exactly one issue in this repo. Read `AGENTS.md` at the repo root first.

## 1. Find one issue

Be eager: take the first issue that qualifies under these rules.

1. An issue is a line or comment starting with `TODO` or `FIX`.
2. Skip anything marked `(agent-ignore)`.
3. Never work in a project listed under `# On Hold Projects` in any `ISSUES.md`.
4. Search in this order and stop at the first hit:
   1. `# Priority Issues` in the root `ISSUES.md`, top to bottom. Items may sit
      in `##`/`###` subsections for larger epics; include those.
   2. The `ISSUES.md` of each project under `# Priority Projects` in the root
      `ISSUES.md`, in list order.
   3. Any other `ISSUES.md` in the repo.
   4. `TODO`/`FIX` comments anywhere in the code.
5. If nothing qualifies, investigate the codebase, record a few real issues in
   the right `ISSUES.md` (format below), and open a PR with just those.

## 2. Set up

Install the project's tooling with the commands in the **Dev commands** section
of `AGENTS.md`, from the repo root and again inside the project you will change.

## 3. Do the work

- If the issue is too big for one PR, break it into smaller issues instead:
  add them to the relevant `ISSUES.md` as `TODO (agent-generated): ...` and open
  a PR with only that change. Use a flat list under `# Priority Issues`; only use
  `##`/`###` subsections for a multi-phase epic in a project's own `ISSUES.md`.
- Otherwise, use test-driven development:
  1. Write failing tests for the issue (see `standards/docs/testing.md`; UI
     changes also need Playwright tests).
  2. Implement until they pass, then add tests for edge cases you notice.
  3. Run the CI task from **Dev commands** for every project you changed and fix
     everything it reports.
  4. Update documentation affected by the change (`standards/docs/documentation.md`).
  5. Remove the `TODO`/`FIX` you fixed.
- Out-of-scope improvements you spot: leave a `TODO (agent-generated): ...`
  comment next to the code instead of fixing them.
- If you learned something that would help the next agent, add it to
  `AGENTS.md` or the relevant skill rather than a tool-specific file.

## 4. Deliver

Follow the `open-pull-request` skill. One issue per PR.
