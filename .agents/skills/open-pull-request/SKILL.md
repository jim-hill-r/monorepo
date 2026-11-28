---
name: open-pull-request
description: Branch, commit, and open a pull request in this monorepo the way it expects - always as a draft, marked ready for review only once the agent judges it ready for a human. Use whenever you are about to push changes or open, update, or ready a PR in this repo.
---

# Open a pull request

## Before pushing

1. Work on a new branch with a unique, descriptive name, for example
   `<agent>/<short-issue-slug>` (`claude/fix-homer-parser`). Never push to `main`.
2. Set up the git hooks from **Dev commands** in `AGENTS.md` before your
   first commit. They run the CI task in every project your commit or push
   touches, so a commit or push that goes through has passed what PR CI
   checks. Fix every failure they report, and never skip them. If a tool
   can't be installed in your sandbox, follow the fallback in
   `standards/docs/git-hooks.md` and run the CI task by hand. CI on `main`
   may itself be red; judge your change by whether *your* projects pass.
3. Keep the diff to one issue. Don't commit build output, secrets, or local
   editor settings.
4. Don't open a new automated-task PR while another agent-authored PR is still
   open; finish that one first.

## Open it as a draft

Always open the PR as a **draft**, against `main`. Write the description
as "Pull request descriptions" in `standards/docs/pull-requests.md` says
(Jim, 2026-11-27), following `.github/pull_request_template.md`:

1. The attribution lines your platform requires, if any, first.
2. `## Attention`: the attention tags, one requirements, one cost, one
   threat/risk and one docs tag, from `chores badges` (never written by
   hand).
3. `## What changed`: at most 50 words.
4. `## Why`: at most 50 words, with any engineering tradeoffs.
5. `## Cost changes`: under 25 words, or `None.`
6. `## Risk changes`: under 25 words, or `None.`
7. `## Details`: everything else: the issue it addresses (the issue file
   it deletes, or the `TODO`/`FIX` comment), how it was tested (the hooks
   passed, or which commands you ran by hand and their result, and any
   check you couldn't run), anything a reviewer should look at closely,
   and links to the committed plan, decision, issue file or doc. Put long
   explanations in those files, in the same PR, never in the description.

Build it with the `pr-body` command rather than by hand. It writes that
shape and exits 1, printing the PR-001 warnings, until every section is
there and within its word limit, so the description is right before the
PR exists:

```bash
cargo run --quiet --manifest-path chores/Cargo.toml -- pr-body \
  --login <github login> --requested-by <name> --thread-url <project thread link> \
  --what "<what changed, at most 50 words>" \
  --why "<why, and tradeoffs, at most 50 words>" \
  --cost "<under 25 words; leave the flag out for None.>" \
  --risk "<under 25 words; leave the flag out for None.>" \
  --details "<issue it closes; hooks passed, or what you ran; checks that didn't run; review notes; links>" \
  --footer "<closing lines your platform asks for>" --out /tmp/pr-body.md
```

Pass the attribution and footer exactly as your platform's instructions
give them, and leave out the flags it doesn't ask for. Your judgment is
only the words. Send the printed body as the PR description.

The **Attention** section comes from `chores badges --base origin/main`
(commit first so the diff is complete): 🔴 `must-requirement-changed`,
🟡 `requirement-changed` or 🟢 `requirements-unchanged`; 🟠
`cost-changed` or 🟢 `cost-unchanged`; 🟠 `threat-risk-changed` or 🟢
`threat-risk-unchanged`; 🔵 `docs-only` (every changed file is docs,
plans included), 🟢 `docs-changed` or 🟢 `docs-unchanged` ("Attention tags" in
`standards/docs/pull-requests.md`). Never edit the tags by hand. When a
later push changes which files the PR touches, run `chores badges` and
replace the section. A red requirement tag means the PR gets
`needs-human` and waits for Jim. The Cost changes and Risk changes
summaries are your words on what actually moves; the tags only say which
files changed.

The **PR summary** check (PR-001) warns when a section is missing or
empty, a summary is over its limit, or a tag is unknown; fix the
description (`pr-body --check <file>` checks an edited one), and it reruns
on the edit.

## Mark it ready for review

Mark the PR ready for review once **all** of these hold:

- The change is complete for the issue; nothing is left as "follow-up" unless
  it's recorded as an issue file tagged `[agent-generated]` in a `docs/issues/`
  folder.
- CI for the changed projects passes on the PR's head commit.
- The `TODO`/`FIX` you fixed is removed and docs are updated.
- You've re-read the full diff and would be comfortable with a human reviewing it.
- The description has the Attention, What changed, Why, Cost changes,
  Risk changes and Details sections, and the PR summary check shows no
  warning.

Ready for review hands the PR to a human; it is not approval.

## Turn on auto-merge

Right after marking it ready, turn on GitHub auto-merge with squash
(`gh api -X PUT repos/jimhillr/monorepo/pulls/<n>/ccr/auto_merge -f merge_method=SQUASH`;
`gh pr merge --auto` needs GraphQL, which Claude sessions can't use), so the PR lands as soon
as a reviewer approves it. Skip this when the PR is labelled `needs-human` or
`credential-change`, or a check is red, even one also failing on `main`.
A PR labelled `cost` or `human-review`, or whose Attention section has a
red tag (`must-requirement-changed`) or `cost-changed`, needs a human's
approval ("Human review" in `standards/docs/pull-requests.md`). That is
not a check to wait for: mark it ready and turn auto-merge on once its
checks are green, like any other PR, so his approval merges it (Jim,
2026-11-28: "Yes, merge on my approval").
The auto-merge rules are under
"Auto-merge" in `standards/docs/pull-requests.md`. You still never approve
your own PR, so it waits for a human's approval either way.

## Stacked PRs: one approval for the stack

If your PR is one of a stack (its base is another PR's branch), don't hand
Jim each step. Once every PR in the stack is complete and green, retarget
the top PR to `main`, add a `Stack: #a, #b, #c` line to its description
(lowest first, top last), with What changed and Why covering the whole stack, move the other
step PRs back to draft with a comment naming the top PR, and mark only the
top PR ready. Close the step PRs once it merges. The steps are under
"Stacked pull requests" in `standards/docs/pull-requests.md` (decision
0014).

## Working on a PR that already exists

Before you start any work on an existing PR (a reviewer's changes, a CI fix,
a merge from `main`, any push), **convert it back to draft first** (Jim,
2026-11-26), and turn off its auto-merge
(`gh api -X DELETE repos/jimhillr/monorepo/pulls/<n>/ccr/auto_merge`). Do the work, then mark it ready for review
and turn auto-merge on again only once it
meets every point of the list above, including green checks on the new head
commit. Move the PR back to draft too if it stops meeting the list for any
other reason.

Never approve your own PR. Once it is mergeable as defined in
`standards/docs/pull-requests.md` (not a draft, no conflict, an approving
review on the head commit, green checks), you may squash-merge it yourself
instead of waiting for the PR triage task.
