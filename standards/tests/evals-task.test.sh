#!/usr/bin/env bash
# Tests standards/scripts/evals.sh, the eval steps of the shared mise tasks:
# `report` saves a command's output and keeps its exit status, `run` passes
# the saved reports to the evals CLI (cargo is a stub) and does nothing for a
# project without an evals.toml, and `publish` skips a missing results file.
# Run from anywhere:
#   bash standards/tests/evals-task.test.sh
set -uo pipefail

script="$(cd "$(dirname "$0")/.." && pwd)/scripts/evals.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# A repo with a project that has sub-projects (cookbook, cookbook/web) and
# one without an evals.toml (homer).
repo="$work/repo"
mkdir -p "$repo/cookbook/web" "$repo/homer" "$work/stubs"
git -C "$repo" init -q
touch "$repo/cookbook/evals.toml" "$repo/cookbook/mise.toml" "$repo/cookbook/web/mise.toml" "$repo/homer/mise.toml"
git -C "$repo" add -A

# cargo stub: logs its arguments.
cat >"$work/stubs/cargo" <<SH
#!/bin/sh
echo "cargo \$*" >>"$work/calls"
SH
chmod +x "$work/stubs/cargo"

failures=0
fail() { echo "FAIL $1"; [ -z "${2:-}" ] || echo "$2" | sed 's/^/    /'; failures=$((failures + 1)); }
expect() { case "$3" in *"$2"*) ;; *) fail "$1: missing '$2'" "$3" ;; esac; }
refuse() { case "$3" in *"$2"*) fail "$1: unexpected '$2'" "$3" ;; esac; }

# evals DIR ARGS...: runs evals.sh in DIR with the stub cargo; output in $out,
# cargo calls in $calls, status in $status.
evals() {
  local dir=$1
  shift
  : >"$work/calls"
  out=$(cd "$dir" && env -u EVALS_RESULTS -u EVALS_STAGE PATH="$work/stubs:/usr/bin:/bin" bash "$script" "$@" 2>&1)
  status=$?
  calls=$(cat "$work/calls")
}

# report: saves the output and keeps the command's exit status.
evals "$repo/homer" report cargo-test sh -c 'echo "test a ... ok"; exit 3'
[ "$status" -eq 3 ] || fail "report: exit $status, want 3"
expect report-output "test a ... ok" "$out"
expect report-file "test a ... ok" "$(cat "$repo/homer/target/evals-reports/cargo-test.log" 2>/dev/null)"
evals "$repo/homer" report cargo-test true
[ "$status" -eq 0 ] || fail "report-pass: exit $status, want 0"

# run: a project without evals.toml collects nothing.
evals "$repo/homer" run
[ "$status" -eq 0 ] || fail "run-no-evals: exit $status"
expect run-no-evals "homer has no evals.toml" "$out"
[ -z "$calls" ] || fail "run-no-evals: cargo ran" "$calls"

# run: a sub-project passes its reports, the top-level project and --partial.
mkdir -p "$repo/cookbook/web/target/evals-reports"
touch "$repo/cookbook/web/target/evals-reports/cargo-test.log" "$repo/cookbook/web/target/evals-reports/playwright.json"
evals "$repo/cookbook/web" run
[ "$status" -eq 0 ] || fail "run-sub: exit $status" "$out"
expect run-sub "--project cookbook" "$calls"
expect run-sub "--stage ci" "$calls"
expect run-sub "--cargo-output $repo/cookbook/web/target/evals-reports/cargo-test.log" "$calls"
expect run-sub "--playwright-report $repo/cookbook/web/target/evals-reports/playwright.json" "$calls"
expect run-sub "--partial" "$calls"

# run: a top-level project with sub-projects is partial too, and passes no
# report it didn't save.
evals "$repo/cookbook" run
expect run-top "--project cookbook" "$calls"
expect run-top "--partial" "$calls"
refuse run-top "--cargo-output" "$calls"
refuse run-top "--playwright-report" "$calls"

# run: a project with no sub-projects is complete.
touch "$repo/homer/evals.toml"
git -C "$repo" add -A
evals "$repo/homer" run
expect run-whole "--project homer" "$calls"
refuse run-whole "--partial" "$calls"

# publish: no results file, nothing to do.
evals "$repo/cookbook/web" publish
[ "$status" -eq 0 ] || fail "publish-none: exit $status"
[ -z "$calls" ] || fail "publish-none: cargo ran" "$calls"
touch "$repo/cookbook/web/target/evals-reports/evals-results.jsonl"
evals "$repo/cookbook/web" publish
expect publish "publish --results $repo/cookbook/web/target/evals-reports/evals-results.jsonl" "$calls"

if [ "$failures" -gt 0 ]; then
  echo "evals-task: $failures failure(s)"
  exit 1
fi
echo "evals-task: all cases passed"
