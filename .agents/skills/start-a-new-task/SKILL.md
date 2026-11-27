---
name: start-a-new-task
description: Pick one TODO or FIX from this monorepo's issue files (issues/ folders), following its priority rules, and complete it end to end with tests and a draft pull request. Use when asked to "start a new task", "work the next issue", or for scheduled autonomous work on the repo.
metadata:
  replaces: agent_copilot/prompts/start-a-new-task.md
---

# Start a new task

Complete exactly one issue in this repo. Read `AGENTS.md` at the repo root first.

## 1. Find one issue

Run the pick first; it applies every rule below and prints the issue's
text and its epic notes, so you don't need to read the issue list or the
READMEs yourself:

```bash
cargo run --quiet --manifest-path chores/Cargo.toml -- next-task
```

Add `--project <name>` when your thread is scoped to a project, and
`--skip <path>` for an issue an open PR already handles. A coordinator
starting several threads runs it once with `--count <n> --exclude
<projects or epic:names other threads have>` to get one issue per project.
Your judgment is only whether the pick is really workable now (a phase
that needs something first, a human task it waits on); if not, run it
again with `--skip`. The rules it applies, for reference:

Be eager: take the first issue that qualifies under these rules.

1. An issue is a file in an `issues/` folder (one file per issue, format in
   `standards/docs/issue-management.md`) or a code comment starting with
   `TODO` or `FIX`. `mise run //:issues` lists the files, priority first.
2. Skip anything tagged `agent-ignore`, and never take work from
   `issues/human/`: those tasks need a person. If an issue turns out to need
   one (a dashboard, a secret, a sign-off), add a file to `issues/human/` for
   that part, saying why it can't be automated and linking its runbook in
   `standards/docs/runbooks/`, which you write in the same PR (the format
   is under "Human issues" in `standards/docs/issue-management.md`).
3. Never work in a project listed under `# On Hold Projects` in the root
   `issues/README.md`.
4. Search in this order and stop at the first hit:
   1. `priority: priority` issues in the root `issues/`. Within an epic, take
      the earliest phase and lowest `order`.
   2. `priority: priority` issues of each project under `# Priority Projects`
      in the root `issues/README.md`, in list order (a project's sub-projects,
      such as `cahokia/web/issues/`, count as part of it).
   3. Any other issue, priority before backlog.
   4. `TODO`/`FIX` comments anywhere in the code.
   Read the folder's `README.md` too: it has the epic's notes and what a
   phase needs first.
5. If nothing qualifies, investigate the codebase, record a few real issues as
   issue files in the right `issues/` folder, and open a PR with just those.

## 2. Set up

Install the project's tooling with the commands in the **Dev commands** section
of `AGENTS.md`, from the repo root and again inside the project you will change.

## 3. Do the work

- If the issue is too big for one PR, break it into smaller issues instead:
  one issue file each, written with the `write-issue` skill (tagged
  `[agent-generated]`, sharing an `epic` and an
  `order` when they are phases of one piece of work), delete the original
  issue's file, and open a PR with only that change.
- Otherwise, use test-driven development:
  1. Write failing tests for the issue (see `standards/docs/testing.md`; UI
     changes also need Playwright tests).
  2. Implement until they pass, then add tests for edge cases you notice.
  3. Run the CI task from **Dev commands** for every project you changed and fix
     everything it reports.
  4. Update documentation affected by the change (`standards/docs/documentation.md`).
  5. Delete the issue's file (or remove the `TODO`/`FIX` comment) in the same
     PR.
- Out-of-scope improvements you spot: add an issue file tagged
  `[agent-generated]` (the `write-issue` skill), or leave a `TODO (agent-generated): ...` comment next
  to the code, instead of fixing them.
- If you learned something that would help the next agent, add it to
  `AGENTS.md` or the relevant skill rather than a tool-specific file.

## 4. Deliver

Follow the `open-pull-request` skill. One issue per PR.
