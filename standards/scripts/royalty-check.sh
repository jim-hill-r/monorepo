#!/usr/bin/env bash
# ROY-001 and RTOML-*: every project's royalty.toml, checked with
# `royalty check` (the royalty project; the format is in
# royalty/docs/royalty-toml.md and the rule in standards/docs/configuration.md).
#
#   ROY-001  A project (a directory with a mise.toml, not the repo root) has
#            no royalty.toml.
#   RTOML-*  A finding `royalty check` reports for a project's royalty.toml.
#
# Errors: every project has a valid royalty.toml, so any ROY-001 or
# RTOML-* error fails the script (exit 1). Warnings `royalty check` reports
# are printed and don't fail it. Called by `mise run //:audit`.
# The binary is built once into royalty/target, so later runs are quick.
# Without cargo it prints a warning and exits 0; PR CI checks it.
# Run from anywhere in the repo.
set -uo pipefail

cd "$(git rev-parse --show-toplevel)"

findings=0
errors=0
fail() { echo "error[$1] $2"; findings=$((findings + 1)); errors=$((errors + 1)); }

projects=$(git ls-files -- '*/mise.toml' | xargs -r -n1 dirname | sort -u)

files=()
for d in $projects; do
  if [ -f "$d/royalty.toml" ]; then
    files+=("$d/royalty.toml")
  else
    fail ROY-001 "$d has no royalty.toml; add one with a [project] name and description (standards/docs/configuration.md)"
  fi
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "warning: royalty check skipped: cargo is not installed"
  exit 0
fi
if ! cargo build --quiet --manifest-path royalty/Cargo.toml -p royalty_cli 2>&1; then
  echo "warning: royalty check could not build; see the output above"
  exit 0
fi
royalty=royalty/target/debug/royalty

for f in "${files[@]}"; do
  # Only finding lines; "royalty check: ..." summary lines are dropped.
  while IFS= read -r line; do
    case "$line" in
      error\[*) echo "$line"; findings=$((findings + 1)); errors=$((errors + 1)) ;;
      warning\[*) echo "$line"; findings=$((findings + 1)) ;;
    esac
  done < <("$royalty" check "$f" 2>&1)
done

if [ "$findings" -eq 0 ]; then
  echo "royalty check: ok (${#files[@]} royalty.toml files)"
else
  echo "royalty check: $findings finding(s), $errors error(s), in ${#files[@]} royalty.toml files"
fi
[ "$errors" -eq 0 ]
