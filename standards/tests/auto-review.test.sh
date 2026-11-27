#!/usr/bin/env bash
# Tests standards/scripts/auto-review.sh decide (PR-003): jimhillr-reviewer
# approves only green agent PRs with no red or orange label, no protected
# path and a small diff, once per head commit.
#   bash standards/tests/auto-review.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
script=$root/standards/scripts/auto-review.sh
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
failures=0

quiet=$'## Attention\n\n- 🟢 requirements-unchanged\n- 🟢 cost-unchanged\n- 🟢 threat-risk-unchanged\n\n## What changed\n\nA fix.\n'
pr() { # draft, author, labels JSON, additions, base, body
  jq -n --argjson draft "$1" --arg author "$2" --argjson labels "$3" --argjson adds "$4" --arg base "${5:-main}" \
    --arg body "${6-$quiet}" \
    '{number: 7, draft: $draft, base: {ref: $base}, user: {login: $author}, head: {sha: "abcdef1234"},
      additions: $adds, deletions: 3, labels: $labels, body: $body}' >"$tmp/pr.json"
}
green='{"check_runs":[{"name":"CI result","status":"completed","conclusion":"success"},
  {"name":"PR summary","status":"completed","conclusion":"success"},
  {"name":"Jim review","status":"completed","conclusion":"success"},
  {"name":"Analyze (rust)","status":"completed","conclusion":"skipped"},
  {"name":"Auto review","status":"in_progress","conclusion":null}],"statuses":[]}'

check() { # name, files, checks JSON, reviews JSON, expected prefix
  printf '%s\n' "$2" >"$tmp/files.txt"
  printf '%s' "$3" >"$tmp/checks.json"
  printf '%s' "$4" >"$tmp/reviews.json"
  local out
  out=$(bash "$script" decide "$tmp/pr.json" "$tmp/files.txt" "$tmp/checks.json" "$tmp/reviews.json" 2>&1)
  if [[ $out != "$5"* ]]; then
    echo "FAIL $1: wanted '$5...', got: $out"
    failures=$((failures + 1))
  else
    echo "ok   $1"
  fi
}

pr false jimhillr-agt '[]' 40
check "green agent PR is approved" "pane/web/src/main.rs" "$green" '[]' "approve: jimhillr-reviewer rules v2"
check "a second run doesn't approve again" "pane/web/src/main.rs" "$green" \
  '[{"user":{"login":"jimhillr-reviewer"},"state":"APPROVED","commit_id":"abcdef1234"}]' "skip: already approved"
check "an approval of an older commit doesn't count" "pane/web/src/main.rs" "$green" \
  '[{"user":{"login":"jimhillr-reviewer"},"state":"APPROVED","commit_id":"0000000"}]' "approve:"
check "changes requested" "pane/web/src/main.rs" "$green" '[{"user":{"login":"jimhillr"},"state":"CHANGES_REQUESTED"}]' "skip: changes requested"
check "workflow change" $'pane/web/src/main.rs\n.github/workflows/x.yml' "$green" '[]' "skip: touches .github/workflows/x.yml"
check "threat model" "pane/docs/threat.toml" "$green" '[]' "skip: touches pane/docs/threat.toml"
check "requirements" "cookbook/docs/requirements.md" "$green" '[]' "skip: touches cookbook/docs/requirements.md"
check "evals" "pane/evals.toml" "$green" '[]' "skip: touches pane/evals.toml"
check "standards" "standards/scripts/audit.sh" "$green" '[]' "skip: touches standards/scripts/audit.sh"
check "security headers" "pane/web/public/_headers" "$green" '[]' "skip: touches pane/web/public/_headers"
check "a failing check" "pane/web/src/main.rs" \
  '{"check_runs":[{"name":"CI result","status":"completed","conclusion":"failure"}],"statuses":[]}' '[]' "skip: check CI result is failure"
check "a pending check" "pane/web/src/main.rs" \
  '{"check_runs":[{"name":"CI result","status":"completed","conclusion":"success"},{"name":"PR summary","status":"completed","conclusion":"success"},{"name":"Jim review","status":"completed","conclusion":"success"},{"name":"CodeQL","status":"queued"}],"statuses":[]}' '[]' "skip: check CodeQL is queued"
check "CI result missing" "pane/web/src/main.rs" \
  '{"check_runs":[{"name":"PR summary","status":"completed","conclusion":"success"}],"statuses":[]}' '[]' "skip: check CI result has not passed"
check "a failing commit status" "pane/web/src/main.rs" \
  "$(jq -c '.statuses = [{"context":"deploy","state":"pending"}]' <<<"$green")" '[]' "skip: check deploy is pending"

check "Jim review missing" "pane/web/src/main.rs" \
  "$(jq -c 'del(.check_runs[] | select(.name == "Jim review"))' <<<"$green")" '[]' "skip: check Jim review has not passed"
check "wrangler.jsonc" "pane/cloudflare/wrangler.jsonc" "$green" '[]' "skip: touches pane/cloudflare/wrangler.jsonc"

pr false jimhillr-agt '[]' 40 main $'## Attention\n\n- 🔴 must-requirement-changed: `COOK-001`\n- 🟢 cost-unchanged\n- 🟢 threat-risk-unchanged\n'
check "red attention tag" "pane/web/src/main.rs" "$green" '[]' "skip: attention tag must-requirement-changed needs Jim"
pr false jimhillr-agt '[]' 40 main $'## Attention\r\n\r\n- 🟢 requirements-unchanged\r\n- 🟠 cost-changed: `pane/docs/cost.toml`\r\n'
check "cost attention tag" "pane/web/src/main.rs" "$green" '[]' "skip: attention tag cost-changed needs Jim"
pr false jimhillr-agt '[]' 40 main $'## Attention\n\n- 🟢 requirements-unchanged\n- 🟢 cost-unchanged\n- 🟠 threat-risk-changed: x\n'
check "risk attention tag" "pane/web/src/main.rs" "$green" '[]' "skip: attention tag threat-risk-changed needs Jim"
pr false jimhillr-agt '[]' 40 main $'## What changed\n\nNo tags.\n\n## Details\n\n- cost-changed\n'
check "no Attention section" "pane/web/src/main.rs" "$green" '[]' "skip: no Attention section"
pr false jimhillr-agt '[]' 40 main $'## Attention\n\n- 🟡 requirement-changed: x\n- 🟢 cost-unchanged\n- 🟢 threat-risk-unchanged\n\n## Details\n\n- 🔴 must-requirement-changed is only mentioned here\n'
check "tags outside Attention don't count" "pane/web/src/main.rs" "$green" '[]' "approve:"

pr false jimhillr-agt '[{"name":"cost"}]' 40
check "orange label" "pane/web/src/main.rs" "$green" '[]' "skip: labelled cost"
pr false jimhillr-agt '[{"name":"needs-human"}]' 40
check "red label" "pane/web/src/main.rs" "$green" '[]' "skip: labelled needs-human"
pr true jimhillr-agt '[]' 40
check "draft" "pane/web/src/main.rs" "$green" '[]' "skip: draft"
pr false dependabot[bot] '[]' 40
check "not an agent PR" "pane/web/src/main.rs" "$green" '[]' "skip: author dependabot[bot]"
pr false jimhillr-agt '[]' 2000
check "too big" "pane/web/src/main.rs" "$green" '[]' "skip: 2003 changed lines"
pr false jimhillr-agt '[]' 40 claude/other
check "stacked" "pane/web/src/main.rs" "$green" '[]' "skip: based on claude/other"

if [ "$failures" -gt 0 ]; then
  echo "$failures auto-review test(s) failed"
  exit 1
fi
echo "auto-review: all tests passed"
