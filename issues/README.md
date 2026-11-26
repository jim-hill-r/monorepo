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
- cookbook
- homer
- controlled_chaos
- royalty
- sparkhill
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

## Design system

See [standards/docs/plans/2026-11-design-system.md](../standards/docs/plans/2026-11-design-system.md)
(approved by Jim, 2026-11-26). One group per phase.

## Resource cleanup

See [standards/docs/plans/2026-11-resource-cleanup.md](../standards/docs/plans/2026-11-resource-cleanup.md)
(approved by Jim, 2026-11-26; Part 6 and phase 5, onboarding every live
resource to IaC, added in PR #674). One group per phase.
