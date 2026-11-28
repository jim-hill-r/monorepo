---
name: human-issues-digest
description: Gather the tasks only a person can do (the issue files in issues/human/, due checks of settings made by hand, orphans from the weekly resource sweep, and open needs-human PRs) and post one short digest to Jim. Use for the weekly scheduled digest, when asked what is waiting on Jim or on a human, or when Jim answers a settings check.
---

# Human issues digest

Bring every task that is waiting on a person to Jim's attention in one
message. Read `AGENTS.md` first. The process is in
`standards/docs/issue-management.md` ("Human issues"), and the checks of
settings made by hand are under "Human checks" in `standards/docs/evals.md`.
This skill changes the repo only to add due settings checks to
`issues/human/` (step 2), to propose entries in `standards/resources.toml`
for what the resource sweep found (step 3) and to record Jim's answers
(step 6), each through PRs; it never changes anything on GitHub or in a
dashboard itself, and never deletes a resource.

## 1. Gather

1. On the latest `main`, read every task in the root `issues/human/` (one
   file each, not `README.md`): `mise run //:issues -- --human` lists them
   oldest first with what each unblocks.
2. Find the tasks added in the last 7 days: those whose `added` date is
   within the week, or
   `git log --since="7 days ago" --diff-filter=A --name-only -- issues/human/`.
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
   `agent-ignore` in any other `issues/` folder that say "Needs Jim" or name a
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
   It writes `issues/human/settings-check-<id>.md` for each setting that was never
   checked, was last checked longer ago than its `every`, or was last
   answered "not ok", skipping any whose file already exists, and at most
   five a week (`--limit`) so the digest stays short; the rest come up next
   week. Without `--write` it only lists them.
2. If it wrote any, open one PR with the `open-pull-request` skill ("Human
   checks due, <date>"), changing only those files.
3. List every open `settings-check-*.md` task in the digest under **Checks due**, one
   line each: the setting, what to look at and where, ending "reply ok or
   what is wrong".

## 3. Act on the resource sweep

The weekly resource sweep (the `threat-live` job of the All Free Tier
Usage workflow, Mondays) sorts every live Cloudflare and GitHub resource
against what the repo claims ("The weekly sweep" in
`standards/docs/resources.md`). A workflow can't make the judgement calls,
so this step does, with no extra scheduled task
([plan](../../../standards/docs/plans/2026-11-resource-cleanup.md), Part 3).

1. With your GitHub tools, find the latest completed run of All Free Tier
   Usage on `main` and read `resource-sweep/report.json` from its
   `threat-live` artifact (kept 7 days). It has `today`, `rows` (each a
   `kind`, `name`, `status` and `note`) and `unlisted` (kinds a token
   couldn't read). If there is no run in the last 8 days, or no report in
   it, say so under **Resources** in the digest and skip the rest of this
   step.
2. **`orphan` rows.** For each one that no open PR already proposes (look
   for its `kind` and `name` in the `standards/resources.toml` diff of open
   PRs), open one draft PR with the `open-pull-request` skill
   ("Resources: retire <kind> <name>"), changing only
   `standards/resources.toml`:
   - a `[[retired]]` entry with `kind`, `name`, every `url` it answers on,
     `why` (what it was, if its name or history says, and that the sweep
     found nothing claiming it), `retired` (today) and `delete_after` (7
     days later), and `replaced_by` when a claimed resource clearly took
     its place;
   - or a `[[keep]]` entry with `why` and a `review` date six months out,
     when it is clearly Jim's own and made outside the repo (a hand-made
     landing page, a resource named after something only he uses).
   The Resource entries job labels it `needs-human`, so it waits for Jim;
   his approval is the approval to delete. When you can't tell what it is,
   propose `[[retired]]` and say in the PR that you couldn't tell. One PR
   per orphan, so Jim can approve or refuse each on its own.
3. **`gone` rows.** A `[[retired]]` entry the sweep no longer finds live:
   set its `deleted` to the report's `today` date, all of them in one PR
   ("Resources: mark deleted, <date>"). It adds no entry, so it needs no
   approval beyond the usual review.
4. **Rows only Jim can decide** go in the digest, not in a PR:
   `keep-overdue` (keep it for another six months, or retire it?),
   `retired-still-live` (the cleanup failed; link the run), and `no-pr`
   (a branch with no PR, untouched for 14 days: delete it, or open a PR?).
   List each `unlisted` kind too, with its `why`, so a token that lost a
   permission is noticed.

## 4. Check what is already done

Where you can check a task without any write (for example, whether a repo
secret now exists, from the secrets list if your account can read it), do so.
List a task that looks done under "Looks done" with what you saw, and offer to
remove it. Never remove a task without Jim's go-ahead.

## 5. Report

Post one message. If nothing is open, say so in one line. Otherwise:

- First line: how many tasks are waiting, and how many are new this week.
- **New this week**, then **Still open**, oldest first. One line each: the
  task in plain words, where to do it, what it unblocks, and a link to its
  runbook (the file's `runbook` field, as a GitHub link on `main`). PRs as
  `#<n> <title>: <reason>`.
- **Checks due**, only when there are any: every open
  `issues/human/settings-check-*.md` task, new or not. These go here and not under
  the lists above.
- **Renewals**, only when there are any: one line per renewal from step 1.6,
  with its price and date, and "consider dropping" on a domain nothing
  serves any more.
- **Resources**, only when step 3 found anything: one line per retirement
  or keep PR it opened (`#<n> <kind> <name>: <why>`), then each
  `keep-overdue`, `retired-still-live` and `no-pr` row as a question for
  Jim, then any kind the sweep couldn't read, or a line saying the sweep
  report was missing.
- **Looks done** and **Misfiled**, only when there are any.
- End with one line: tell any thread when a task is done and it will remove
  it.

Keep it short and in full sentences, with no repo jargon Jim would have to
look up.

## 6. When Jim says a task is done, or answers a check

Delete its file from `issues/human/` and open a PR with the
`open-pull-request` skill. If the task unblocks an agent issue, mention it in
the PR description.

For a `settings-check-<id>.md` task, record his answer instead, in one PR, from
`evals/`; the command also deletes the task file:

- "ok" (or "fine", "yes"):
  `cargo run -q -p evals_cli -- human-checks answer <id> --ok`
- Anything wrong:
  `cargo run -q -p evals_cli -- human-checks answer <id> --not-ok "<his words>"`,
  then file the fix: a `kind: fix` issue in the owning project's `issues/`
  when an agent can fix it, or a new `issues/human/` task saying what to
  change when only Jim can. The check comes up again next week until a later
  "ok".

The command appends to `standards/settings-checks.jsonl`; never edit that
file by hand. If what Jim says shows the manifest entry itself is wrong (a
renamed page, a setting that no longer exists), fix `standards/settings.toml`
in the same PR.
