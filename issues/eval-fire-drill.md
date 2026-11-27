---
kind: todo
priority: priority
tags: [agent-generated]
epic: Eval engine
order: 100
added: 2026-11-27
---

Run one eval fire drill. No eval result has ever failed (1,780 results to 2026-11-27), so the loop has never been seen working. On one project, push a change on a branch that breaks one `ci` eval and confirm the PR check fails; then point one `cd` `http` eval at a missing string via a manual Push Main CI/CD run with a test ref (or a dry run of `evals run --stage cd` against the live site) and confirm the result is stored as `fail`, the verdict turns red in `verdicts.json`, triage files a `FIX`, and the next good run turns it green. Write what happened as a short retro in `evals/docs/` and fix anything that didn't work. Needs the triage workflow (eval-triage-free-workflow.md) first.
