#!/usr/bin/env bash
# Tests ISS-004 in standards/scripts/issues.sh in a throwaway repo: a human
# task needs the Requirement, Why and Runbook sections and a `runbook`
# field naming a runbook that exists.
#   bash standards/tests/issues.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
repo=$work/repo
failures=0

mkdir -p "$repo/standards/scripts" "$repo/standards/docs/runbooks" "$repo/cost/docs/issues/human"
cp "$root/standards/scripts/issues.sh" "$repo/standards/scripts/"
echo '# Do it' >"$repo/standards/docs/runbooks/do-it.md"
git -C "$repo" init -q

task() { # task FILE RUNBOOK [SECTIONS...]: write a human task.
  local f=$1 rb=$2
  shift 2
  {
    printf -- '---\nadded: 2026-11-27\n'
    [ -n "$rb" ] && echo "runbook: $rb"
    printf -- '---\n\n'
    for s in "$@"; do printf '## %s\n\nText.\n\n' "$s"; done
  } >"$repo/cost/docs/issues/human/$f"
}

# expect NAME WANT_EXIT WANT_TEXT: run --check and compare.
expect() {
  local out code
  out=$(cd "$repo" && bash standards/scripts/issues.sh --check 2>&1)
  code=$?
  if [ "$code" != "$2" ] || ! grep -qF -- "$3" <<<"$out"; then
    echo "FAIL $1: exit $code, output:"
    echo "$out"
    failures=$((failures + 1))
  else
    echo "ok   $1"
  fi
  rm -f "$repo"/cost/docs/issues/human/*.md "$repo"/issues/*.md
}

all=("Requirement" "Why it can't be automated" "Runbook")
task good.md standards/docs/runbooks/do-it.md "${all[@]}"
expect "a complete task passes" 0 ""

task no-why.md standards/docs/runbooks/do-it.md "Requirement" "Runbook"
expect "missing why fails" 1 "needs a '## Why it can't be automated' section"

task no-field.md "" "${all[@]}"
expect "missing runbook field fails" 1 "needs 'runbook: standards/docs/runbooks/<name>.md'"

task gone.md standards/docs/runbooks/nope.md "${all[@]}"
expect "missing runbook file fails" 1 "runbook standards/docs/runbooks/nope.md doesn't exist"

task elsewhere.md docs/do-it.md "${all[@]}"
expect "runbook outside the folder fails" 1 "must be a file in standards/docs/runbooks/"

mkdir -p "$repo/issues"
printf -- '---\nkind: todo\npriority: backlog\nadded: 2026-11-28\n---\n\nDo it.\n' >"$repo/issues/root.md"
expect "an issue in the retired root issues/ folder fails" 1 "error[ISS-001] issues/root.md"

if [ "$failures" -gt 0 ]; then
  echo "$failures failure(s)"
  exit 1
fi
