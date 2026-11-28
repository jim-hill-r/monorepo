---
name: human-issues-digest
description: Gather the tasks only a person can do (the issue files in docs/issues/human/, due checks of settings made by hand, and open needs-human PRs) and post one short digest to Jim. Use for the weekly scheduled digest, when asked what is waiting on Jim or on a human, or when Jim answers a settings check.
---

# Human issues digest

Bring every task that is waiting on a person to Jim's attention in one
message. Read `AGENTS.md` first. The process is in
`standards/docs/issue-management.md` ("Human issues"), and the checks of
settings made by hand are under "Human checks" in `standards/docs/evals.md`.
This skill changes the repo only to add due settings checks to
`docs/issues/human/` (step 2) and to record Jim's answers (step 5), each
through one PR; it never changes anything on GitHub or in a dashboard
itself.

## 1. Gather

1. On the latest `main`, read every task in a project's `docs/issues/human/` (one
   file each, not `README.md`): `mise run //:issues -- --human` lists them
   oldest first with what each unblocks.
2. Find the tasks added in the last 7 days: those whose `added` date is
   within the week, or
   `git log --since="7 days ago" --diff-filter=A --name-only -- '*/docs/issues/human/'`.
3. List open pull requests labelled `needs-human` or `human-review` (the
   Human review workflow's hold: waiting only for a human's approval, which
   merges it). Until PR triage has renamed it, also include PRs still
   labelled `needs-jim`. For each, note the
   number, title, how many days it has been open, and the reason it needs a
   person (from the latest triage comment or the PR description).
4. Note any task that has no runbook or no "Why it can't be automated"
   section (`mise run //:issues -- --check` reports ISS-004); list it under
   "Misfiled" so an agent adds them.
5. Look for stray manual tasks the process missed: issues tagged
   `agent-ignore` in any other `docs/issues/` folder that say "Needs Jim" or name a
   dashboard, secret or account. List them under "Misfiled" so they can be
   moved.
6. In `cost/`, run `cargo run -q -p cost_cli -- renewals --within 60`. It
   lists every `[[cost]]` (a domain, a yearly plan) whose next charge is
   in the next 60 days, with its price, and any whose `renews` date has
   already passed. For each domain on the list, check that its project
   still deploys to it (its `wrangler.toml`, DNS records or README); one
   that nothing serves any more is one to consider dropping.

## 2. Add the settings checks that are due

Settings made by hand (connectors, secrets, Auth0, Cloudflare tokens,
billing) are listed in `standards/settings.toml`, each with how often Jim
checks it.

1. In `evals/`, run `cargo run -q -p evals_cli -- human-checks due --write`.
   It writes `standards/docs/issues/human/settings-check-<id>.md` for each setting that was never
   checked, was last checked longer ago than its `every`, or was last
   answered "not ok", skipping any whose file already exists, and at most
   five a week (`--limit`) so the digest stays short; the rest come up next
   week. Without `--write` it only lists them.
2. If it wrote any, open one PR with the `open-pull-request` skill ("Human
   checks due, <date>"), changing only those files.
3. List every open `settings-check-*.md` task in the digest under **Checks due**, one
   line each: the setting, what to look at and where, ending "reply ok or
   what is wrong".

## 3. Check what is already done

Where you can check a task without any write (for example, whether a repo
secret now exists, from the secrets list if your account can read it), do so.
List a task that looks done under "Looks done" with what you saw, and offer to
remove it. Never remove a task without Jim's go-ahead.

## 4. Report

Post one message. If nothing is open, say so in one line. Otherwise:

- First line: how many tasks are waiting, and how many are new this week.
- **New this week**, then **Still open**, oldest first. One line each: the
  task in plain words, where to do it, what it unblocks, and a link to its
  runbook (the file's `runbook` field, as a GitHub link on `main`). PRs as
  `#<n> <title>: <reason>`.
- **Checks due**, only when there are any: every open
  `standards/docs/issues/human/settings-check-*.md` task, new or not. These go here and not under
  the lists above.
- **Renewals**, only when there are any: one line per renewal from step 1.6,
  with its price and date, and "consider dropping" on a domain nothing
  serves any more.
- **Looks done** and **Misfiled**, only when there are any.
- End with one line: tell any thread when a task is done and it will remove
  it.

Keep it short and in full sentences, with no repo jargon Jim would have to
look up.

## 5. When Jim says a task is done, or answers a check

Delete its file from its project's `docs/issues/human/` and open a PR with the
`open-pull-request` skill. If the task unblocks an agent issue, mention it in
the PR description.

For a `settings-check-<id>.md` task, record his answer instead, in one PR, from
`evals/`; the command also deletes the task file:

- "ok" (or "fine", "yes"):
  `cargo run -q -p evals_cli -- human-checks answer <id> --ok`
- Anything wrong:
  `cargo run -q -p evals_cli -- human-checks answer <id> --not-ok "<his words>"`,
  then file the fix: a `kind: fix` issue in the owning project's `docs/issues/`
  when an agent can fix it, or a new `docs/issues/human/` task saying what to
  change when only Jim can. The check comes up again next week until a later
  "ok".

The command appends to `standards/settings-checks.jsonl`; never edit that
file by hand. If what Jim says shows the manifest entry itself is wrong (a
renamed page, a setting that no longer exists), fix `standards/settings.toml`
in the same PR.
