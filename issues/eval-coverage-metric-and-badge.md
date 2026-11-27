---
kind: todo
priority: priority
tags: [agent-generated]
epic: Eval engine
order: 15
added: 2026-11-27
---

Eval coverage metric and badge (Jim, 2026-11-27: "Maybe we introduce an eval coverage metric / badge"). Add an eval coverage figure per project to `evals verdicts` and `verdicts.json`: the share of the project's non-retired requirements whose verdict is green on production evidence, with `must` requirements shown separately. Have `evals store` also write `badge.svg` (repo total) and `badges/<project>.svg` on the `evals-results` branch, the same way `nightly-coverage.yml` writes the code coverage badge on `coverage-results`. Add the total badge to the "Repo health" table in the root README ([repo-health.md](../standards/docs/repo-health.md)) and each project's badge to the top of its README. Projects with no requirements show as 0%. Decide the warning level with Jim in the PR (suggest under 80% of musts green is a warning, like COV-001). Background: the [executive review](../docs/reviews/2026-11-27-evals/review.md).
