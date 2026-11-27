#!/usr/bin/env bash
# Tests standards/scripts/coverage.sh without cargo-llvm-cov: badge colors
# and text, the repo summary, and that a project without Rust is skipped.
#   bash standards/tests/coverage.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
script=$root/standards/scripts/coverage.sh
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0

ok() { echo "ok   $1"; }
fail() { echo "FAIL $1"; failures=$((failures + 1)); }
expect() { # expect NAME HAYSTACK NEEDLE
  if [[ "$2" == *"$3"* ]]; then ok "$1"; else fail "$1: wanted '$3' in: $2"; fi
}

expect "80% is green" "$(bash "$script" badge coverage 80)" 'fill="#4c1"'
expect "badge shows the rounded percent" "$(bash "$script" badge coverage 79.6)" '>80%<'
expect "60% is yellow" "$(bash "$script" badge coverage 60)" 'fill="#dfb317"'
expect "59.9% is red" "$(bash "$script" badge coverage 59.9)" 'fill="#e05d44"'
expect "no data is grey" "$(bash "$script" badge coverage none)" '>no data<'
expect "badge is labelled" "$(bash "$script" badge coverage 50)" 'aria-label="coverage: 50%"'

echo '{"project":"b","lines":{"count":100,"covered":50,"percent":50}}' >"$work/b.json"
echo '{"project":"a/x","lines":{"count":300,"covered":270,"percent":90}}' >"$work/a.json"
summary=$(bash "$script" summarize "$work/a.json" "$work/b.json")
expect "total is covered lines over all lines" "$(jq -c .lines <<<"$summary")" '{"count":400,"covered":320,"percent":80}'
expect "projects are sorted" "$(jq -c '[.projects[].project]' <<<"$summary")" '["a/x","b"]'

mkdir -p "$work/norust"
out=$(cd "$work/norust" && git init -q && bash "$script" run 2>&1)
status=$?
if [ "$status" -eq 0 ]; then ok "a project without Cargo.toml passes"; else fail "a project without Cargo.toml exited $status"; fi
expect "and says it was skipped" "$out" "nothing to measure"

# store: commits to coverage-results on a throwaway origin, twice.
git init -q --bare "$work/origin.git"
git init -q "$work/repo"
git -C "$work/repo" remote add origin "$work/origin.git"
git -C "$work/repo" -c user.name=t -c user.email=t@t commit -q --allow-empty -m init
(cd "$work/repo" && bash "$script" store "$work/a.json" "$work/b.json" >/dev/null 2>&1)
(cd "$work/repo" && bash "$script" store "$work/b.json" >/dev/null 2>&1)
tree=$(git -C "$work/origin.git" ls-tree -r --name-only coverage-results 2>&1)
expect "store writes the total badge" "$tree" "badge.svg"
expect "store writes a badge per project" "$tree" "badges/a-x.svg"
expect "store keeps the latest result per project" "$tree" "projects/b.json"
expect "store writes the summary" "$(git -C "$work/origin.git" show coverage-results:summary.json 2>&1 | jq -c .lines)" '"percent":80'
expect "store appends history" "$(git -C "$work/origin.git" show "coverage-results:history/$(date -u +%Y-%m).jsonl" 2>&1 | wc -l | tr -d ' ')" "3"
expect "store never writes main" "$(git -C "$work/origin.git" branch --list)" "coverage-results"
if git -C "$work/origin.git" rev-parse -q --verify refs/heads/main >/dev/null || git -C "$work/origin.git" rev-parse -q --verify refs/heads/master >/dev/null; then
  fail "store pushed a branch other than coverage-results"
else
  ok "only coverage-results was pushed"
fi

out=$(bash "$script" nope 2>&1)
expect "unknown command prints usage" "$out" "usage: coverage.sh"

if [ "$failures" -gt 0 ]; then
  echo "coverage.test.sh: $failures failure(s)"
  exit 1
fi
echo "coverage.test.sh: all cases passed"
