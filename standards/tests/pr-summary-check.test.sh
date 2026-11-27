#!/usr/bin/env bash
# Tests standards/scripts/pr-summary-check.sh (PR-001): the summary after the
# attribution lines is counted up to the first heading, links count as their
# text, and long, missing or link-less summaries warn without failing.
#   bash standards/tests/pr-summary-check.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
script=$root/standards/scripts/pr-summary-check.sh
failures=0

check() { # name, body, expected substring
  local out
  out=$(PR_BODY=$2 GITHUB_ACTIONS='' GITHUB_STEP_SUMMARY='' bash "$script" 2>&1)
  local status=$?
  if [ "$status" -ne 0 ] || ! grep -qF -- "$3" <<<"$out"; then
    echo "FAIL $1: wanted '$3' (exit $status), got:"
    echo "$out"
    failures=$((failures + 1))
  else
    echo "ok   $1"
  fi
}

attribution='<!-- claude-projects-attribution: {"github_login":"jimhillr-agt"} -->
_Requested by **Jim** · [project thread](https://claude.ai/code/project/x)_'

long=$(printf 'word %.0s' $(seq 1 80))

check "short summary with link" "$attribution

Adds a PR summary rule so reviewers read PRs fast. See [the plan](standards/docs/pull-requests.md).

## Details

$long" "PR-001 ok: executive summary is 13 words."

check "Summary heading is skipped" "$attribution

## Summary

Adds a rule. Details in [pull-requests.md](https://example.invalid/a/b/c).

## Testing
$long" "ok: executive summary is 6 words."

check "too long" "$attribution

$long [doc](x.md)" "is 81 words; keep it to 75"

check "missing summary" "$attribution

## What changed
$long" "has no executive summary"

check "no link" "$attribution

Adds a rule without pointing anywhere." "has no link"

check "CRLF body" "$(printf 'Short summary, see [doc](a.md).\r\n\r\n## More\r\n')" "is 4 words."

check "template placeholder" "$(cat "$root/.github/pull_request_template.md")" "still the template's placeholder"

check "multi-line comment ignored" "<!--
lots of guidance words here that must not count
-->
Short, see [doc](a.md)." "is 3 words."

if [ "$failures" -gt 0 ]; then
  echo "$failures pr-summary-check test(s) failed"
  exit 1
fi
echo "pr-summary-check tests passed"
