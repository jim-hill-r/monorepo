#!/usr/bin/env bash
# Tests standards/scripts/threat-live.sh: it lists the account into
# account.json, requests each URL `threat live` names into probes.json,
# reports, and fails on findings. `curl` and `cargo` are stubs, so no
# network or build is needed. Run from anywhere:
#   bash standards/tests/threat-live.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0
ok() { echo "ok: $*"; }
fail() { echo "FAIL: $*"; failures=$((failures + 1)); }

mkdir -p "$work/repo/standards/scripts" "$work/bin"
cp "$root/standards/scripts/threat-live.sh" "$work/repo/standards/scripts/"

# curl: the Cloudflare API from canned bodies (WORKERS=fail makes the
# Workers list fail), and the probed sites by URL. The token must come on
# stdin, never as an argument.
cat >"$work/bin/curl" <<'STUB'
#!/usr/bin/env bash
url="" headers=""
args=("$@")
for ((i = 0; i < ${#args[@]}; i++)); do
  case "${args[$i]}" in
    http*) url=${args[$i]} ;;
    -D) headers=${args[$((i + 1))]} ;;
  esac
done
echo "$*" >>"$CALLS"
case "$*" in *secret-token*) echo "token on the command line" >&2; exit 99 ;; esac
case "$url" in
  */pages/projects\?*)
    cat >/dev/null
    echo '{"success": true, "result": [{"name": "jimhillr-app", "subdomain": "jimhillr-app.pages.dev", "domains": ["app.example.dev"]}, {"name": "pane-dbc", "subdomain": "pane-dbc.pages.dev"}]}' ;;
  */pages/projects/jimhillr-app/deployments*)
    cat >/dev/null
    echo '{"success": true, "result": [{"url": "https://abc.jimhillr-app.pages.dev"}]}' ;;
  */pages/projects/pane-dbc/deployments*)
    cat >/dev/null
    echo '{"success": true, "result": []}' ;;
  */workers/scripts)
    cat >/dev/null
    [ "${WORKERS:-}" = fail ] && exit 22
    echo '{"success": true, "result": [{"id": "jimhillr-telemetry"}]}' ;;
  https://jimhillr-app.pages.dev)
    printf 'HTTP/2 200\r\nContent-Security-Policy: default-src x\r\nX-Content-Type-Options: nosniff\r\n\r\n' >"$headers"
    printf 200 ;;
  https://abc.jimhillr-app.pages.dev)
    printf 'HTTP/2 302\r\nlocation: https://t.cloudflareaccess.com/login\r\n\r\n' >"$headers"
    printf 302 ;;
  https://down.example.dev)
    exit 6 ;;
  *) echo "curl stub: unexpected $url" >&2; exit 22 ;;
esac
STUB
# cargo: `threat live` without --probes lists URLs; with them it copies
# probes.json next to the test and exits $LIVE_STATUS.
cat >"$work/bin/cargo" <<'STUB'
#!/usr/bin/env bash
echo "cargo $*" >>"$CALLS"
case "$*" in
  *--probes*)
    for ((i = 1; i <= $#; i++)); do [ "${!i}" = --probes ] && { j=$((i + 1)); cp "${!j}" "$SAVE/probes.json"; }; done
    for ((i = 1; i <= $#; i++)); do [ "${!i}" = --account ] && { j=$((i + 1)); cp "${!j}" "$SAVE/account.json"; }; done
    echo "## Live threat check"
    exit "${LIVE_STATUS:-0}" ;;
  *) printf 'https://jimhillr-app.pages.dev\nhttps://abc.jimhillr-app.pages.dev\nhttps://down.example.dev\n' ;;
esac
STUB
chmod +x "$work/bin/curl" "$work/bin/cargo"

run() { # run [VAR=value ...]
  : >"$work/calls"
  rm -rf "$work/save" "$work/out"
  mkdir -p "$work/save"
  out=$(env CALLS="$work/calls" SAVE="$work/save" THREAT_LIVE_OUT="$work/out" \
    GITHUB_STEP_SUMMARY="$work/summary" PATH="$work/bin:$PATH" "$@" \
    bash "$work/repo/standards/scripts/threat-live.sh" 2>&1)
  status=$?
}

run CLOUDFLARE_API_TOKEN=secret-token CLOUDFLARE_ACCOUNT_ID=acct
if [ "$status" -eq 0 ]; then ok "clean run exits 0"; else fail "clean run exited $status: $out"; fi
account=$(cat "$work/save/account.json" 2>/dev/null)
if [ "$(jq -c '.pages[0]' <<<"$account")" = '{"name":"jimhillr-app","domains":["app.example.dev","jimhillr-app.pages.dev"],"preview_url":"https://abc.jimhillr-app.pages.dev"}' ]; then
  ok "lists Pages projects with domains and the newest preview"
else
  fail "account.json pages: $account"
fi
if [ "$(jq -c '.pages[1].preview_url, .workers' <<<"$account" | tr '\n' ' ')" = 'null ["jimhillr-telemetry"] ' ]; then
  ok "a project without previews, and the Workers"
else
  fail "account.json: $account"
fi
probes=$(cat "$work/save/probes.json" 2>/dev/null)
if [ "$(jq -c '.[0] | [.status, .headers["x-content-type-options"], .location]' <<<"$probes")" = '[200,"nosniff",null]' ] &&
  [ "$(jq -c '.[1] | [.status, .location]' <<<"$probes")" = '[302,"https://t.cloudflareaccess.com/login"]' ] &&
  [ "$(jq -c '.[2] | [.status, .headers]' <<<"$probes")" = '[0,{}]' ]; then
  ok "records status, lowercase headers and location, and 0 for a failed request"
else
  fail "probes.json: $probes"
fi
if grep -q -- "--issues-dir $work/out" "$work/calls"; then ok "writes orphan issues to the output dir"; else fail "no --issues-dir: $(cat "$work/calls")"; fi
if grep -q "jimhillr-threat-live" "$work/calls" && grep -q "x-jimhillr-synthetic: 1" "$work/calls"; then
  ok "probes say who they are"
else
  fail "user agent or synthetic header missing"
fi
if grep -q "## Live threat check" "$work/summary"; then ok "report in the job summary"; else fail "summary: $(cat "$work/summary")"; fi

rm -f "$work/summary"
run CLOUDFLARE_API_TOKEN=secret-token CLOUDFLARE_ACCOUNT_ID=acct LIVE_STATUS=1
if [ "$status" -eq 1 ] && grep -q "::error::live threat check: deployed sites" <<<"$out"; then
  ok "findings fail the run"
else
  fail "findings: exit $status: $out"
fi

run CLOUDFLARE_API_TOKEN=secret-token CLOUDFLARE_ACCOUNT_ID=acct WORKERS=fail
if [ "$status" -eq 0 ] && [ "$(jq -c '.workers' "$work/save/account.json")" = null ] && grep -q "can't list Workers" <<<"$out"; then
  ok "a token that can't list Workers is a warning"
else
  fail "workers fail: exit $status: $out"
fi

run CLOUDFLARE_API_TOKEN= CLOUDFLARE_ACCOUNT_ID=
if [ "$status" -eq 0 ] && grep -q "skipped" <<<"$out" && [ ! -s "$work/calls" ]; then
  ok "no token: skipped, no calls"
else
  fail "no token: exit $status: $out"
fi

if [ "$failures" -gt 0 ]; then
  echo "threat-live: $failures failure(s)"
  exit 1
fi
echo "threat-live: all cases passed"
