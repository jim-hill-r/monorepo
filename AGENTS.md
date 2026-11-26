# Agent instructions

Repo-wide rules for any AI coding agent working in this monorepo. Task-specific
procedures live as [Agent Skills](https://agentskills.io) in `.agents/skills/`
(`.claude/skills` is a symlink to the same folder).

## Ground rules

- **No local clone required.** The repo is meant to be fully automated. Do
  maintenance through a GitHub workflow in `.github/workflows/` or an agent skill
  in `.agents/skills/`, never through a script someone has to run locally.
- **Standards are the source of truth.** Coding, testing, documentation, naming
  and workflow rules live in `standards/docs/`. If this file disagrees with
  them, the standards win; fix this file.
- **Work is tracked in `ISSUES.md` files**, not GitHub issues. See
  `standards/docs/issue-management.md`.
- **Pull requests always open as drafts.** Mark a PR ready for review only once
  you judge it ready for human review. The `open-pull-request` skill has the
  full checklist. Don't hand a PR to a human (or present it as awaiting their
  review) until its checks are green and it is out of draft.
- **Plans, decisions and retros live in the repo**, not in chat tools or
  hosted docs. Write them as Markdown next to what they are about (a project's
  own folder, or `standards/docs/` for repo-wide work) and commit them with the
  related PR.

## Project rules and notes

This file is the home for project rules and memory. Keep them here, in the
repo, rather than in any one agent tool's settings.

- **Branch cleanup** is done with the `all-delete-branches.yml` workflow (dry
  run by default), not by deleting branches by hand or with a local script.
- **Pinned dependencies — do not bump:**
  - `dioxus` in `cahokia` is deliberately pinned to an older version.
  - `sycamore` in `sparkhill/website` and `profiles/jimhillr` must stay on
    0.8, which `perseus` 0.4.2 requires.
- **Vendors (Jim, 2026-11-26).** When designing a system, use as few vendors
  as possible, but cost always comes first: if a non-approved vendor is
  clearly cheaper (including free tiers), cost wins over vendor count.
  - **Approved vendors:** GitHub (including Actions and Dependabot),
    Cloudflare and Squarespace. Reach for these first.
  - **Any other vendor needs Jim's sign-off.** Propose it in a decision record
    in `standards/docs/decisions/` (or the PR description for a small
    change), saying what it costs, why no approved vendor fits, and what a
    switch would take. Label the PR `needs-jim` and don't merge it until Jim
    approves.
  - **Approved exceptions:**
    - PostHog, for real-user session replay (decision 0001).
    - Auth0, used by `auth_sdk` and `cahokia` for sign-in.
  - **Pending Jim's confirmation:** RapidAPI (Zillow data), used by `homer`.
- **No project templates.** The old `example/` templates are intentionally
  gone and must not be restored. Create new projects by hand.
- **Merge rule.** Any agent may squash-merge any PR, including one its own
  thread opened, once it is mergeable: not a draft, no merge conflict, at
  least one approving review on the current head commit and no outstanding
  "changes requested", and green checks. Green means every check has finished
  and passed, except a check that is also failing on the latest `main` commit
  for reasons the PR didn't cause. A pending check is never green. Agents
  still never approve their own PRs. Details are in
  `standards/docs/pull-requests.md`.
- **PR lifecycle.** Every PR, from any author, is driven to closure (merged
  or closed). The policy is in `standards/docs/pull-requests.md`. An hourly
  scheduled agent task, "Hourly PR triage", acts as `jimhillr-agt` on every
  open PR: it approves safe dependabot and human PRs, squash-merges every
  mergeable PR (per the merge rule above), asks dependabot to rebase
  conflicts, closes superseded or pinned bumps, adopts orphaned agent PRs, and
  labels anything that needs Jim `needs-jim`. Safe to approve means green
  checks, no conflict, patch or minor bumps only (a 0.x minor bump counts as
  major), never the pinned bumps above, and no workflow, auth or secrets
  changes unless trivially safe. It reads this file for extra pins, so list
  any new pins here.
- **Starting new work.** There is no scheduled job for new work. Whenever no
  agent PRs or agent tasks are in flight, the coordinating agent starts a new
  task by running the `start-a-new-task` skill.
- **CI on `main` may be red.** Judge a PR by whether the projects it changes
  pass, not by the overall status.

## Dev commands

Each project's build, test and CI steps run through the repo's task runner.
Run the CI task for every project you changed and make it pass before
opening or readying a PR.

- Install project tooling: `mise install` (in the project directory)
- Run CI for a project (fmt, clippy, build, tests, and npm tests where a
  `package.json` exists): `mise run ci`
- Run only tests: `mise run test`
- Fix formatting and lints: `mise run fix`
- List the project's tasks: `mise tasks`
- Check the repo standards: `mise run //:audit`

Skills refer to this section instead of naming commands directly, so when the
task runner changes, only this section needs updating.

Every project has the same tasks (`ci`, `fix`, `build`, `test`, `dev`,
`release`, `deploy`), from one shared file in `standards/tasks/`. The rules
and decisions (no root Rust workspace, no committed build artifacts) are in
[standards/docs/mise.md](standards/docs/mise.md). CI runs `mise run ci` in
each changed project; pushes to `main` also run `mise run deploy`. The old
Cast CLI and `Cast.toml` files are still in the tree until they are removed,
but nothing runs them.

## Skills

| Skill | Use it to |
| --- | --- |
| `start-a-new-task` | Pick one TODO from the `ISSUES.md` files and complete it end to end. |
| `code-review` | Review one project against the standards and record five TODOs. |
| `open-pull-request` | Branch, commit, open a draft PR, and mark it ready when it is. |

New skills go in `.agents/skills/<name>/SKILL.md` and must follow the
[Agent Skills specification](https://agentskills.io/specification). The
`Agent Skills` workflow validates them on every pull request.
