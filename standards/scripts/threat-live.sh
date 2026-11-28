#!/usr/bin/env bash
# The weekly live threat check ("The weekly live check" in
# standards/docs/plans/2026-11-threat-check.md, and "Live check" in
# standards/docs/security.md). The `threat-live` job of All Free Tier Usage
# runs it every Monday with the same read-only Cloudflare token as the usage
# check, so it needs no new secret.
#
#   threat-live.sh
#
# 1. Lists every Pages project (with its domains and newest preview URL) and
#    every Worker on the account into account.json.
# 2. Asks `threat live` which URLs to request: each reachable web surface's
#    `url` in every docs/threat.toml, and one preview per Pages project.
# 3. Requests each once, without following redirects, into probes.json.
# 4. `threat live` compares both with the threat models: a Pages project or
#    Worker no surface claims (an orphan such as pane-dbc.pages.dev), a
#    `cloudflare-access` surface that doesn't redirect to Access, an API that
#    answers without a token, a public site missing a security header.
#
# The report goes to the job summary, and an issue file for each orphan into
# $THREAT_LIVE_OUT/threat/docs/issues/ (uploaded as the run's artifact) for PR triage to
# commit (standards/docs/repo-health.md). Exit 1 on any finding, or when a
# call fails. Without the token it warns and exits 0. It only reads.
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
CF_API=${CF_API:-https://api.cloudflare.com/client/v4}
out=${THREAT_LIVE_OUT:-$(mktemp -d)}
summary=${GITHUB_STEP_SUMMARY:-/dev/null}
mkdir -p "$out"

if [ -z "${CLOUDFLARE_API_TOKEN:-}" ] || [ -z "${CLOUDFLARE_ACCOUNT_ID:-}" ]; then
  echo "::warning::live threat check skipped: CLOUDFLARE_API_TOKEN or CLOUDFLARE_ACCOUNT_ID isn't set"
  echo "Live threat check skipped: no Cloudflare token." >>"$summary"
  exit 0
fi

die() { echo "::error::live threat check: $*"; echo "Live threat check failed: $*" >>"$summary"; exit 1; }

# cf PATH: the API response body, or nothing and status 1. The token goes
# through stdin so it is never on a command line.
cf() {
  printf 'header = "Authorization: Bearer %s"\n' "$CLOUDFLARE_API_TOKEN" |
    curl -fsS -K - -A jimhillr-threat-live "$CF_API/accounts/$CLOUDFLARE_ACCOUNT_ID/$1"
}

threat() { (cd "$root" && cargo run --quiet --manifest-path threat/Cargo.toml -p threat_cli -- "$@"); }

# 1. The account.
pages='[]'
page=1
while :; do
  body=$(cf "pages/projects?per_page=10&page=$page") || die "could not list Pages projects"
  jq -e '.success == true' >/dev/null <<<"$body" || die "Pages projects: $(jq -c '.errors' <<<"$body")"
  while IFS=$'\t' read -r name domains; do
    [ -n "$name" ] || continue
    # The newest preview deployment, if any; one missing is not a failure.
    preview=$(cf "pages/projects/$name/deployments?env=preview&per_page=1" 2>/dev/null |
      jq -r 'if .success == true then (.result[0].url // "") else "" end' 2>/dev/null || true)
    pages=$(jq -nc --argjson acc "$pages" --arg n "$name" --argjson d "$domains" --arg p "$preview" \
      '$acc + [{name: $n, domains: $d, preview_url: (if $p == "" then null else $p end)}]')
  done < <(jq -r '.result[] | [.name, ((.domains // []) + [.subdomain // empty] | unique | tojson)] | @tsv' <<<"$body")
  [ "$(jq '.result | length' <<<"$body")" -ge 10 ] || break
  page=$((page + 1))
  [ "$page" -le 20 ] || break
done
# The deploy token may not have Workers Scripts read access; then Workers
# are not listed rather than failing the run (as free-tiers.sh does).
if body=$(cf "workers/scripts" 2>/dev/null) && jq -e '.success == true' >/dev/null <<<"$body"; then
  workers=$(jq -c '[.result[].id]' <<<"$body")
else
  echo "::warning::live threat check: the token can't list Workers (needs Workers Scripts: Read)"
  workers=null
fi
jq -n --argjson p "$pages" --argjson w "$workers" '{pages: $p, workers: $w}' >"$out/account.json"

# 2 and 3. The URLs, requested once each.
urls=$(threat live --account "$out/account.json") || die "threat live could not list the URLs"
probes='[]'
while IFS= read -r url; do
  [ -n "$url" ] || continue
  code=$(curl -sS -o /dev/null -D "$out/headers" -w '%{http_code}' --max-time 20 --proto '=https' \
    -A jimhillr-threat-live -H 'x-jimhillr-synthetic: 1' "$url" 2>/dev/null) || code=0
  [ -f "$out/headers" ] || : >"$out/headers"
  probes=$(jq -c --arg url "$url" --argjson status "${code:-0}" --argjson acc "$probes" -R -s '
    [split("\n")[] | sub("\r$"; "") | select(test("^[A-Za-z0-9-]+:"))
      | capture("^(?<k>[^:]+):\\s*(?<v>.*)$") | {key: (.k | ascii_downcase), value: .v}]
    | from_entries as $h
    | $acc + [{url: $url, status: $status, location: ($h.location // null), headers: $h}]' <"$out/headers")
  rm -f "$out/headers"
done <<<"$urls"
printf '%s\n' "$probes" >"$out/probes.json"

# 4. The report.
status=0
threat live --account "$out/account.json" --probes "$out/probes.json" --issues-dir "$out" >"$out/report.md" || status=$?
cat "$out/report.md"
cat "$out/report.md" >>"$summary"
case $status in
  0) ;;
  1) echo "::error::live threat check: deployed sites don't match their threat models; see the run summary" ;;
  *) die "threat live failed (exit $status)" ;;
esac
exit "$status"
