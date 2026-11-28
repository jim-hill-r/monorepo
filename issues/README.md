# Issues

Every open task in this monorepo is one Markdown file in an `issues/`
folder: this one for repo-wide work, `<project>/issues/` for a project's
own, and [`human/`](human/) for tasks only a person can do. One file per
issue means parallel PRs add and close issues without touching the same
file, so they don't conflict. The format and rules are in
[standards/docs/issue-management.md](../standards/docs/issue-management.md).

To see the list, run `mise run //:issues` (it is generated, never
committed).

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
[standards/docs/plans/2026-11-deploy-feedback.md](../standards/docs/plans/2026-11-deploy-feedback.md).
They build on the mise workflows from PR #570 (`mise-projects.yml`,
`push-main.yml`, `pull-request-ci.yml`).

## Eval engine

See
[standards/docs/plans/2026-11-eval-engine.md](../standards/docs/plans/2026-11-eval-engine.md)
and decision 0003. Work the phases in order. Phases 0 and 1 depend on
nothing; later phases need the "Feedback on every change" issues. The pane
pages for it are in [pane/issues/](../pane/issues/).

## Free tiers

See [standards/docs/free-tiers.md](../standards/docs/free-tiers.md).

## Cost check

See [standards/docs/plans/2026-11-cost-check.md](../standards/docs/plans/2026-11-cost-check.md)
(approved by Jim, 2026-11-26). One group per phase.

## Threat check

See [standards/docs/plans/2026-11-threat-check.md](../standards/docs/plans/2026-11-threat-check.md)
(approved by Jim, 2026-11-26). One group per phase.

## User feedback

See [standards/docs/plans/2026-11-user-feedback.md](../standards/docs/plans/2026-11-user-feedback.md)
and decisions 0020 and 0021 (proposed, 2026-11-28): in-app help from each
project's user guide, support requests and feedback, built as libraries
each product runs on its own origin, with product data in tenant-scoped
D1 tables. One issue per phase, in this folder and in `sparkhill/` and
`bear_down/` `issues/`. Phase 1 replaces the eval engine's phase 6 model
issue.

## Design system

See [standards/docs/plans/2026-11-design-system.md](../standards/docs/plans/2026-11-design-system.md)
(approved by Jim, 2026-11-26). One group per phase.

## Infrastructure as code

See [standards/docs/decisions/0007-infrastructure-as-code.md](../standards/docs/decisions/0007-infrastructure-as-code.md)
(accepted by Jim, 2026-11-26). One group per phase; phase 1 needs the
Auth0 credentials human issue first. We fix bugs found in `terrars`
upstream (Jim, 2026-11-26).

## Resource cleanup

See [standards/docs/plans/2026-11-resource-cleanup.md](../standards/docs/plans/2026-11-resource-cleanup.md)
(approved by Jim, 2026-11-26; Part 6 and phase 5, onboarding every live
resource to IaC, added in PR #674). One group per phase.

## Review roles

See [decision 0019](../standards/docs/decisions/0019-review-roles.md): who
may approve what, as a matrix of domains and project tiers in
`standards/reviewers.toml`. Work the phases in order; phase 1 needs PR 904
(the `Human review` check) merged first.

## Project isolation

See [decision 0026](../standards/docs/decisions/0026-project-isolation.md)
and [standards/docs/project-isolation.md](../standards/docs/project-isolation.md).
One `write-isolation-guide.md` per project, in any order (products first);
`telemetry/docs/isolation.md` is the example. The last issue,
`project-isolation-strict-audit.md`, makes the check fail instead of warn.
