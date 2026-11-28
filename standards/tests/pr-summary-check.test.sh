#!/usr/bin/env bash
# Tests standards/scripts/pr-summary-check.sh (PR-001): the sections
# (Attention, What changed, Why, Cost changes, Risk changes, Details) are
# found by heading, attention tags come from the closed set with one per
# group, each summary is held to its word limit, links count as their
# text, and missing, empty, long or placeholder sections warn without
# failing.
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
_Requested by **Jim** · [project thread](https://claude.ai/code/project/x)_

## Attention

- 🔴 must-requirement-changed: `COOK-001`
- 🟢 cost-unchanged
- 🟠 threat-risk-changed: `cookbook/docs/threat.toml` (the Security summary check says whether it weakens anything)
- 🟢 docs-changed: `cookbook/docs/requirements.md` alongside other files'

att=$'## attention\n- 🟢 requirements-unchanged\n- `cost-unchanged`\n- threat-risk-unchanged: none\n- 🔵 docs-only: `a.md`\n'

words() { printf 'word %.0s' $(seq 1 "$1"); }

body() { # what, why, cost, risk, details
  printf '%s\n\n## What changed\n\n%s\n\n## Why\n\n%s\n\n## Cost changes\n\n%s\n\n## Risk changes\n\n%s\n\n## Details\n\n%s\n' \
    "$attribution" "$1" "$2" "$3" "$4" "$5"
}

good_what='Replaces the 75-word summary with five sections. See [the rules](standards/docs/pull-requests.md).'
good_why='Jim wants each PR quick to skim: what, why, cost and risk.'

check "all five sections" "$(body "$good_what" "$good_why" "None." "None." "$(words 200)")" \
  "PR-001 ok: What changed 10, Why 12, Cost changes 1, Risk changes 1 words."

check "at the limits" "$(body "$(words 50)" "$(words 50)" "$(words 24)" "$(words 24)" "")" \
  "PR-001 ok: What changed 50, Why 50, Cost changes 24, Risk changes 24 words."

check "What changed too long" "$(body "$(words 51)" "$good_why" "None." "None." "x")" \
  "\"## What changed\" section is 51 words; keep it to at most 50 words"

check "Why too long" "$(body "$good_what" "$(words 60) [doc](https://example.invalid/a/b)" "None." "None." "x")" \
  "\"## Why\" section is 61 words"

check "Cost at 25 is too long" "$(body "$good_what" "$good_why" "$(words 25)" "None." "x")" \
  "\"## Cost changes\" section is 25 words; keep it to under 25 words"

check "Risk too long" "$(body "$good_what" "$good_why" "None." "$(words 30)" "x")" \
  "\"## Risk changes\" section is 30 words"

check "empty cost" "$(body "$good_what" "$good_why" "" "None." "x")" \
  "\"## Cost changes\" section is empty; write \"None.\""

check "missing sections" "$attribution

Old-style 75-word summary, see [doc](a.md).

## Testing

Hooks passed." "no \"## What changed\" section"

check "missing Details" "$attribution

## What changed
$good_what
## Why
$good_why
## Cost changes
None.
## Risk changes
None." "no \"## Details\" section"

check "unknown attention tag" "$(body "$good_what" "$good_why" "None." "None." "x" | sed 's/cost-unchanged/looks-fine/')" \
  "\"looks-fine\" is not an attention tag; use one of must-requirement-changed,"

check "missing cost tag" "$(body "$good_what" "$good_why" "None." "None." "x" | sed 's/cost-unchanged/looks-fine/')" \
  "\"## Attention\" section has 0 cost tags; it needs exactly one."

check "two requirements tags" "$(body "$good_what" "$good_why" "None." "None." "x" | sed 's/^- 🟢 cost-unchanged$/- 🟢 cost-unchanged\n- 🟢 requirements-unchanged/')" \
  "\"## Attention\" section has 2 requirements tags"

check "two docs tags" "$(body "$good_what" "$good_why" "None." "None." "x" | sed 's/^- 🟢 docs-changed: .*$/- 🔵 docs-only\n- 🟢 docs-unchanged/')" \
  "\"## Attention\" section has 2 docs tags"

check "missing Attention" "$(body "$good_what" "$good_why" "None." "None." "x" | sed 's/^## Attention$/## Badges/')" \
  "no \"## Attention\" section"

check "headings any level and case" "$att$(printf '### what changed\nA b.\n# WHY\nC d.\n## cost changes\nNone.\n## Risk Changes\nNone.\n## details\n')" \
  "PR-001 ok: What changed 2, Why 2, Cost changes 1, Risk changes 1 words."

check "rule ends a section" "$att$(printf '## What changed\nA b.\n\n---\n\n%s\n## Why\nC.\n## Cost changes\nNone.\n## Risk changes\nNone.\n## Details\n' "$(words 80)")" \
  "PR-001 ok: What changed 2,"

check "CRLF body" "$att$(printf '## What changed\r\nA [doc](a.md).\r\n## Why\r\nB.\r\n## Cost changes\r\nNone.\r\n## Risk changes\r\nNone.\r\n## Details\r\n')" \
  "PR-001 ok: What changed 2,"

check "comments ignored" "$att$(printf '## What changed\n<!--\nlots of guidance words that must not count\n-->\nShort.\n## Why\nB.\n## Cost changes\nNone.\n## Risk changes\nNone.\n## Details\n')" \
  "PR-001 ok: What changed 1,"

check "template placeholder" "$(cat "$root/.github/pull_request_template.md")" \
  "\"## What changed\" section is still the template's placeholder."

if [ "$failures" -gt 0 ]; then
  echo "$failures pr-summary-check test(s) failed"
  exit 1
fi
echo "pr-summary-check tests passed"
