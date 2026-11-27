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
  echo "  AUTH0_DOMAIN=\${AUTH0_DOMAIN:-} AUTH0_CLIENT_ID=\${AUTH0_CLIENT_ID:-} CLOUDFLARE_API_TOKEN=\${CLOUDFLARE_API_TOKEN:-} GITHUB_TOKEN=\${GITHUB_TOKEN:-}"
  echo "  TF_VAR_tofu_state_passphrase=\${TF_VAR_tofu_state_passphrase:-} TF_HTTP_PASSWORD=\${TF_HTTP_PASSWORD:-}"
} >>"$work/tofu.log"
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
chmod +x "$work/bin/tofu" "$work/bin/cargo"
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

# github: the read App's token for plans, the infra App's for credential
# applies, and no everyday apply tier (decision 0010).
code=0
base_env bash "$script" ready github read >/dev/null || code=$?
[ "$code" = 3 ] && ok "github needs its read token" || fail "github ready without a token: code $code"
: >"$work/tofu.log"
base_env GH_INFRA_READ_TOKEN=gh-read bash "$script" plan github read "$work/gh" >/dev/null 2>&1 || fail "github plan"
grep -q "GITHUB_TOKEN=gh-read" "$work/tofu.log" && ok "github plans read with the read App's token" || fail "github read token: $(cat "$work/tofu.log")"
: >"$work/tofu.log"
base_env GH_INFRA_CREDENTIALS_TOKEN=gh-write bash "$script" apply github credentials "$work/gh" >/dev/null 2>&1 || fail "github apply"
grep -q "GITHUB_TOKEN=gh-write" "$work/tofu.log" && ok "github applies with the infra App's token" || fail "github write token"
code=0
out=$(base_env GH_INFRA_READ_TOKEN=gh-read bash "$script" apply github apply "$work/gh" 2>&1) || code=$?
[ "$code" != 0 ] && grep -q "only in infra-credentials" <<<"$out" && ok "github has no everyday apply" || fail "github apply role: code $code, $out"

# Bad input.
base_env bash "$script" plan "../etc" read "$work/x" >/dev/null 2>&1 && fail "bad stack accepted" || ok "bad stack names are refused"
base_env bash "$script" plan auth0 admin "$work/x" >/dev/null 2>&1 && fail "bad role accepted" || ok "unknown roles are refused"

if [ "$failures" -gt 0 ]; then
  echo "tofu.sh tests: $failures failed"
  exit 1
fi
echo "tofu.sh tests: all passed"
