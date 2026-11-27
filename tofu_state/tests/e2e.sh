#!/usr/bin/env bash
# End-to-end test of the state Worker with real OpenTofu (decision 0009):
# `wrangler dev` serves the built Worker with a local D1, and `tofu` uses it
# as its http backend, with the same backend settings infra synthesizes.
#
# Needs: the Worker built (`worker-build --release`), wrangler, tofu, curl,
# jq. Uses only OpenTofu's built-in `terraform_data`, so no provider
# download.
set -euo pipefail

here=$(cd "$(dirname "$0")/.." && pwd)
work=$(mktemp -d)
port=${TOFU_STATE_E2E_PORT:-8799}
origin="http://127.0.0.1:$port"
read_pw="e2e-read-$RANDOM$RANDOM"
write_pw="e2e-write-$RANDOM$RANDOM"
dev_pid=""

cleanup() {
  [ -z "$dev_pid" ] || kill "$dev_pid" 2>/dev/null || true
  wait 2>/dev/null || true
  rm -rf "$work"
}
trap cleanup EXIT

fail() { echo "e2e: FAIL: $*" >&2; [ ! -f "$work/dev.log" ] || tail -n 30 "$work/dev.log" >&2; exit 1; }
pass() { echo "e2e: ok: $*"; }

cd "$here"
[ -f build/worker/shim.mjs ] || fail "build the Worker first (worker-build --release)"

wrangler d1 migrations apply DB --local --persist-to "$work/d1" >"$work/migrate.log" 2>&1 ||
  { cat "$work/migrate.log" >&2; fail "D1 migrations"; }

# Secrets as --var, so nothing is written to .dev.vars in the project.
wrangler dev --local --ip 127.0.0.1 --port "$port" --persist-to "$work/d1" \
  --var "TOFU_STATE_READ_PASSWORD:$read_pw" --var "TOFU_STATE_WRITE_PASSWORD:$write_pw" \
  >"$work/dev.log" 2>&1 &
dev_pid=$!
for _ in $(seq 1 60); do
  code=$(curl -s -o /dev/null -w '%{http_code}' "$origin/" || true)
  [ "$code" = "404" ] && break
  kill -0 "$dev_pid" 2>/dev/null || fail "wrangler dev exited"
  sleep 1
done
[ "$code" = "404" ] || fail "wrangler dev did not start"

d1() {
  wrangler d1 execute DB --local --persist-to "$work/d1" --json --command "$1" 2>/dev/null
}

# A stack like infra's: the http backend with LOCK and UNLOCK, and enforced
# state encryption, plus a resource whose apply takes a few seconds so two
# applies overlap.
stack="$work/stack"
mkdir -p "$stack"
cat >"$stack/main.tf.json" <<'EOF'
{
  "terraform": {
    "backend": {
      "http": {
        "lock_method": "LOCK",
        "unlock_method": "UNLOCK",
        "update_method": "POST",
        "username": "tofu",
        "retry_max": 2
      }
    },
    "encryption": {
      "key_provider": { "pbkdf2": { "state": { "passphrase": "${var.tofu_state_passphrase}" } } },
      "method": { "aes_gcm": { "state": { "keys": "key_provider.pbkdf2.state" } } },
      "state": { "method": "method.aes_gcm.state", "enforced": true },
      "plan": { "method": "method.aes_gcm.state", "enforced": true }
    }
  },
  "variable": {
    "tofu_state_passphrase": { "type": "string", "sensitive": true },
    "generation": { "type": "number", "default": 0 },
    "pause": { "type": "number", "default": 0 }
  },
  "resource": {
    "terraform_data": {
      "marker": {
        "input": "e2e-plaintext-marker",
        "triggers_replace": "${var.generation}",
        "provisioner": { "local-exec": { "command": "sleep ${var.pause}" } }
      }
    }
  }
}
EOF

export TF_VAR_tofu_state_passphrase="e2e passphrase that is long enough"
export TF_IN_AUTOMATION=1
tofu_in() { (cd "$stack" && tofu "$@" -no-color); }

addr="$origin/states/e2e"
TF_HTTP_PASSWORD=$write_pw tofu_in init -input=false \
  "-backend-config=address=$addr" "-backend-config=lock_address=$addr" \
  "-backend-config=unlock_address=$addr" >"$work/init.log" 2>&1 ||
  { cat "$work/init.log" >&2; fail "tofu init"; }
pass "tofu init against the Worker"

TF_HTTP_PASSWORD=$write_pw tofu_in apply -input=false -auto-approve >"$work/apply.log" 2>&1 ||
  { cat "$work/apply.log" >&2; fail "tofu apply"; }
pass "tofu apply stores state"

body=$(curl -sf -u "tofu:$read_pw" "$addr") || fail "GET with the read password"
jq -e '.encrypted_data' >/dev/null <<<"$body" || fail "stored state is not encrypted"
grep -q 'e2e-plaintext-marker' <<<"$body" && fail "plaintext reached the Worker"
pass "the Worker only sees ciphertext"

TF_HTTP_PASSWORD=$read_pw tofu_in plan -input=false -lock=false -detailed-exitcode >"$work/plan.log" 2>&1 ||
  { cat "$work/plan.log" >&2; fail "plan with the read password"; }
pass "the read password plans with -lock=false and sees no changes"

if TF_HTTP_PASSWORD=$read_pw tofu_in apply -input=false -auto-approve -var generation=9 >"$work/ro.log" 2>&1; then
  fail "apply with the read password succeeded"
fi
pass "the read password can't apply"

if TF_HTTP_PASSWORD=wrong tofu_in plan -input=false -lock=false >"$work/bad.log" 2>&1; then
  fail "plan with a wrong password succeeded"
fi
pass "a wrong password is refused"

# Two applies at once: the first holds the lock while its provisioner
# sleeps, so the second must fail on the lock and name the holder.
TF_HTTP_PASSWORD=$write_pw tofu_in apply -input=false -auto-approve -var generation=1 -var pause=8 \
  >"$work/first.log" 2>&1 &
first=$!
for _ in $(seq 1 30); do
  [ "$(d1 'SELECT COUNT(*) AS n FROM locks' | jq '.[0].results[0].n')" = "1" ] && break
  sleep 0.5
done
if TF_HTTP_PASSWORD=$write_pw tofu_in apply -input=false -auto-approve -var generation=2 -lock-timeout=0s \
  >"$work/second.log" 2>&1; then
  fail "a second apply ran while the first held the lock"
fi
grep -q 'Error acquiring the state lock' "$work/second.log" || { cat "$work/second.log" >&2; fail "no lock error"; }
grep -q 'OperationTypeApply' "$work/second.log" || { cat "$work/second.log" >&2; fail "lock error without the holder's info"; }
wait "$first" || { cat "$work/first.log" >&2; fail "the first apply failed"; }
[ "$(d1 'SELECT COUNT(*) AS n FROM locks' | jq '.[0].results[0].n')" = "0" ] || fail "lock not released"
pass "concurrent applies: one waits on the lock, which is released after"

for g in $(seq 3 14); do
  TF_HTTP_PASSWORD=$write_pw tofu_in apply -input=false -auto-approve -var "generation=$g" >"$work/many.log" 2>&1 ||
    { cat "$work/many.log" >&2; fail "apply $g"; }
done
n=$(d1 "SELECT COUNT(*) AS n FROM states WHERE stack = 'e2e'" | jq '.[0].results[0].n')
[ "$n" = "10" ] || fail "kept $n versions, not 10"
pass "only the last 10 versions are kept"

TOFU_STATE_READ_PASSWORD=$read_pw bash scripts/smoke.sh "$origin" >"$work/smoke.log" 2>&1 ||
  { cat "$work/smoke.log" >&2; fail "smoke check"; }
pass "the deploy smoke check passes against the Worker"

echo "e2e: all passed"
