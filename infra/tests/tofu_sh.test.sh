#!/usr/bin/env bash
# Tests for infra/scripts/tofu.sh with stub `tofu` and `cargo` on PATH: they
# log their arguments and the credentials they were given, so the tests can
# check which secret reaches which provider variable, the lock flag, and
# that apply refuses a plan whose hash doesn't match.
set -euo pipefail

here=$(cd "$(dirname "$0")/.." && pwd)
script="$here/scripts/tofu.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0
fail() { echo "FAIL: $*"; failures=$((failures + 1)); }
ok() { echo "ok: $*"; }

mkdir -p "$work/bin"
cat >"$work/bin/tofu" <<EOF
#!/usr/bin/env bash
{
  echo "tofu \$*"
  echo "  AUTH0_DOMAIN=\${AUTH0_DOMAIN:-} AUTH0_CLIENT_ID=\${AUTH0_CLIENT_ID:-} CLOUDFLARE_API_TOKEN=\${CLOUDFLARE_API_TOKEN:-}"
  echo "  TF_VAR_tofu_state_passphrase=\${TF_VAR_tofu_state_passphrase:-} TF_HTTP_PASSWORD=\${TF_HTTP_PASSWORD:-}"
  echo "  TF_VAR_tofu_state_passphrase_old=\${TF_VAR_tofu_state_passphrase_old-unset} TF_ENCRYPTION=\${TF_ENCRYPTION-unset}"
} >>"$work/tofu.log"
case "\$1 \${2:-}" in
  "state pull") echo '{"plain":"state"}' ;;
  "state push") echo "  pushed: \$(cat)" >>"$work/tofu.log" ;;
esac
case "\$1" in
  plan)
    for a in "\$@"; do
      case "\$a" in -out=*) echo "encrypted plan" >"\${a#-out=}" ;; esac
    done
    exit "\${STUB_PLAN_EXIT:-2}" ;;
  show)
    if [ "\$2" = "-json" ]; then echo '{"resource_changes":[]}'; else echo "  # auth0_client.cookbook will be updated in-place"; fi ;;
esac
EOF
cat >"$work/bin/cargo" <<EOF
#!/usr/bin/env bash
# cargo run --quiet -- <subcommand> ...
shift 3
case "\$1" in
  encryption)
    if [ "\$2" -gt 1 ]; then echo "{\"state\":{\"method\":\"method.aes_gcm.gen_\$2\"}}"; else echo '{"state":{"method":"method.aes_gcm.state"}}'; fi ;;
  backend-config)
    echo "-backend-config=address=\$3/states/\$2" ;;
  guard)
    report=""
    while [ \$# -gt 0 ]; do [ "\$1" = "--report" ] && report=\$2; shift; done
    echo "{\"credential_change\": \${STUB_CREDENTIAL:-false}, \"changes\": []}" >"\$report"
    echo "guard: ok"
    exit "\${STUB_GUARD_EXIT:-0}" ;;
esac
EOF
# curl: the raw state from the Worker, as STUB_STATE (empty: no state yet).
cat >"$work/bin/curl" <<EOF
#!/usr/bin/env bash
{ echo "curl \$*"; echo "  config: \$(cat)"; } >>"$work/curl.log"
printf '%s' "\${STUB_STATE:-}"
EOF
chmod +x "$work/bin/tofu" "$work/bin/cargo" "$work/bin/curl"
export PATH="$work/bin:$PATH"

base_env() {
  env -i PATH="$PATH" HOME="$HOME" \
    TOFU_STATE_ORIGIN=https://state.example.com TF_HTTP_PASSWORD=state-pw TOFU_STATE_PASSPHRASE=pass \
    AUTH0_INFRA_READ_DOMAIN=read.auth0 AUTH0_INFRA_READ_CLIENT_ID=read-id AUTH0_INFRA_READ_CLIENT_SECRET=read-secret \
    AUTH0_INFRA_APPLY_DOMAIN=apply.auth0 AUTH0_INFRA_APPLY_CLIENT_ID=apply-id AUTH0_INFRA_APPLY_CLIENT_SECRET=apply-secret \
    GITHUB_STEP_SUMMARY="$work/summary.md" "$@"
}

# ready: names what is missing, with exit 3.
code=0
out=$(env -i PATH="$PATH" TOFU_STATE_ORIGIN=x bash "$script" ready auth0 read) || code=$?
[ "$code" = 3 ] && grep -q AUTH0_INFRA_READ_CLIENT_SECRET <<<"$out" && grep -q TOFU_STATE_PASSPHRASE <<<"$out" &&
  ok "ready names the missing secrets" || fail "ready: code $code, output: $out"
base_env bash "$script" ready auth0 read >/dev/null && ok "ready passes when set" || fail "ready with everything set"
code=0
base_env bash "$script" ready auth0 credentials >/dev/null || code=$?
[ "$code" = 3 ] && ok "credentials role needs its own secrets" || fail "credentials role: code $code"

# plan with the read role: read credentials, no lock for a PR plan.
: >"$work/tofu.log"
base_env TOFU_LOCK=false bash "$script" plan auth0 read "$work/plan" >/dev/null 2>&1 || fail "plan exited non-zero"
grep -q "AUTH0_DOMAIN=read.auth0 AUTH0_CLIENT_ID=read-id" "$work/tofu.log" && ok "read role maps the read secrets" || fail "read secrets: $(cat "$work/tofu.log")"
grep -q -- "-lock=false" "$work/tofu.log" && ok "PR plans don't lock" || fail "no -lock=false"
grep -q "TF_VAR_tofu_state_passphrase=pass" "$work/tofu.log" && ok "passphrase reaches tofu" || fail "passphrase"
grep -q -- "init .*-backend-config=address=https://state.example.com/states/auth0" "$work/tofu.log" && ok "init uses the stack's state URL" || fail "init args"
[ -f "$work/plan/plan.tfplan" ] && [ -f "$work/plan/plan.sha256" ] && [ -f "$work/plan/report.json" ] && ok "plan writes the plan, its hash and the report" || fail "plan outputs"
[ ! -e "$work/plan/plan.json" ] && ok "the JSON plan isn't kept" || fail "plan.json left behind"
grep -q "changes=yes" "$work/plan/result" && grep -q "credential_change=no" "$work/plan/result" && ok "result records changes" || fail "result: $(cat "$work/plan/result")"
grep -q "Plan SHA-256" "$work/summary.md" && ok "summary has the plan hash" || fail "summary"

# A credential change is flagged.
base_env STUB_CREDENTIAL=true bash "$script" plan auth0 read "$work/cred" >/dev/null 2>&1 || fail "credential plan"
grep -q "credential_change=yes" "$work/cred/result" && grep -q "Credential change" "$work/summary.md" && ok "credential changes are flagged" || fail "credential flag"

# Failing guards fail the plan.
code=0
base_env STUB_GUARD_EXIT=1 bash "$script" plan auth0 read "$work/bad" >/dev/null 2>&1 || code=$?
[ "$code" != 0 ] && ok "failing guards fail the plan" || fail "guard failure ignored"

# Plans lock by default (main).
: >"$work/tofu.log"
base_env bash "$script" plan auth0 read "$work/main" >/dev/null 2>&1 || fail "main plan"
grep -q -- "-lock=true" "$work/tofu.log" && ok "plans on main lock" || fail "main plan lock"

# apply: apply credentials, exactly the saved plan, refuses a changed plan.
: >"$work/tofu.log"
base_env bash "$script" apply auth0 apply "$work/main" >/dev/null 2>&1 || fail "apply exited non-zero"
grep -q "AUTH0_DOMAIN=apply.auth0" "$work/tofu.log" && ok "apply role maps the apply secrets" || fail "apply secrets"
grep -q "tofu apply -no-color -input=false $work/main/plan.tfplan" "$work/tofu.log" && ok "apply applies the saved plan" || fail "apply args: $(cat "$work/tofu.log")"
echo tampered >>"$work/main/plan.tfplan"
: >"$work/tofu.log"
code=0
base_env bash "$script" apply auth0 apply "$work/main" >/dev/null 2>&1 || code=$?
[ "$code" != 0 ] && ! grep -q "tofu apply" "$work/tofu.log" && ok "apply refuses a plan whose hash changed" || fail "tampered plan applied"

# drift: exit 2 on drift, refresh-only without a lock.
code=0
base_env STUB_PLAN_EXIT=2 bash "$script" drift auth0 >/dev/null 2>&1 || code=$?
[ "$code" = 2 ] && grep -q -- "-refresh-only -lock=false" "$work/tofu.log" && ok "drift exits 2 on drift" || fail "drift: code $code"
base_env STUB_PLAN_EXIT=0 bash "$script" drift auth0 >/dev/null 2>&1 && ok "no drift passes" || fail "no drift"

# Passphrase generations: 1 sets no TF_ENCRYPTION; later ones pass the
# generation's block and the old passphrase.
: >"$work/tofu.log"
base_env bash "$script" plan auth0 read "$work/gen1" >/dev/null 2>&1 || fail "generation 1 plan"
grep -q "TF_VAR_tofu_state_passphrase_old=unset TF_ENCRYPTION=unset" "$work/tofu.log" &&
  ok "generation 1 uses the committed encryption block" || fail "generation 1: $(cat "$work/tofu.log")"
: >"$work/tofu.log"
base_env TOFU_STATE_KEY_GEN=3 TOFU_STATE_PASSPHRASE_OLD=old-pass bash "$script" plan auth0 read "$work/gen3" >/dev/null 2>&1 || fail "generation 3 plan"
grep -q 'TF_VAR_tofu_state_passphrase_old=old-pass TF_ENCRYPTION={"state":{"method":"method.aes_gcm.gen_3"}}' "$work/tofu.log" &&
  ok "a later generation passes its block and the old passphrase" || fail "generation 3: $(cat "$work/tofu.log")"
bad_gen=no
for gen in 0 x 1.5 -2; do
  base_env TOFU_STATE_KEY_GEN=$gen bash "$script" plan auth0 read "$work/badgen" >/dev/null 2>&1 && bad_gen=$gen
done
[ "$bad_gen" = no ] && ok "bad generations are refused" || fail "generation '$bad_gen' accepted"

# rekey: skips a stack with no state or already on the current key, and
# otherwise pipes state pull into state push.
: >"$work/tofu.log"
: >"$work/curl.log"
code=0
env -i PATH="$PATH" TOFU_STATE_ORIGIN=https://state.example.com bash "$script" rekey auth0 >/dev/null 2>&1 || code=$?
[ "$code" = 3 ] && ok "rekey names what is missing" || fail "rekey without secrets: code $code"
base_env TOFU_STATE_KEY_GEN=2 bash "$script" rekey auth0 >/dev/null 2>&1 || fail "rekey with no state"
grep -q "^curl .*https://state.example.com/states/auth0" "$work/curl.log" && grep -q 'config: user = "tofu:state-pw"' "$work/curl.log" &&
  ok "rekey reads the raw state with the password on stdin" || fail "rekey curl: $(cat "$work/curl.log")"
! grep "^curl " "$work/curl.log" | grep -q state-pw && ok "the password isn't a curl argument" || fail "password on the command line"
! grep -q "tofu state" "$work/tofu.log" && ok "no state, nothing to rekey" || fail "rekeyed an empty state"
base_env TOFU_STATE_KEY_GEN=2 STUB_STATE='{"meta":{"key_provider.pbkdf2.gen_2":"x"}}' bash "$script" rekey auth0 >/dev/null 2>&1 || fail "rekey current"
! grep -q "tofu state" "$work/tofu.log" && ok "state on the current key is left alone" || fail "rekeyed current state"
base_env TOFU_STATE_KEY_GEN=2 TOFU_STATE_PASSPHRASE_OLD=old-pass STUB_STATE='{"meta":{"key_provider.pbkdf2.state":"x"}}' \
  bash "$script" rekey auth0 >/dev/null 2>&1 || fail "rekey old"
grep -q "tofu state pull" "$work/tofu.log" && grep -q 'pushed: {"plain":"state"}' "$work/tofu.log" &&
  grep -q "TF_VAR_tofu_state_passphrase_old=old-pass" "$work/tofu.log" &&
  ok "state on the old key is pulled and pushed back" || fail "rekey: $(cat "$work/tofu.log")"
grep -q "Rekey: auth0: re-encrypted with key_provider.pbkdf2.gen_2" "$work/summary.md" && ok "rekey writes a summary" || fail "rekey summary"

# Bad input.
base_env bash "$script" plan "../etc" read "$work/x" >/dev/null 2>&1 && fail "bad stack accepted" || ok "bad stack names are refused"
base_env bash "$script" plan auth0 admin "$work/x" >/dev/null 2>&1 && fail "bad role accepted" || ok "unknown roles are refused"

if [ "$failures" -gt 0 ]; then
  echo "tofu.sh tests: $failures failed"
  exit 1
fi
echo "tofu.sh tests: all passed"
