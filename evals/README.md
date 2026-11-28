# evals

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/evals.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

The eval engine: it checks that each project's product requirements hold
as built, at every stage from CI to production. The name is plain: an eval
is one check of one requirement.

Requirements live in `<project>/docs/requirements.md` and their evals in
`<project>/evals.toml`; the formats are in
[standards/docs/evals.md](../standards/docs/evals.md). The design and the
phases are in the
[eval engine plan](../standards/docs/plans/2026-11-eval-engine.md) and
[decision 0003](../standards/docs/decisions/0003-eval-engine.md).

What comes next, and where every part of the eval process lives, is in
the [evals roadmap](docs/roadmap.md).

## Crates

| Crate | Job |
| --- | --- |
| `model` (`evals_model`) | Serde types (`Requirement`, `Eval`, `EvalResult`, `Verdict`), the parsers for `requirements.md` and `evals.toml`, and the verdict rules. No I/O, so pane can use the same rules. |
| `cli` (`evals_cli`) | The `evals` binary. |

The verdict thresholds (how near a metric limit is "at risk", how many
failures make an eval flaky, how fresh a production result must be) are in
[verdicts.toml](verdicts.toml).

## Commands

```bash
# Check every project's requirements.md and evals.toml (EVL-001 to EVL-006;
# EVL-005 checks that every Playwright config tags its user agent).
cargo run -p evals_cli -- check            # warnings, exit 0
cargo run -p evals_cli -- check --strict   # exit 1 on any finding

# Turn saved test reports into one result per eval (cargo, trycmd and
# playwright kinds; trycmd's case lines are in the cargo output), appended
# to evals-results.jsonl. Runs no tests itself.
cargo test 2>&1 | tee cargo-test.log
cargo run -p evals_cli -- run --stage ci --project cookbook \
  --cargo-output cargo-test.log --playwright-report web/results.json

# Stage cd, after a deploy: make the requests of the http evals against the
# live site (path joined to --deploy-url, default $DEPLOY_URL), and exit 1
# if an eval of a must requirement failed.
cargo run -p evals_cli -- run --stage cd --project cookbook \
  --deploy-url https://jimhillr-cookbook.pages.dev --fail-on-must

# Write the job summary table ($GITHUB_STEP_SUMMARY, or stdout).
cargo run -p evals_cli -- publish

# Settings made by hand (standards/settings.toml): list the checks that are
# due (--write adds their issues/human/settings-check-<id>.md tasks), and record
# Jim's answer (which deletes the task).
cargo run -p evals_cli -- human-checks due --write
cargo run -p evals_cli -- human-checks answer claude-connectors --ok
cargo run -p evals_cli -- human-checks answer claude-connectors --not-ok "GitHub needs reconnecting"

# Add results to a checkout of the evals-results branch and rewrite its
# verdicts.json (standards/scripts/evals.sh store does the git part).
cargo run -p evals_cli -- store --results evals-results.jsonl --dir ../evals-results

# File the work the verdicts call for: a FIX or TODO issue file for each
# requirement that is not green and has no open issue (--write writes them,
# at most --limit). standards/scripts/evals.sh triage runs it daily.
cargo run -p evals_cli -- triage --dir ../evals-results --write

# The fire drill: break one http eval against the live site, check its
# requirement turns red and triage would file it, then back to green.
# Reads the checkout, writes nothing (evals-fire-drill.yml runs it).
cargo run -p evals_cli -- drill --dir ../evals-results --project cookbook --eval recipe-page-live

# Every requirement's current verdict, from a checkout of that branch
# (standards/scripts/evals.sh verdicts fetches it for you).
cargo run -p evals_cli -- verdicts --dir ../evals-results --project cookbook
```

Each result says `pass`, `fail`, `error` (an eval matched no test, or an
`http` eval has a `path` and no deploy URL) or
`skip` (no report of its kind, or every matching test was skipped), with a
short note. In GitHub Actions the commit, run id and run link come from the
`GITHUB_*` variables.

`--partial` is for a run whose reports cover only part of a project (one
sub-project's CI job, such as `cookbook/web`): an eval with no matching test
then gets no result instead of `skip` or `error`.

In CI the shared `ci` and `evals` tasks call these for you through
`standards/scripts/evals.sh`; see "In CI" in
[standards/docs/evals.md](../standards/docs/evals.md#in-ci). Pushes to
`main` and nightly CI commit the results to the long-lived `evals-results`
branch; see "Results and verdicts" there. Results are files, never a
database.
