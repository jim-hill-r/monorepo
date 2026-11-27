#!/usr/bin/env bash
# COST-*: check every project's docs/cost.toml against the budgets with
# `cost check` (the cost project; the convention is in standards/docs/cost.md).
# Warnings only until phase 3 of the cost check plan, so this always exits 0.
#
#   cost-check.sh [check] [--projects a,b]
#       Run the per-project rules (for the listed projects, or every one) and
#       the account-wide rules. Called by `mise run //:audit`. Without
#       --projects, COST_PROJECTS (comma separated, set by hooks.sh and PR
#       CI to the affected projects) picks them when it is set, even empty.
#   cost-check.sh render-check
#       Fail when the limit tables in standards/docs/free-tiers.md don't
#       match standards/radar/limits.toml. Called by standards CI.
#
# The binary is built once into cost/target, so later runs take well under a
# second. Without cargo it prints a warning and exits 0; PR CI checks it.
# Run from anywhere in the repo.
set -uo pipefail

cd "$(git rev-parse --show-toplevel)"

mode=check
case "${1:-}" in
  check) shift ;;
  render-check) mode=render-check; shift ;;
esac

if ! command -v cargo >/dev/null 2>&1; then
  echo "warning: cost check skipped: cargo is not installed"
  exit 0
fi

cost=(cargo run --quiet --manifest-path cost/Cargo.toml -p cost_cli --)

if [ "$mode" = render-check ]; then
  "${cost[@]}" render --root . --check
  status=$?
  # 1 means out of date; anything else means the tool could not run.
  if [ "$status" -eq 1 ]; then exit 1; fi
  [ "$status" -eq 0 ] || echo "warning: cost render could not run; see the output above"
  exit 0
fi

if [ $# -eq 0 ] && [ -n "${COST_PROJECTS+set}" ]; then
  set -- --projects "$COST_PROJECTS"
fi

if ! "${cost[@]}" check --root . "$@"; then
  echo "warning: cost check could not run; see the output above"
fi
exit 0
