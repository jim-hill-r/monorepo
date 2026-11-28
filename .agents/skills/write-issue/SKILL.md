---
name: write-issue
description: Write an issue file (a TODO or FIX in a docs/issues/ folder) or a human task with its runbook in this monorepo, using the chores command so the format and the audit are right the first time. Use whenever you record new work, split an issue into phases, or ask Jim for something only a person can do.
---

# Write an issue

Issues are one Markdown file each (`standards/docs/issue-management.md`).
The `chores` commands write the front matter, tags, date and sections, so
you only decide what the issue says and where it goes. Run them from the
repo root; cargo is in every sandbox.

```bash
c() { cargo run --quiet --manifest-path chores/Cargo.toml -- "$@"; }
```

## An agent issue

```bash
c new-issue --folder <project or sub-project, or . for repo-wide> \
  --slug <short-kebab-slug> --kind todo|fix --priority priority|backlog \
  [--epic "<Epic> / Phase <n>: <name>" --order <10, 20, ...>] [--ignore] \
  --text "<what to do and why, with links relative to the issue file>"
```

Your judgment:

- **Where:** the project's own folder (`cookbook`, `cahokia/web`); the file
  goes in its `docs/issues/`. Work across projects goes to the platform
  project that owns it (`evals`, `infra`, `threat`, `cost`), or `.` for
  `standards` when none fits. Never a project under `# On Hold Projects` in
  `standards/docs/issues/README.md` unless Jim asked.
- **Priority:** `priority` only for work that should come before the
  project's other issues; most new findings are `backlog`.
- **Text:** one task, specific enough to start without asking: the file
  or behaviour, what done looks like, and a link to the plan, review or
  decision behind it. Links are relative to the issue file
  (`../docs/...`, `../../standards/docs/...`).
- **Epics:** phases of one piece of work share `--epic` and step `--order`
  by 10. The epic's notes (goal, plan link, what a phase needs first) go
  under a heading named after the epic in the folder's `README.md`.

## A task only a person can do

Only for a dashboard, an account, a first secret, billing or a sign-off.
A credential automation could make belongs in `infra/scripts/mint.sh`
instead (AGENTS.md).

```bash
c new-human-task --folder <project> --slug <short-kebab-slug> --title "<the task in plain words>" \
  --requirement "<what to do, where, and what it unblocks>" \
  --why "<why no workflow or agent can do it>" \
  [--unblocks "<what it unblocks>"] [--runbook <existing runbook name>]
```

It writes `<project>/docs/issues/human/<slug>.md` (the project the task
belongs to; `--folder` defaults to `standards` for repo-wide tasks) with
the three sections the audit needs
(ISS-004). Unless `--runbook` names an existing runbook, it also starts
`standards/docs/runbooks/<slug>.md` from `TEMPLATE.md`; fill in every
section (kind, time, cost, steps, where secrets go, verify, undo) before you
commit. Also ask Jim in your thread, so he hears about it before the
weekly digest.

## Then

Commit the files with the PR they belong to. The hooks run the audit
(`issues.sh --check`); if it still reports something, fix the file by hand.
To close an issue, delete its file in the PR that does the work.
