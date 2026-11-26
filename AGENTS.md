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
- **No project templates.** The old `example/` templates are intentionally
  gone and must not be restored. Create new projects by hand.
- **PR review automation.** An hourly scheduled agent task, "Hourly PR
  review", reviews every open PR not opened by an agent account (any GitHub
  login ending in `-agt`, such as `jimhillr-agt`). That covers humans,
  dependabot and other bots. It reviews as `jimhillr-agt` and approves PRs it
  judges safe. It only approves: it never merges, comments or requests changes.
  It requires green checks and no conflicts, approves only patch or minor
  dependency bumps (a 0.x minor bump counts as major), never approves the
  pinned bumps above, and never approves changes to workflows, auth code or
  secrets unless they're trivially safe. It reads this file for extra pins, so
  list any new pins here.
- **Starting new work.** There is no scheduled job for new work. Whenever no
  agent PRs or agent tasks are in flight, the coordinating agent starts a new
  task by running the `start-a-new-task` skill.
- **CI on `main` may be red.** Judge a PR by whether the projects it changes
  pass, not by the overall status.

## Dev commands

Each project's build, test and CI steps run through the repo's task runner.
Run the CI task for every project you changed and make it pass before
opening or readying a PR.

- Install project tooling: `cast install` (in the project directory)
- Run CI for a project (fmt, clippy, build, tests, and npm lint/test where a
  `package.json` exists): `cast ci`
- Run only tests: `cast test`
- Explore other commands: `cast -h`

Skills refer to this section instead of naming commands directly, so when the
task runner changes, only this section needs updating.

The repo is moving from Cast to [mise](https://mise.jdx.dev). Each project
already has a `mise.toml`, and `mise run ci` runs the same checks as `cast ci`
in a non-blocking CI job. The rules and decisions for that move (no root Rust
workspace, the same tasks in every project, no committed build artifacts) are in
[standards/docs/mise.md](standards/docs/mise.md). This section switches to mise
once Cast is removed.

## Skills

| Skill | Use it to |
| --- | --- |
| `start-a-new-task` | Pick one TODO from the `ISSUES.md` files and complete it end to end. |
| `code-review` | Review one project against the standards and record five TODOs. |
| `open-pull-request` | Branch, commit, open a draft PR, and mark it ready when it is. |

New skills go in `.agents/skills/<name>/SKILL.md` and must follow the
[Agent Skills specification](https://agentskills.io/specification). The
`Agent Skills` workflow validates them on every pull request.
