#!/usr/bin/env bash
# End-to-end check of pane's Worker: builds it with worker-build, serves it
# with `wrangler dev` over a stand-in assets folder (the real Dioxus build
# takes minutes and is checked by pane/web's own CI), and checks that pages
# and data come from the assets, client-side routes get index.html, and only
# /api/* reaches the Worker. Run by `mise run ci` (cloudflare-worker-assets.toml).
set -euo pipefail
cd "$(dirname "$0")/.."

port=${E2E_PORT:-8799}
assets=$(mktemp -d)
log=$(mktemp)
mkdir -p "$assets/data"
echo '<!doctype html><title>pane e2e</title>' > "$assets/index.html"
echo '{"e2e":true}' > "$assets/data/snapshot.json"

worker-build --release >/dev/null

wrangler dev --assets "$assets" --port "$port" --ip 127.0.0.1 >"$log" 2>&1 &
pid=$!
cleanup() { kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true; rm -rf "$assets" "$log"; }
trap cleanup EXIT

for _ in $(seq 1 60); do
  curl -sf -o /dev/null "http://127.0.0.1:$port/" && break
  sleep 1
done

failures=0
check() { # method path status body-substring [extra curl args]
  local method=$1 path=$2 want_status=$3 want_body=$4
  shift 4
  local out status body
  out=$(curl -s -X "$method" -w '\n%{http_code}' "$@" "http://127.0.0.1:$port$path")
  status=${out##*$'\n'}
  body=${out%$'\n'*}
  if [ "$status" != "$want_status" ] || [[ "$body" != *"$want_body"* ]]; then
    echo "FAIL $method $path: got $status '$body', want $want_status '$want_body'"
    failures=$((failures + 1))
  else
    echo "ok   $method $path -> $status"
  fi
}

check GET / 200 'pane e2e'
check GET /data/snapshot.json 200 '"e2e":true'
check GET /projects/pane 200 'pane e2e' -H 'Sec-Fetch-Mode: navigate'
check GET /api/health 200 '{"status":"ok"}'
check POST /api/health 405 'use GET'
check GET /api/nope 404 'no such API route'

if [ "$failures" -gt 0 ]; then
  echo "e2e: $failures check(s) failed; wrangler dev log:"
  cat "$log"
  exit 1
fi
echo "e2e: ok"
