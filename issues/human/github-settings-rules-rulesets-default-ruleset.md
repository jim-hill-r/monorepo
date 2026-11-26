---
added: 2026-11-26
---

In GitHub, Settings > Rules > Rulesets > "Default" (the ruleset on `main`), turn on "Require status checks to pass" and add the `CI result` check (GitHub Actions).

Makes auto-merge wait for green CI after any push; until then agents only turn auto-merge on while checks are green, see the Auto-merge section of standards/docs/pull-requests.md.
