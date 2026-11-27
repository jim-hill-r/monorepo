#!/usr/bin/env bash
# Tests standards/scripts/threat-check.sh: it runs `threat check` (passing
# --projects, THREAT_PROJECTS or COST_PROJECTS through), never fails the
# audit, and skips without cargo. cargo is a stub. Run from anywhere:
#   bash standards/tests/threat-check.test.sh
set -uo pipefail

# Git hooks export these; they would point the scratch repo at this one.
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE GIT_PREFIX

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/repo/standards/scripts" "$work/stubs" "$work/empty"
cp "$root/standards/scripts/threat-check.sh" "$work/repo/standards/scripts/"
git -C "$work/repo" init -q
# CARGO_STATUS sets the stub's exit status.
cat >"$work/stubs/cargo" <<SH
#!/bin/sh
echo "cargo \$*" >>"$work/calls"
echo "threat: stub output"
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
  out=$(cd "$work/repo" && env -u CARGO_STATUS -u COST_PROJECTS -u THREAT_PROJECTS "${vars[@]}" PATH="$path" "$bash_bin" standards/scripts/threat-check.sh "$@" 2>&1)
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

prefix="cargo run --quiet --manifest-path threat/Cargo.toml -p threat_cli -- check --root ."

run_case "check" "$work/stubs:$work/empty" 0
expect "check" "runs threat check" test "$calls" = "$prefix"
expect "check" "prints its output" grep -q "stub output" <<<"$out"

run_case "projects" "$work/stubs:$work/empty" 0 -- --projects cookbook,pane
expect "projects" "passes --projects" test "$calls" = "$prefix --projects cookbook,pane"

run_case "affected" "$work/stubs:$work/empty" 0 COST_PROJECTS=cookbook,pane
expect "affected" "passes COST_PROJECTS as --projects" test "$calls" = "$prefix --projects cookbook,pane"

run_case "threat projects" "$work/stubs:$work/empty" 0 THREAT_PROJECTS=pane COST_PROJECTS=cookbook
expect "threat projects" "prefers THREAT_PROJECTS" test "$calls" = "$prefix --projects pane"

run_case "none affected" "$work/stubs:$work/empty" 0 COST_PROJECTS=
expect "none affected" "passes an empty --projects" test "$calls" = "$prefix --projects "

run_case "args win" "$work/stubs:$work/empty" 0 COST_PROJECTS=cookbook -- --strict
expect "args win" "passes its own arguments only" test "$calls" = "$prefix --strict"

run_case "check broken" "$work/stubs:$work/empty" 0 CARGO_STATUS=2
expect "check broken" "warns" grep -q "warning: threat check could not run" <<<"$out"

run_case "no cargo" "$work/empty" 0
expect "no cargo" "warns" grep -q "skipped: cargo is not installed" <<<"$out"

if [ "$failures" -gt 0 ]; then
  echo "threat-check: $failures failure(s)"
  exit 1
fi
echo "threat-check: all cases passed"
