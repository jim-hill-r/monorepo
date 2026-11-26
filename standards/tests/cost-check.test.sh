#!/usr/bin/env bash
# Tests standards/scripts/cost-check.sh: it runs `cost check` (passing
# --projects through), never fails the audit, fails render-check only when
# the free-tier tables are out of date, and skips without cargo. cargo is a
# stub. Run from anywhere:
#   bash standards/tests/cost-check.test.sh
set -uo pipefail

# Git hooks export these; they would point the scratch repo at this one.
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE GIT_PREFIX

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/repo/standards/scripts" "$work/stubs" "$work/empty"
cp "$root/standards/scripts/cost-check.sh" "$work/repo/standards/scripts/"
git -C "$work/repo" init -q
# CARGO_STATUS sets the stub's exit status.
cat >"$work/stubs/cargo" <<SH
#!/bin/sh
echo "cargo \$*" >>"$work/calls"
echo "cost: stub output"
exit \${CARGO_STATUS:-0}
SH
chmod +x "$work/stubs/cargo"
for tool in git; do ln -s "$(command -v $tool)" "$work/empty/$tool"; done
bash_bin=$(command -v bash)

failures=0

# run_case NAME PATH WANT_STATUS [VAR=value ...] -- [ARGS...]
run_case() {
  local name=$1 path=$2 want=$3
  shift 3
  local vars=()
  while [ $# -gt 0 ] && [ "$1" != "--" ]; do vars+=("$1"); shift; done
  [ "${1:-}" = "--" ] && shift
  : >"$work/calls"
  out=$(cd "$work/repo" && env -u CARGO_STATUS "${vars[@]}" PATH="$path" "$bash_bin" standards/scripts/cost-check.sh "$@" 2>&1)
  status=$?
  calls=$(cat "$work/calls")
  if [ "$status" -ne "$want" ]; then
    echo "FAIL $name: exit $status, want $want"; echo "$out"; failures=$((failures + 1))
  fi
}

expect() { # expect NAME DESCRIPTION TEST...
  local name=$1 what=$2
  shift 2
  if ! "$@"; then
    echo "FAIL $name: $what"; echo "output: $out"; echo "calls: $calls"
    failures=$((failures + 1))
  fi
}

prefix="cargo run --quiet --manifest-path cost/Cargo.toml -p cost_cli --"

run_case "check" "$work/stubs:$work/empty" 0
expect "check" "runs cost check" test "$calls" = "$prefix check --root ."
expect "check" "prints its output" grep -q "stub output" <<<"$out"

run_case "projects" "$work/stubs:$work/empty" 0 -- check --projects cookbook,pane
expect "projects" "passes --projects" test "$calls" = "$prefix check --root . --projects cookbook,pane"

run_case "check findings" "$work/stubs:$work/empty" 0 CARGO_STATUS=1
expect "check findings" "warns" grep -q "warning: cost check could not run" <<<"$out"

run_case "render ok" "$work/stubs:$work/empty" 0 -- render-check
expect "render ok" "runs cost render --check" test "$calls" = "$prefix render --root . --check"

run_case "render stale" "$work/stubs:$work/empty" 1 CARGO_STATUS=1 -- render-check

run_case "render broken" "$work/stubs:$work/empty" 0 CARGO_STATUS=2 -- render-check
expect "render broken" "warns" grep -q "warning: cost render could not run" <<<"$out"

run_case "no cargo" "$work/empty" 0
expect "no cargo" "warns" grep -q "skipped: cargo is not installed" <<<"$out"

run_case "no cargo render" "$work/empty" 0 -- render-check
expect "no cargo render" "warns" grep -q "skipped: cargo is not installed" <<<"$out"

if [ "$failures" -gt 0 ]; then
  echo "cost-check: $failures failure(s)"
  exit 1
fi
echo "cost-check: all cases passed"
