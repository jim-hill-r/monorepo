# Eval fire drill, 2026-11-27

Until today none of the 1,780 stored eval results had ever failed, so nobody
had seen the eval loop catch anything. This drill broke cookbook on purpose,
in both stages, and followed each break through results, verdicts and triage
to green again. Production and the `evals-results` branch were never
changed: the broken content stayed local and was reverted, and every result
went into a temporary copy of `evals-results`.

The drill is now repeatable: the **evals fire drill** workflow
(`.github/workflows/evals-fire-drill.yml`, run by hand, which runs
`evals drill`) does the production half against the live site (see "Fire drill" in
[standards/docs/evals.md](../../standards/docs/evals.md#fire-drill)).

## What we did and what happened

| Step | Expected | Happened |
| --- | --- | --- |
| CI break: week 48's first recipe uses 3 tablespoons of olive oil instead of 2 | `test_get_week_shopping_list_adds_up` fails, its evals fail | 63 passed, 1 failed; `shopping-list-built` (COOK-002) and `shopping-list-sums` (COOK-003) both `fail` |
| Verdicts after the CI break | COOK-002 and COOK-003 amber | COOK-002 `at-risk`. **COOK-003 stayed `unknown`** (fixed, see below) |
| CI break revert | green again | 64 passed, COOK-002 back to `met` |
| CI break: remove recipe 200's `day-200` tag | `day-files-exist` (COOK-001) fails | The build script stops the build first. **No eval result at all** with `--partial`, `error` without it (issue filed) |
| CD break: `recipe-page-live` looks for a string that isn't on the page | result `fail` | `fail`: "body doesn't contain ..." |
| Verdict after the CD failure | COOK-001 red | `not-met` |
| Triage | a `kind: fix` issue for COOK-001 | Not live on the day. `evals-triage.yml` landed later the same day, and the drill now checks a dry run of it plans the issue |
| Next good CD run | COOK-001 green | `met` (one failure in the last 7 runs is under the flaky threshold of 2) |

The CD half ran from a cloud sandbox whose proxy blocks `*.pages.dev`, so it
ran there against a local stand-in page; the workflow ran the same steps
against the live site from GitHub Actions and passed (COOK-001 met, not met,
met).

## What we found

1. **A failing `ci` eval hid behind "unknown".** COOK-003 has no
   production-stage eval, and a requirement with no production results was
   `unknown` before any other rule ran, so its failing test never showed.
   Fixed in this PR: it is now `at-risk` with "is failing in ci" (test
   `ci_failure_without_production_results_is_at_risk_not_unknown` in
   `evals/model/src/verdict.rs`).
2. **A build failure leaves no eval result.** The PR check fails, so nothing
   ships, but the verdicts can't see it.
   [evals/issues/build-failure-leaves-no-result.md](../issues/build-failure-leaves-no-result.md).
3. **Triage was the missing link.** Red verdicts reached `verdicts.json`,
   but nothing filed the `FIX`. The daily `evals-triage.yml` now does, and
   `evals drill` checks that a dry run of triage picks the requirement up.
4. **An unreachable site is a `fail`, not an `error`.** From the sandbox the
   proxy refused the request and both page evals recorded `fail`. That is
   right for production (users can't reach it either), but a runner network
   problem would turn requirements red. Watch for it in triage rather than
   change it.
5. **COOK-005 (accessibility, a must) is always `unknown`.** Its `cd`
   Playwright eval never runs, because no smoke step runs after deploy yet
   (`issues/nightly-uat-smoke-step-after-deploy.md`), yet the coverage line
   says "3 of 3 musts evaluated in production". The coverage metric issue
   should count musts with production results, not musts with evals.

## Next drill

Run the workflow after any change to the verdict rules, `evals store` or
the triage workflow.
