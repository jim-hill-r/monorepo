# controlled_chaos issues

One file per issue; see [issue-management.md](../../../standards/docs/issue-management.md).

# Epic notes

## Feedback on every change

See
[standards/docs/plans/2026-11-deploy-feedback.md](../../../standards/docs/plans/2026-11-deploy-feedback.md).
They build on the mise workflows from PR #570 (`mise-projects.yml`,
`push-main.yml`, `pull-request-ci.yml`).

## Threat check

See [standards/docs/plans/2026-11-threat-check.md](../../../standards/docs/plans/2026-11-threat-check.md)
(approved by Jim, 2026-11-26). One group per phase.

## Rulebook mechanics

The game plays a small part of [rulebook.md](../rulebook.md): value
battles, life, happiness and deck exhaustion, written down in
[official_rules.md](../official_rules.md). These issues build the rest,
listed there under "Not built yet". The rulebook is a brainstorm with gaps
(no profession effects, blank action cards, crises with no stated way to
avert them), so each phase first proposes its rules in official_rules.md in
a PR labelled `needs-human` for Jim, and builds them only once he agrees.
Rules live in the `RulesEngine` (`src/rules.rs`), each with a CHAOS-002
eval in `evals.toml`. The phases don't depend on each other; work them in
order.
