#!/usr/bin/env bash
# Billing check: proves with read-only billing credentials that the vendors
# whose APIs allow it still have no payment method, no paid plan and no
# charges (Jim, 2026-11-27: "create some credentials with readonly that
# would let our processes audit things like this"). The `billing` job of
# All Free Tier Usage runs it weekly, in the `infra` environment, so PR
# branches never get these credentials.
#
#   billing-check.sh [all|cloudflare|github|posthog]
#   billing-check.sh vendors     the API names it knows, for radar.sh RAD-007
#
# `all` covers every vendor in the tech radar (Jim, 2026-11-27): it runs the
# API check each vendor's `billing.api` in standards/radar/radar.json names,
# and lists every other vendor with the settings check Jim does for it, or
# that it has no account.
#
#   cloudflare  CLOUDFLARE_BILLING_READ_TOKEN (Billing Read, minted by Infra
#               Mint) and CLOUDFLARE_ACCOUNT_ID: no payment method, and no
#               subscription with a price or on a paid rate plan.
#   github      GITHUB_BILLING_READ_TOKEN (Jim's fine-grained token with only
#               the "Plan" account permission, read) and GITHUB_BILLING_USER:
#               this month's billing usage has a net amount of $0.
#   posthog     POSTHOG_BILLING_READ_KEY (Jim's personal API key with only
#               billing:read) and POSTHOG_API_HOST: no active subscription
#               and $0 this period.
#
# It only reads, and prints only counts, plan names and amounts: never card,
# address or billing-portal details. A credential that isn't set yet is a
# warning, not a failure, and the vendor shows as "not checked" in the run
# summary. Exit 1 when anything is found, or a call fails. What no API can
# show stays on cost/docs/issues/human/check-there-still-no-payment-method.md
# ("Billing checks" in standards/docs/free-tiers.md).
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
RADAR=${RADAR:-$root/standards/radar/radar.json}
KNOWN="cloudflare github posthog"
CF_API=${CF_API:-https://api.cloudflare.com/client/v4}
GH_API=${GH_API:-https://api.github.com}
POSTHOG_API_HOST=${POSTHOG_API_HOST:-https://us.posthog.com}
summary=${GITHUB_STEP_SUMMARY:-/dev/null}
problems=0

row() { echo "| $1 | $2 |" >>"$summary"; echo "$1: $2"; }
problem() {
  echo "::error::billing check: $1: $2"
  echo "| $1 | **Problem:** $2 |" >>"$summary"
  problems=$((problems + 1))
}
unset_cred() {
  echo "::warning::billing check: $1 not checked, $2 isn't set yet ($3)"
  row "$1" "not checked: \`$2\` isn't set yet ($3)"
}

# get URL AUTH_HEADER: prints the response body, or what went wrong and
# returns 1 (it runs in a subshell, so the caller records the problem). The
# header comes through stdin so no credential is on a command line.
get() {
  local url=$1 auth=$2 out code
  out=$(printf 'header = "%s"\n' "$auth" | curl -sS -K - -H "Accept: application/json" -w '\n%{http_code}' "$url") ||
    { echo "GET ${url%%\?*} failed"; return 1; }
  code=${out##*$'\n'}
  out=${out%$'\n'*}
  if [[ "$code" != 2* ]] || ! jq -e . >/dev/null 2>&1 <<<"$out"; then
    echo "GET ${url%%\?*}: HTTP $code $(jq -c '.errors // .message // .detail // empty' <<<"$out" 2>/dev/null | head -c 300)"
    return 1
  fi
  printf '%s' "$out"
}

# Rate plans that free-tiers.md forbids even when their base price is $0
# (R2 and Images bill per use; Workers Paid is a plan change).
CF_PAID_PLANS='^(workers_paid|r2_paid|images_.*|stream_.*|.*_pro|.*_business|.*_enterprise|.*_paid)$'

cloudflare() {
  if [ -z "${CLOUDFLARE_BILLING_READ_TOKEN:-}" ] || [ -z "${CLOUDFLARE_ACCOUNT_ID:-}" ]; then
    unset_cred Cloudflare CLOUDFLARE_BILLING_READ_TOKEN "approve an Infra Mint run"
    return 0
  fi
  local acct=$CF_API/accounts/$CLOUDFLARE_ACCOUNT_ID auth="Authorization: Bearer $CLOUDFLARE_BILLING_READ_TOKEN" pm subs n paid
  if ! pm=$(get "$acct/payment-methods" "$auth"); then
    problem Cloudflare "$pm"
  else
    if ! jq -e '.success == true' >/dev/null <<<"$pm"; then
      problem Cloudflare "payment methods: $(jq -c '.errors' <<<"$pm")"
    else
      n=$(jq '.result // [] | length' <<<"$pm")
      if [ "$n" -gt 0 ]; then
        problem Cloudflare "$n payment method(s) on file; remove it (standards/docs/runbooks/payment-method-check.md)"
      else
        row Cloudflare "no payment method"
      fi
    fi
  fi
  if ! subs=$(get "$acct/subscriptions" "$auth"); then
    problem Cloudflare "$subs"
  else
    if ! jq -e '.success == true' >/dev/null <<<"$subs"; then
      problem Cloudflare "subscriptions: $(jq -c '.errors' <<<"$subs")"
      return 0
    fi
    row Cloudflare "subscriptions: $(jq -r '[.result // [] | .[] | "\(.rate_plan.id // "?") ($\(.price // 0))"]
      | if length == 0 then "none" else join(", ") end' <<<"$subs")"
    paid=$(jq -r --arg re "$CF_PAID_PLANS" '[.result // [] | .[]
      | select((.price // 0) > 0 or ((.rate_plan.id // "") | test($re)))
      | "\(.rate_plan.id // "?") at $\(.price // 0) \(.frequency // "")"] | join(", ")' <<<"$subs")
    [ -z "$paid" ] || problem Cloudflare "paid subscription: $paid; free-tiers.md rule 1 forbids it"
  fi
}

github() {
  if [ -z "${GITHUB_BILLING_READ_TOKEN:-}" ] || [ -z "${GITHUB_BILLING_USER:-}" ]; then
    unset_cred GitHub GITHUB_BILLING_READ_TOKEN "standards/docs/runbooks/billing-read-credentials.md"
    return 0
  fi
  local year month out net
  year=$(date -u +%Y)
  month=$((10#$(date -u +%m)))
  if ! out=$(get "$GH_API/users/$GITHUB_BILLING_USER/settings/billing/usage?year=$year&month=$month" \
    "Authorization: Bearer $GITHUB_BILLING_READ_TOKEN"); then
    problem GitHub "$out"
    return 0
  fi
  net=$(jq '[.usageItems // [] | .[] | (.netAmount // 0)] | add // 0' <<<"$out")
  if jq -e --argjson n "$net" -n '$n > 0' >/dev/null; then
    problem GitHub "\$$net billed so far in $year-$month: $(jq -r '[.usageItems[] | select((.netAmount // 0) > 0) | "\(.product)/\(.sku) $\(.netAmount)"] | join(", ")' <<<"$out")"
  else
    row GitHub "\$0 net billing usage in $year-$month ($(jq '.usageItems // [] | length' <<<"$out") usage lines)"
  fi
}

posthog() {
  if [ -z "${POSTHOG_BILLING_READ_KEY:-}" ]; then
    unset_cred PostHog POSTHOG_BILLING_READ_KEY "standards/docs/runbooks/posthog-sign-up.md"
    return 0
  fi
  local out amount
  if ! out=$(get "$POSTHOG_API_HOST/api/billing/" "Authorization: Bearer $POSTHOG_BILLING_READ_KEY"); then
    problem PostHog "$out"
    return 0
  fi
  amount=$(jq -r '(.current_total_amount_usd // "0") | tostring | tonumber? // 0' <<<"$out")
  if [ "$(jq -r '.has_active_subscription // false' <<<"$out")" = true ]; then
    problem PostHog "an active paid subscription (plan $(jq -r '.billing_plan // .subscription_level // "?"' <<<"$out")); adding a card starts one, so remove it"
  elif jq -e --argjson a "$amount" -n '$a > 0' >/dev/null; then
    problem PostHog "\$$amount billed this period"
  else
    row PostHog "no active subscription, \$0 this period (plan $(jq -r '.subscription_level // .billing_plan // "free"' <<<"$out"))"
  fi
}

what=${1:-all}
case "$what" in
  vendors) echo "$KNOWN"; exit 0 ;;
  all | cloudflare | github | posthog) ;;
  *) echo "usage: billing-check.sh [all|cloudflare|github|posthog|vendors]" >&2; exit 2 ;;
esac
{ echo "### Billing check"; echo; echo "| Vendor | Result |"; echo "| --- | --- |"; } >>"$summary"
if [ "$what" = all ]; then
  for api in $(jq -r '[.[] | select(.kind == "vendor") | .billing.api[]?] | unique | .[]' "$RADAR"); do
    case " $KNOWN " in *" $api "*) "$api" ;; *) problem radar "billing.api $api has no check here" ;; esac
  done
else
  "$what"
fi
if [ "$what" = all ]; then
  {
    echo
    echo "Every vendor in the tech radar ([radar.json](../../standards/radar/radar.json)):"
    echo
    echo "| Vendor | Ring | Checked by API | Jim checks |"
    echo "| --- | --- | --- | --- |"
    jq -r '.[] | select(.kind == "vendor")
      | "| \(.name) | \(.ring) | \(if (.billing.api // []) == [] then "-" else (.billing.api | join(", ")) end) | \(
          if (.billing.human // "") != "" then "`\(.billing.human)` (standards/settings.toml)"
          elif (.billing.api // []) == [] then "no account" else "-" end) |"' "$RADAR"
  } >>"$summary"
fi

if [ "$problems" -gt 0 ]; then
  echo "billing check: $problems problem(s)"
  exit 1
fi
echo "billing check: ok"
