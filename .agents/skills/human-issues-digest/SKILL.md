---
name: human-issues-digest
description: Gather the tasks only a person can do (the issue files in issues/human/ and open needs-human PRs) and post one short digest to Jim. Use for the weekly scheduled digest, or when asked what is waiting on Jim or on a human.
---

# Human issues digest

Bring every task that is waiting on a person to Jim's attention in one
message. Read `AGENTS.md` first. The process is in
`standards/docs/issue-management.md` ("Human issues"). This skill only reads
and reports; it changes nothing in the repo or on GitHub.

## 1. Gather

1. On the latest `main`, read every task in the root `issues/human/` (one
   file each, not `README.md`): `mise run //:issues -- --human` lists them
   oldest first with what each unblocks.
2. Find the tasks added in the last 7 days: those whose `added` date is
   within the week, or
   `git log --since="7 days ago" --diff-filter=A --name-only -- issues/human/`.
3. List open pull requests labelled `needs-human`. Until PR triage has
   renamed it, also include PRs still labelled `needs-jim`. For each, note the
   number, title, how many days it has been open, and the reason it needs a
   person (from the latest triage comment or the PR description).
4. Look for stray manual tasks the process missed: issues tagged
   `agent-ignore` in any other `issues/` folder that say "Needs Jim" or name a
   dashboard, secret or account. List them under "Misfiled" so they can be
   moved.

## 2. Check what is already done

Where you can check a task without any write (for example, whether a repo
secret now exists, from the secrets list if your account can read it), do so.
List a task that looks done under "Looks done" with what you saw, and offer to
remove it. Never remove a task without Jim's go-ahead.

## 3. Report

Post one message. If nothing is open, say so in one line. Otherwise:

- First line: how many tasks are waiting, and how many are new this week.
- **New this week**, then **Still open**, oldest first. One line each: the
  task in plain words, where to do it, and what it unblocks. PRs as
  `#<n> <title>: <reason>`.
- **Looks done** and **Misfiled**, only when there are any.
- End with one line: tell any thread when a task is done and it will remove
  it.

Keep it short and in full sentences, with no repo jargon Jim would have to
look up.

## 4. When Jim says a task is done

Delete its file from `issues/human/` and open a PR with the
`open-pull-request` skill. If the task unblocks an agent issue, mention it in
the PR description.
