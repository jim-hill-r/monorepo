---
name: human-issues-digest
description: Gather the tasks only a person can do (TODO (human) lines in HUMAN_ISSUES.md, due checks of hand-made settings, and open needs-human PRs) and post one short digest to Jim. Use for the weekly scheduled digest, when asked what is waiting on Jim or on a human, or when Jim answers a settings check.
---

# Human issues digest

Bring every task that is waiting on a person to Jim's attention in one
message. Read `AGENTS.md` first. The process is in
`standards/docs/issue-management.md` ("Human issues"), and the checks of
settings made by hand are under "Human checks" in `standards/docs/evals.md`.
This skill changes the repo only to add due settings checks to
`HUMAN_ISSUES.md` (step 2) and to record Jim's answers (step 5), each
through one PR; it never changes anything on GitHub or in a dashboard
itself.

## 1. Gather

1. On the latest `main`, read every `TODO (human): ...` line under `# Open` in
   the root `HUMAN_ISSUES.md`.
2. Find the lines added in the last 7 days:
   `git log --since="7 days ago" -p -- HUMAN_ISSUES.md`, keeping added lines
   that start with `- TODO (human)`.
3. List open pull requests labelled `needs-human`. Until PR triage has
   renamed it, also include PRs still labelled `needs-jim`. For each, note the
   number, title, how many days it has been open, and the reason it needs a
   person (from the latest triage comment or the PR description).
4. Look for stray manual tasks the process missed: `TODO (agent-ignore)`
   lines in any `ISSUES.md` that say "Needs Jim" or name a dashboard, secret
   or account. List them under "Misfiled" so they can be moved.

## 2. Add the settings checks that are due

Settings made by hand (connectors, secrets, Auth0, Cloudflare tokens,
billing) are listed in `standards/settings.toml`, each with how often Jim
checks it.

1. In `evals/`, run `cargo run -q -p evals_cli -- human-checks due`. It
   prints one ready-made `- TODO (human): Check <id>: ...` line for each
   setting that was never checked, was last checked longer ago than its
   `every`, or was last answered "not ok".
2. Skip any line whose `Check <id>:` is already in `HUMAN_ISSUES.md`. Of the
   rest, take at most five, in the order printed, so a week's digest stays
   short; the others come up next week.
3. Add them under `# Open` in `HUMAN_ISSUES.md` and open one PR with the
   `open-pull-request` skill ("Human checks due, <date>"), changing only
   `HUMAN_ISSUES.md`.
4. List them in the digest under **Checks due**, one line each: the setting,
   what to look at and where, ending "reply ok or what is wrong".

## 3. Check what is already done

Where you can check a task without any write (for example, whether a repo
secret now exists, from the secrets list if your account can read it), do so.
List a task that looks done under "Looks done" with what you saw, and offer to
remove its line. Never remove a line without Jim's go-ahead.

## 4. Report

Post one message. If nothing is open, say so in one line. Otherwise:

- First line: how many tasks are waiting, and how many are new this week.
- **New this week**, then **Still open**, oldest first. One line each: the
  task in plain words, where to do it, and what it unblocks. PRs as
  `#<n> <title>: <reason>`.
- **Checks due**, only when there are any: every open `Check <id>` line,
  new or not. These lines go here and not under the lists above.
- **Looks done** and **Misfiled**, only when there are any.
- End with one line: tell any thread when a task is done and it will remove
  the line.

Keep it short and in full sentences, with no repo jargon Jim would have to
look up.

## 5. When Jim says a task is done, or answers a check

Remove its line from `HUMAN_ISSUES.md` and open a PR with the
`open-pull-request` skill. If the task unblocks a `TODO (agent-generated)`
item, mention it in the PR description.

For a `Check <id>` line, also record his answer in the same PR, from
`evals/`:

- "ok" (or "fine", "yes"):
  `cargo run -q -p evals_cli -- human-checks answer <id> --ok`
- Anything wrong:
  `cargo run -q -p evals_cli -- human-checks answer <id> --not-ok "<his words>"`,
  then file the fix: a `FIX` in the owning project's `ISSUES.md` when an
  agent can fix it, or a new `TODO (human)` line saying what to change when
  only Jim can. The check stays due until a later "ok".

The command appends to `standards/settings-checks.jsonl`; never edit that
file by hand. If what Jim says shows the manifest entry itself is wrong (a
renamed page, a setting that no longer exists), fix `standards/settings.toml`
in the same PR.
