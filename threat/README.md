# threat

[![Eval coverage](https://raw.githubusercontent.com/jimhillr/monorepo/evals-results/badges/threat.svg)](https://github.com/jimhillr/monorepo/blob/evals-results/verdicts.json)

The threat check: it holds each project's threat model
(`<project>/docs/threat.toml`) to the security rules in
[standards/docs/security.md](../standards/docs/security.md), so a PR that
could weaken security is caught before it merges. The design and phases are
in the [threat check plan](../standards/docs/plans/2026-11-threat-check.md).

## Crates

| Crate | Job |
| --- | --- |
| `model` (`threat_model`) | Serde types for `docs/threat.toml`, the PR security summary (`summarize`) and the rules (THR-001 to THR-009; THR-004, drift through the shared detectors, comes in phase 3) as pure functions. No I/O, so pane, the eval engine and the deep review can use the same rules. |
| `cli` (`threat_cli`) | The `threat` binary (`check`, and `summary` for PR CI's security summary). It reads each project's file, the setting ids in `standards/settings.toml`, the workflows in `.github/workflows/` and each surface's `headers` file. |

## Commands

```bash
# Check every project's docs/threat.toml. Warnings, exit 0.
cargo run -p threat_cli -- check
# Only some projects.
cargo run -p threat_cli -- check --projects cookbook,pane
# Exit 1 on any finding except THR-009 (phase 4).
cargo run -p threat_cli -- check --strict
# The PR security summary against a base branch, and the labels to set.
cargo run -p threat_cli -- summary --base origin/main --labels labels.txt
```

The audit runs it through `standards/scripts/threat-check.sh`. It reads
only files in the repo, needs no credentials or network, and takes well
under a second once built.
