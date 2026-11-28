# Notes on pane's issues

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

## Deck sign-off

Reviewers sign off on review decks in pane, and each feedback item becomes
an issue file. The plan is
[docs/plans/2026-11-deck-sign-off.md](../docs/plans/2026-11-deck-sign-off.md).
Phase 1 (the format and display) is built. Phase 2 is the review dashboard
epic's Cloudflare Access issue (order 110). Phase 4 needs the pane Worker and
the `/api/edits` write path from that epic.

## Review feedback

Issue files a reviewer filed by signing off on a review deck
(`review-<deck folder>-<reviewer>-<n>.md`). Each links its deck and the
sign-off. Work them like any other issue; deleting the file marks the item
done on the deck's Review page.

