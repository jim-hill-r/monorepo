# Issue Management Standards

This document defines how issues, tasks and TODOs are tracked in this
monorepo. Work is tracked in the repo, not in GitHub issues.

## Issue Tracking Locations

1. **Issue files** - one Markdown file per issue, in the `docs/issues/`
   folder of the project it belongs to
   (decision [0006](decisions/0006-one-file-per-issue.md)):
   - `<project>/docs/issues/` for a project's own work (any project or
     sub-project folder, such as `pane/` or `cahokia/web/`).
   - Repo-wide work goes in the platform project that owns it (`evals`
     for the eval engine, `infra` for IaC, `threat` for the threat check,
     `cost` for cost and revenue), and in `standards/docs/issues/` when no
     other project fits.
   - `<project>/docs/issues/human/` for tasks only a person can do (see
     [Human issues](#human-issues)).
   - There is no root `issues/` folder, and no `<project>/issues/`: the
     audit fails an issue file there (ISS-001).
2. **TODO comments** - in-code markers for specific issues
3. **FIX comments** - in-code markers for bugs or problems

There are no `ISSUES.md` or `HUMAN_ISSUES.md` files any more. Many agent PRs
run in parallel, and when they all edited the same few list files they kept
conflicting. Adding an issue now adds a file, and closing one deletes it, so
two PRs only conflict when they change the same issue. `mise run //:audit`
fails (ISS-001) if an `ISSUES.md` or `HUMAN_ISSUES.md` comes back.

## Seeing the list

The list of issues is generated, never committed (a committed index would
conflict just like the old files):

```bash
mise run //:issues               # agent issues: priority first, then backlog
mise run //:issues -- --human    # human tasks, oldest first
mise run //:issues -- --check    # validate every issue file (part of the audit)
```

The same script is `bash standards/scripts/issues.sh`. Pane's snapshot reads
the same files for its Work and Human issues pages.

## Issue file format

The file name is a short kebab-case slug of the issue, unique in its folder:
`<project>/docs/issues/<slug>.md`, for example
`cahokia/docs/issues/auth-003-playwright-test-home-page-never-shows.md`. Don't
number files: two PRs picking the same next number would clash.

The file starts with front matter, then the issue's text in Markdown:

```markdown
---
kind: todo
priority: priority
tags: [agent-generated]
epic: "Review dashboard epic / Phase 1: read-only from the repo"
order: 30
added: 2026-11-26
---

Build the Work page: every `TODO` and `FIX` across the repo, filterable by
project, kind, priority or backlog, and tag.
```

| Field | Required | Values |
| --- | --- | --- |
| `kind` | yes | `todo` (work to do) or `fix` (something broken) |
| `priority` | yes | `priority` (work it first) or `backlog` |
| `added` | yes | the date it was added, `YYYY-MM-DD` |
| `tags` | no | `[agent-generated]` if an agent wrote it, `[agent-ignore]` if agents must not work it, or both |
| `epic` | no | the epic, and phase, it belongs to: `Epic` or `"Epic / Phase N: name"` |
| `order` | no | a whole number that orders the issues of one epic (use steps of 10 so an issue can go between two) |

Front matter is plain `key: value` lines. Quote a value that has a `:`, `#`
or quote in it. Links in the text are relative to the issue file, so a link
to the project's `docs/plans/` is `../plans/...` and one to the repo root
is `../../../...` from a top-level project.

### Notes and project lists

Prose that isn't one issue, such as an epic's goal, its plan link or what a
phase needs first, goes in the folder's `README.md` under a heading named
after the epic. `README.md` is never an issue.

[`standards/docs/issues/README.md`](issues/README.md) also holds the
`# Priority Projects` and `# On Hold Projects` lists.

## TODO Comment Conventions

When adding TODO comments in code, follow these conventions:

### Agent-Generated TODOs

Mark TODOs with `(agent-generated)` if created by an automated agent:

```rust
// TODO (agent-generated): Add validation for email format
```

### Agent-Ignore TODOs

Mark TODOs with `(agent-ignore)` if they should not be worked on by automated agents:

```rust
// TODO (agent-ignore): This requires manual architecture review
```

### Human TODOs

Tasks only a person can do go in the owning project's `docs/issues/human/`,
never among its agent issues or in a code comment. See [Human issues](#human-issues).

### Removing Completed TODOs

**Always remove** TODO and FIX comments when the work is completed, and
delete the issue file of a completed issue. Completed work should not leave
behind TODO markers.

## Finding Issues

When looking for work to do, check these in priority order:

1. **Priority projects' `docs/issues/`** with `priority: priority`, in the
   order of the list in `standards/docs/issues/README.md`.
2. **Other projects' `docs/issues/`** with `priority: priority`, then
   backlog issues.
3. **TODO/FIX comments in code** (skip items marked `(agent-ignore)`).

`chores next-task` picks in that order (with the product share
heuristic); `mise run //:issues` lists priority first, then backlog, by
project. Within an epic, work the phases and
`order` in order.

## Searching for References

When documenting files that need updates or searching for related code:

### For Code Files

Use `grep` to search non-markdown files, excluding build artifacts:

```bash
grep -r "pattern" --exclude-dir=.git --exclude-dir=target
```

### For Documentation

Use `grep` to search markdown files specifically:

```bash
grep -r "pattern" --include="*.md"
```

### For Dependencies

Find Cargo.toml files that reference a specific dependency:

```bash
find . -name "Cargo.toml" -exec grep -l "pattern" {} \;
```

**Important**: Always verify if metadata (like a `[package.metadata.*]` table) is just configuration vs actual dependencies.

## Creating New Issues

When creating issues that are too complex to tackle immediately:

1. Break the issue into smaller, manageable tasks.
2. Write one issue file per task in the right `docs/issues/` folder.
3. Tag new issues `[agent-generated]` if created by an agent.
4. Give the tasks of a multi-phase epic the same `epic` (with the phase) and
   an `order`, and put the epic's notes in the folder's `README.md`.

## Working on Issues

When working on an issue:

1. **Check On Hold status** - Do not work on issues in projects listed under `# On Hold Projects`
2. **Start with tests** - Use Test-Driven Development when possible
3. **Make minimal changes** - Only change what's necessary to fix the issue
4. **Update documentation** - Update relevant docs for your changes
5. **Add out-of-scope issues** - If you find improvements that are out of scope, add an issue file (or a `TODO (agent-generated)` comment next to the code)
6. **Close the issue** - Delete its issue file (and any TODO/FIX comment) in the PR that does the work

## Best Practices

1. **Be specific** - Write clear, actionable issues that explain what needs to be done
2. **Link to context** - Reference related issues, PRs, or documentation when helpful
3. **Prioritize appropriately** - Use `priority` for important work, `backlog` for nice-to-haves
4. **Keep issues up-to-date** - Delete completed or obsolete issue files
5. **One issue per file** - Never list several tasks in one issue file

## Human issues

Some tasks only a person can do: anything in a vendor dashboard (Auth0,
Cloudflare, PostHog), creating an account or a secret, payment and billing
checks, and sign-offs an agent may not give itself. Agents can't do these, so
they must reach Jim rather than sit in a project's issues or a chat thread
(decision [0002](decisions/0002-human-issues.md)).

### Where they live

- **One file per task** in the owning project's `docs/issues/human/`
  (repo-wide ones in `standards/docs/issues/human/`), written with
  `chores new-human-task --folder <project>`. Pane's Human issues page
  gathers every project's tasks into one list, each with its project:

  ```markdown
  ---
  added: 2026-11-27
  unblocks: "IaC applies on main (decision 0007)"
  runbook: standards/docs/runbooks/tofu-state-passwords.md
  ---

  ## Requirement

  Make two long random passwords for the OpenTofu state Worker and add
  them as GitHub secrets.

  ## Why it can't be automated

  Nothing in automation can make a secret yet without its value passing
  through git, a log or state.

  ## Runbook

  [Make the OpenTofu state passwords](../../../../standards/docs/runbooks/tofu-state-passwords.md)
  ```

  Every human task has these three sections (Jim, 2026-11-27: "document
  the requirement, why it can't be automated and create a run book. Link
  the run book in the task"):
  - **Requirement**: what to do, where, and what it unblocks.
  - **Why it can't be automated**: the reason no workflow or agent can do
    it (no API, a signed-in account, a first credential, a sign-off, a
    delete). If a bootstrap credential could make it, put it in the
    token-minting workflow instead of here.
  - **Runbook**: a link to its runbook in
    [`standards/docs/runbooks/`](runbooks/README.md), the step-by-step
    page (permissions, steps, where secrets go, how to verify, cost). Add
    the runbook in the same PR, or link an existing one. Runbooks stay
    after the task is done, so a rebuild can follow them.

  Front matter: `added` (required) says when, `runbook` (required) is the
  runbook's path from the repo root, and `unblocks` what it unblocks.
  `kind` and `priority` aren't used here. `mise run //:audit` fails
  (ISS-003) if an issue elsewhere is tagged `human`, and (ISS-004) if a
  human task lacks one of the three sections or its `runbook` field, or
  the runbook file doesn't exist.
- **PRs that need a person** get the `needs-human` label instead (see
  [pull-requests.md](pull-requests.md)). They are not copied into a file;
  the digest reads the label.
- **Questions in chat.** An agent that is blocked on something only Jim can
  do adds a human issue file in its PR as well as asking in its thread, so
  the task survives the thread. A plain question Jim can answer in words
  stays in the thread.

### How they reach Jim

A weekly scheduled task in the Claude project runs the
[`human-issues-digest`](../../.agents/skills/human-issues-digest/SKILL.md)
skill. It posts one message listing the open human issues and the open
`needs-human` PRs, oldest first, with what each one unblocks and what was
added since the last digest. It also adds a `settings-check-<setting>.md` task for
each setting made by hand that is due for Jim to look at, from
`standards/settings.toml` (see "Human checks" in [evals.md](evals.md#human-checks)).

### Closing one

Do the task, then delete its file in a PR (an agent will do this when told a
task is done). Keep its runbook: mark it done with the date, and fix any
step that turned out different on the day. If the task unblocks an agent issue, that issue becomes
workable as usual.

## Open PRs from before the change

A PR opened before decision 0006 that edits an old `ISSUES.md` or
`HUMAN_ISSUES.md` gets a modify/delete conflict once it merges `main`.
Resolve it by deleting the old file again (`git rm`) and writing each line
the PR added as an issue file, and deleting the issue file of each line the
PR removed.
