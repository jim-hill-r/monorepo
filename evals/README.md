# evals

The eval engine: it checks that each project's product requirements hold
as built, at every stage from CI to production. The name is plain: an eval
is one check of one requirement.

Requirements live in `<project>/docs/requirements.md` and their evals in
`<project>/evals.toml`; the formats are in
[standards/docs/evals.md](../standards/docs/evals.md). The design and the
phases are in the
[eval engine plan](../standards/docs/plans/2026-11-eval-engine.md) and
[decision 0003](../standards/docs/decisions/0003-eval-engine.md).

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
# Check every project's requirements.md and evals.toml (EVL-001 to EVL-004).
cargo run -p evals_cli -- check            # warnings, exit 0
cargo run -p evals_cli -- check --strict   # exit 1 on any finding

# Turn saved test reports into one result per eval (cargo and playwright
# kinds), appended to evals-results.jsonl. Runs no tests itself.
cargo test 2>&1 | tee cargo-test.log
cargo run -p evals_cli -- run --stage ci --project cookbook \
  --cargo-output cargo-test.log --playwright-report web/results.json

# Write the job summary table ($GITHUB_STEP_SUMMARY, or stdout).
cargo run -p evals_cli -- publish
```

Each result says `pass`, `fail`, `error` (an eval matched no test) or
`skip` (no report of its kind, or every matching test was skipped), with a
short note. In GitHub Actions the commit, run id and run link come from the
`GITHUB_*` variables.

Wiring `run` and `publish` into the shared tasks and workflows, and
committing results to the `evals-results` branch, come in phase 1 (see the
root `ISSUES.md`). Results are files, never a database.
