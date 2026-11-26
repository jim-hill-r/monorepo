#!/usr/bin/env bash
# EVL-001 to EVL-004: check each project's docs/requirements.md and evals.toml
# with `evals check` (the evals project; formats in standards/docs/evals.md).
# Warnings only until Phase 4 of the eval engine plan, so this always exits 0.
#
# Builds the evals CLI only when some project has one of the files, so the
# audit costs nothing until then. Run from the repo root:
#   bash standards/scripts/evals-check.sh
set -uo pipefail

cd "$(git rev-parse --show-toplevel)"

if ! git ls-files -- 'evals.toml' '*/evals.toml' '*/docs/requirements.md' | grep -q .; then
  exit 0
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "warning: evals check skipped: cargo is not installed"
  exit 0
fi

if ! cargo run --quiet --manifest-path evals/Cargo.toml -p evals_cli -- check --root .; then
  echo "warning: evals check could not run; see the output above"
fi
exit 0
