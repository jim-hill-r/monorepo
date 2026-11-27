#!/usr/bin/env bash
# Tests standards/scripts/billing-check.sh with a stub `curl` that answers
# from files, so each vendor's clean, problem and unset cases are checked.
#   bash standards/tests/billing-check.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
script=$root/standards/scripts/billing-check.sh
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0
ok() { echo "ok: $*"; }
fail() { echo "FAIL: $*"; failures=$((failures + 1)); }

mkdir -p "$work/bin" "$work/resp"
# curl: logs its arguments and stdin (the -K config), then prints
# $work/resp/<key> and its status, where <key> names the endpoint.
cat >"$work/bin/curl" <<STUB
#!/usr/bin/env bash
url=""
for a in "\$@"; do case "\$a" in http*) url=\$a ;; esac; done
echo "args: \$*" >>"$work/curl.log"
echo "config: \$(cat)" >>"$work/curl.log"
case "\$url" in
  */payment-methods) key=cf-pm ;;
  */subscriptions) key=cf-subs ;;
  */settings/billing/usage*) key=gh-usage ;;
  */api/billing/) key=ph-billing ;;
  *) echo "curl stub: unexpected \$url" >&2; exit 22 ;;
esac
cat "$work/resp/\$key"
printf '\n%s' "\$(cat "$work/resp/\$key.code" 2>/dev/null || echo 200)"
STUB
chmod +x "$work/bin/curl"

clean() {
  rm -f "$work/resp/"*
  echo '{"success":true,"errors":[],"result":[]}' >"$work/resp/cf-pm"
  echo '{"success":true,"errors":[],"result":[{"rate_plan":{"id":"teams_free"},"price":0}]}' >"$work/resp/cf-subs"
  echo '{"usageItems":[{"product":"actions","sku":"linux","grossAmount":3.2,"discountAmount":3.2,"netAmount":0}]}' >"$work/resp/gh-usage"
  echo '{"has_active_subscription":false,"current_total_amount_usd":"0.00","subscription_level":"free","stripe_portal_url":"https://portal.example.invalid/secret-session"}' >"$work/resp/ph-billing"
}
creds=(CLOUDFLARE_BILLING_READ_TOKEN=cf-secret CLOUDFLARE_ACCOUNT_ID=acct
  GITHUB_BILLING_READ_TOKEN=gh-secret GITHUB_BILLING_USER=jimhillr POSTHOG_BILLING_READ_KEY=ph-secret)
run() {
  : >"$work/curl.log"; : >"$work/summary"
  env -i PATH="$work/bin:$PATH" HOME="$HOME" GITHUB_STEP_SUMMARY="$work/summary" "$@" bash "$script" all >"$work/out" 2>&1
}

clean
run "${creds[@]}" && ok "clean accounts pass" || fail "clean: $(cat "$work/out")"
grep -q "no payment method" "$work/summary" && grep -q "teams_free" "$work/summary" &&
  grep -q '\$0 net billing usage' "$work/summary" && grep -q "no active subscription" "$work/summary" &&
  ok "summary has a row per vendor" || fail "summary: $(cat "$work/summary")"
! grep -q "secret-session" "$work/out" "$work/summary" && ok "billing portal links are never printed" || fail "portal link leaked"
! grep -q "secret" <(grep '^args' "$work/curl.log") && grep -q "Bearer cf-secret" "$work/curl.log" &&
  ok "credentials reach curl through stdin, not its arguments" || fail "credential on a command line: $(cat "$work/curl.log")"
grep -q "settings/billing/usage?year=$(date -u +%Y)&month=$((10#$(date -u +%m)))" "$work/curl.log" &&
  ok "GitHub usage is this month's" || fail "github url"

run && ok "unset credentials only warn" || fail "unset: $(cat "$work/out")"
[ "$(grep -c '::warning::' "$work/out")" = 3 ] && grep -q "not checked" "$work/summary" &&
  ok "each unset vendor is 'not checked'" || fail "unset warnings: $(cat "$work/out")"

expect_problem() {
  local name=$1 text=$2
  if run "${creds[@]}"; then fail "$name passed"; elif grep -q "$text" "$work/out"; then ok "$name fails"; else fail "$name: $(cat "$work/out")"; fi
  clean
}
clean
echo '{"success":true,"result":[{"type":"card","last_four":"4242","address":"1 Main St"}]}' >"$work/resp/cf-pm"
expect_problem "a Cloudflare card" "1 payment method"
echo '{"success":true,"result":[{"type":"card","last_four":"4242","address":"1 Main St"}]}' >"$work/resp/cf-pm"
run "${creds[@]}"; ! grep -q "4242\|Main St" "$work/out" "$work/summary" && ok "card details are never printed" || fail "card details leaked"
clean
echo '{"success":true,"result":[{"rate_plan":{"id":"workers_paid"},"price":5,"frequency":"monthly"}]}' >"$work/resp/cf-subs"
expect_problem "Workers Paid" "workers_paid"
echo '{"success":true,"result":[{"rate_plan":{"id":"r2_paid"},"price":0}]}' >"$work/resp/cf-subs"
expect_problem "R2 paid at \$0 base" "r2_paid"
echo '{"success":false,"errors":[{"code":10000,"message":"Authentication error"}]}' >"$work/resp/cf-pm"
echo 403 >"$work/resp/cf-pm.code"
expect_problem "a Cloudflare auth error" "HTTP 403"
echo '{"usageItems":[{"product":"actions","sku":"linux_4_core","netAmount":1.5}]}' >"$work/resp/gh-usage"
expect_problem "a GitHub charge" "actions/linux_4_core"
echo '{"has_active_subscription":true,"billing_plan":"paid"}' >"$work/resp/ph-billing"
expect_problem "a PostHog subscription" "active paid subscription"
echo '{"has_active_subscription":false,"current_total_amount_usd":"12.50"}' >"$work/resp/ph-billing"
expect_problem "a PostHog charge" '12.5'

# Every vendor in the tech radar is in the summary (Jim, 2026-11-27).
clean
run "${creds[@]}"
for v in $(jq -r '.[] | select(.kind == "vendor") | .name | @base64' "$root/standards/radar/radar.json"); do
  name=$(base64 -d <<<"$v")
  grep -qF "| $name |" "$work/summary" || fail "summary misses radar vendor $name"
done
grep -q '| Neon | Assess | - | no account |' "$work/summary" && grep -q '`auth0-billing`' "$work/summary" &&
  ok "summary lists every radar vendor and how it is checked" || fail "radar table: $(tail -20 "$work/summary")"
[ "$(bash "$script" vendors)" = "cloudflare github posthog" ] && ok "vendors lists the API checks" || fail "vendors"

# RAD-007: radar.sh fails a vendor with no billing entry, an unknown api,
# an unknown setting, or an approved vendor that nothing checks.
rad() {
  local fake=$work/radar-root
  rm -rf "$fake"; mkdir -p "$fake/standards/scripts" "$fake/standards/radar" "$fake/standards/docs"
  cp "$root/standards/scripts/radar.sh" "$root/standards/scripts/billing-check.sh" "$fake/standards/scripts/"
  cp "$root/standards/settings.toml" "$fake/standards/"
  cp "$root/standards/docs/tech-radar.md" "$fake/standards/docs/"
  jq "$1" "$root/standards/radar/radar.json" >"$fake/standards/radar/radar.json"
  (cd "$fake" && for p in $(jq -r '.[] | (.exemplars[], (.sources[] | select(test("^https?://") | not)))' standards/radar/radar.json); do
    [ -e "$p" ] || mkdir -p "$p"; done)
  bash "$fake/standards/scripts/radar.sh" check 2>&1
}
expect_rad() {
  local name=$1 filter=$2 text=$3 out
  if out=$(rad "$filter"); then fail "RAD-007 $name passed"; elif grep -q "RAD-007.*$text" <<<"$out"; then ok "RAD-007 $name"; else fail "RAD-007 $name: $out"; fi
}
expect_rad "missing billing" 'map(if .name == "Auth0" then del(.billing) else . end)' "Auth0: a vendor needs billing"
expect_rad "unknown api" 'map(if .name == "Auth0" then .billing.api = ["auth0"] else . end)' "auth0 is not a vendor"
expect_rad "unknown setting" 'map(if .name == "Auth0" then .billing.human = "nope" else . end)' "nope is not a setting id"
expect_rad "unchecked Trial vendor" 'map(if .name == "Neon" then .ring = "Trial" else . end)' "Neon: an Trial vendor needs a billing check"

code=0; bash "$script" nope >/dev/null 2>&1 || code=$?
[ "$code" = 2 ] && ok "an unknown vendor is a usage error" || fail "usage: $code"

if [ "$failures" -gt 0 ]; then
  echo "billing-check tests: $failures failed"
  exit 1
fi
echo "billing-check tests: all passed"
