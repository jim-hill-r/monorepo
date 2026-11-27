---
added: 2026-11-26
runbook: standards/docs/runbooks/github-ruleset-required-check.md
---

## Requirement

In GitHub, Settings > Rules > Rulesets > "Default" (the ruleset on `main`), turn on "Require status checks to pass" and add the `CI result` check (GitHub Actions).

Makes auto-merge wait for green CI after any push; until then agents only turn auto-merge on while checks are green, see the Auto-merge section of standards/docs/pull-requests.md.

## Why it can't be automated

Rulesets need Admin, and agents must never be able to change the rules that hold them back (decision 0010).

## Runbook

[Require CI on the main ruleset](../../standards/docs/runbooks/github-ruleset-required-check.md)
