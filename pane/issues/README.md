# Notes on pane's issues

## Simple dashboard

Pane becomes one simple dashboard: a list of every action waiting on Jim
and one scorecard per domain, with drill-down into projects (Jim,
2026-11-28). The why and the order are in
[docs/roadmap.md](../docs/roadmap.md). Phase 1, the Repo health summary
read from `standards/repo-health.toml`, is done. Work the phases in order;
it reuses the Review dashboard epic's signals rather than adding pages.

## Review dashboard epic

Pane becomes one place for Jim to review the monorepo: deliverables and
downloads, executive reviews, human issues, a traffic-light health dashboard
with drill-down, and adding TODOs. The plan, with the architecture, health
rules and costs, is
[docs/plans/2026-11-review-dashboard.md](../docs/plans/2026-11-review-dashboard.md).
Work the phases in order; within a phase, by `order`.

### Phase 2: executive reviews

PR #585 (docs/reviews and `review-decks.sh`) is merged, so this phase can start.

### Phase 6: observability

Needs the `telemetry` Worker and Web Analytics from the "Feedback on every
change" issues in the root [`issues/`](../../issues/).

### Eval engine: requirements and feedback

Needs Phases 3 to 5 above and the eval engine's `evals-results` files (root
`issues/`, eval engine phase 1). The plan is
[standards/docs/plans/2026-11-eval-engine.md](../../standards/docs/plans/2026-11-eval-engine.md).

### Phase 7: revenue and cost

Needs Phase 1 for the pages, Phase 3 for free-tier usage and Phase 5 for the
forms. See "Revenue and cost" in the plan.
