# Issues

Every open task in this monorepo is one Markdown file in the
`docs/issues/` folder of the project it belongs to
(`<project>/docs/issues/`), and tasks only a person can do are in that
project's `docs/issues/human/`. This folder is the standards project's:
it holds repo-wide work that no other platform project owns, and the
priority and on-hold project lists below, which `chores next-task` and pane
read. There is no root `issues/` folder. One file per issue means parallel
PRs add and close issues without touching the same file, so they don't
conflict. The format and rules are in
[standards/docs/issue-management.md](../issue-management.md).

To see the list, run `mise run //:issues` (it is generated, never
committed); `-- --human` lists every project's human tasks.

# Priority Projects
- sparkhill (Blue Eel first: Jim, 2026-11-27)
- cookbook
- homer
- controlled_chaos
- royalty
- pane

# On Hold Projects
- luggage
- starcraft
- profiles

# Epic notes

## Feedback on every change

See
[standards/docs/plans/2026-11-deploy-feedback.md](../plans/2026-11-deploy-feedback.md).
They build on the mise workflows from PR #570 (`mise-projects.yml`,
`push-main.yml`, `pull-request-ci.yml`).

## Free tiers

See [standards/docs/free-tiers.md](../free-tiers.md).

## User feedback

See [standards/docs/plans/2026-11-user-feedback.md](../plans/2026-11-user-feedback.md)
and decisions 0020 and 0021 (accepted, 2026-11-28): in-app help from each
project's user guide, support requests and feedback, built as libraries
each product runs on its own origin, with product data in tenant-scoped
D1 tables. One issue per phase, in this folder and in `sparkhill/` and
`bear_down/` `docs/issues/`. Phase 1 replaces the eval engine's phase 6 model
issue.

## Resource cleanup

See [standards/docs/plans/2026-11-resource-cleanup.md](../plans/2026-11-resource-cleanup.md)
(approved by Jim, 2026-11-26; Part 6 and phase 5, onboarding every live
resource to IaC, added in PR #674). One group per phase.

## Review roles

See [decision 0019](../decisions/0019-review-roles.md): who
may approve what, as a matrix of domains and project tiers in
`standards/reviewers.toml`. Work the phases in order; phase 1 needs PR 904
(the `Human review` check) merged first.
