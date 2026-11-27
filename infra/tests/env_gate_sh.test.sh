#!/usr/bin/env bash
# Tests for infra/scripts/env-gate.sh with a stub `gh` that prints an
# environment's settings.
set -euo pipefail

here=$(cd "$(dirname "$0")/.." && pwd)
script="$here/scripts/env-gate.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0
fail() { echo "FAIL: $*"; failures=$((failures + 1)); }
ok() { echo "ok: $*"; }

mkdir -p "$work/bin"
cat >"$work/bin/gh" <<STUB
#!/usr/bin/env bash
cat "$work/env.json"
STUB
chmod +x "$work/bin/gh"

gate() { PATH="$work/bin:$PATH" GITHUB_REPOSITORY=o/r bash "$script" infra-credentials 2>&1; }
protected='{"can_admins_bypass":false,"deployment_branch_policy":{"protected_branches":false,"custom_branch_policies":true},"protection_rules":[{"type":"branch_policy"},{"type":"required_reviewers","reviewers":[{"type":"User","reviewer":{"login":"jimhillr"}}]}]}'

echo "$protected" >"$work/env.json"
out=$(gate) && ok "a protected environment passes" || fail "protected: $out"

echo "$protected" | jq '.protection_rules = [{"type":"branch_policy"}]' >"$work/env.json"
out=$(gate) && fail "no reviewer passed" || { grep -q "no required reviewer" <<<"$out" && ok "no reviewer fails" || fail "no reviewer: $out"; }

echo "$protected" | jq '.can_admins_bypass = true' >"$work/env.json"
out=$(gate) && fail "admin bypass passed" || { grep -q "bypass" <<<"$out" && ok "admin bypass fails" || fail "bypass: $out"; }

echo "$protected" | jq '.deployment_branch_policy = null' >"$work/env.json"
out=$(gate) && fail "any branch passed" || { grep -q "any branch" <<<"$out" && ok "any branch fails" || fail "branches: $out"; }

if [ "$failures" -gt 0 ]; then
  echo "env-gate.sh tests: $failures failed"
  exit 1
fi
echo "env-gate.sh tests: all passed"
