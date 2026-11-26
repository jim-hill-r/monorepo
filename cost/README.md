# cost

The cost check: it holds each project's cost analysis
(`<project>/docs/cost.toml`) to the budgets in the tech radar and the free
limits in `standards/radar/limits.toml`, so a PR that could change what the
repo costs is caught before it merges. The format and rules are in
[standards/docs/cost.md](../standards/docs/cost.md); the design and phases
are in the [cost check plan](../standards/docs/plans/2026-11-cost-check.md).

## Crates

| Crate | Job |
| --- | --- |
| `model` (`cost_model`) | Serde types for `docs/cost.toml`, `limits.toml`, `detectors.toml` and the radar's vendor blips; the rules (COST-001 to COST-009), the detectors and the PR summary as pure functions. No I/O, so pane and the eval engine can use the same rules and money maths. |
| `cli` (`cost_cli`) | The `cost` binary. It reads files through git (tracked files only), and at a commit through one `git cat-file --batch`. |

## Commands

```bash
# Check every project's docs/cost.toml. Warnings, exit 0.
cargo run -p cost_cli -- check
# Only report per-project findings for some projects; account-wide totals
# (COST-005, COST-006) still read every project.
cargo run -p cost_cli -- check --projects cookbook,pane
# Exit 1 on any finding except COST-008 and COST-009 (phase 3).
cargo run -p cost_cli -- check --strict

# The PR cost summary against a base branch, as Markdown; --json also
# writes the labels (`cost`, `needs-human`) and reasons for PR CI.
cargo run -p cost_cli -- summary --base origin/main --json cost.json

# Rewrite the limit tables in standards/docs/free-tiers.md from limits.toml,
# or (--check) fail when they are out of date.
cargo run -p cost_cli -- render
cargo run -p cost_cli -- render --check
```

The audit runs it through `standards/scripts/cost-check.sh`. It reads only
files in the repo, needs no credentials or network, and takes well under a
second once built.
