#!/usr/bin/env bash
# Tests standards/scripts/evals-check.sh: it skips cargo when no project has
# requirements or evals, runs `evals check` when one does, and never fails.
# cargo is a stub. Run from anywhere:
#   bash standards/tests/evals-check.test.sh
set -uo pipefail

# Git hooks export these; they would point the scratch repo at this one.
unset GIT_DIR GIT_INDEX_FILE GIT_WORK_TREE GIT_PREFIX

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/repo/standards/scripts" "$work/stubs" "$work/empty"
cp "$root/standards/scripts/evals-check.sh" "$work/repo/standards/scripts/"
git -C "$work/repo" init -q
cat >"$work/stubs/cargo" <<SH
#!/bin/sh
echo "cargo \$*" >>"$work/calls"
[ -z "\${FAIL_CARGO:-}" ] || exit 2
echo "evals check: 0 finding(s)"
SH
chmod +x "$work/stubs/cargo"
# The script needs only git and grep besides cargo; link them so PATH can
# leave cargo out.
for tool in git grep; do ln -s "$(command -v $tool)" "$work/empty/$tool"; done
bash_bin=$(command -v bash)

failures=0

# run_case NAME PATH [VAR=value ...]: leaves output in $out, cargo calls in $calls.
run_case() {
  local name=$1 path=$2
  shift 2
  : >"$work/calls"
  out=$(cd "$work/repo" && env -u FAIL_CARGO "$@" PATH="$path" "$bash_bin" standards/scripts/evals-check.sh 2>&1)
  status=$?
  calls=$(cat "$work/calls")
  if [ "$status" -ne 0 ]; then
    echo "FAIL $name: exit $status, want 0"; echo "$out"; failures=$((failures + 1))
  fi
}

expect() { # expect NAME CONDITION_DESCRIPTION TEST...
  local name=$1 what=$2
  shift 2
  if ! "$@"; then
    echo "FAIL $name: $what"; echo "output: $out"; echo "calls: $calls"
    failures=$((failures + 1))
  fi
}

run_case "no files" "$work/stubs:$work/empty"
expect "no files" "cargo not called" test -z "$calls"

mkdir -p "$work/repo/cookbook/docs"
echo "# Cookbook requirements" >"$work/repo/cookbook/docs/requirements.md"
git -C "$work/repo" add cookbook/docs/requirements.md

run_case "with files" "$work/stubs:$work/empty"
expect "with files" "runs evals check" \
  test "$calls" = "cargo run --quiet --manifest-path evals/Cargo.toml -p evals_cli -- check --root ."
expect "with files" "prints its output" grep -q "0 finding" <<<"$out"

run_case "cargo fails" "$work/stubs:$work/empty" FAIL_CARGO=1
expect "cargo fails" "warns" grep -q "warning: evals check could not run" <<<"$out"

run_case "no cargo" "$work/empty"
expect "no cargo" "warns" grep -q "skipped: cargo is not installed" <<<"$out"

if [ "$failures" -gt 0 ]; then
  echo "evals-check: $failures failure(s)"
  exit 1
fi
echo "evals-check: all cases passed"
