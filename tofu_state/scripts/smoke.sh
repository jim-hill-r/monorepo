#!/usr/bin/env bash
# Check a deployed state Worker: no password gets 401, the read password
# reads every stack infra synthesizes (200 or 204), and the read password
# can't take a lock (403). Never prints a state body.
#
#   smoke.sh ORIGIN
# Needs TOFU_STATE_READ_PASSWORD.
set -euo pipefail
origin=${1:?usage: smoke.sh ORIGIN}
: "${TOFU_STATE_READ_PASSWORD:?TOFU_STATE_READ_PASSWORD is not set}"
root=$(git rev-parse --show-toplevel)
code() { curl -s -o /dev/null -w '%{http_code}' --max-time 20 "$@"; }

fail=0
for dir in "$root"/infra/out/*/; do
  stack=$(basename "$dir")
  url="$origin/states/$stack"
  c=$(code "$url")
  [ "$c" = 401 ] || { echo "smoke: $stack without a password got $c, want 401"; fail=1; }
  c=$(code -u "tofu:$TOFU_STATE_READ_PASSWORD" "$url")
  [[ "$c" = 200 || "$c" = 204 ]] || { echo "smoke: $stack with the read password got $c, want 200 or 204"; fail=1; }
  c=$(code -u "tofu:$TOFU_STATE_READ_PASSWORD" -X LOCK -d '{"ID":"smoke"}' "$url")
  [ "$c" = 403 ] || { echo "smoke: LOCK on $stack with the read password got $c, want 403"; fail=1; }
  [ "$fail" -ne 0 ] || echo "smoke: ok: $stack"
done
exit "$fail"
