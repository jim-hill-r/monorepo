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

Always open the PR as a **draft**, against `main`. The description says:

- which issue it addresses (quote the `TODO`/`FIX` line and its `ISSUES.md`),
- what changed and why,
- how it was tested (the hooks passed, or which commands you ran by hand and
  their result, and any check you couldn't run),
- anything a reviewer should look at closely.

## Mark it ready for review

Mark the PR ready for review once **all** of these hold:

- The change is complete for the issue; nothing is left as "follow-up" unless
  it's recorded as a `TODO (agent-generated)` in an `ISSUES.md`.
- CI for the changed projects passes on the PR's head commit.
- The `TODO`/`FIX` you fixed is removed and docs are updated.
- You've re-read the full diff and would be comfortable with a human reviewing it.

Ready for review hands the PR to a human; it is not approval. If a reviewer
requests changes, address them and re-check the list above. Move the PR back to
draft if it stops meeting it.

Never approve your own PR. Once it is mergeable as defined in
`standards/docs/pull-requests.md` (not a draft, no conflict, an approving
review on the head commit, green checks), you may squash-merge it yourself
instead of waiting for the PR triage task.
