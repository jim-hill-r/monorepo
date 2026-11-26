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
```

`evals run` and `evals publish` come next (see the phase 0 TODOs in the
root `ISSUES.md`). Results are files, never a database: JSONL on the
`evals-results` branch.
