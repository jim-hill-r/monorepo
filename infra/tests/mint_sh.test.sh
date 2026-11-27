#!/usr/bin/env bash
# Tests for infra/scripts/mint.sh with stub `gh`, `curl` and `keytool` on
# PATH. The stubs log what they were asked to do; `gh secret set` records the
# value it read from stdin, so the tests can check that each secret reaches
# the right place, only when missing or rotated, and never on a command line.
set -euo pipefail

here=$(cd "$(dirname "$0")/.." && pwd)
script="$here/scripts/mint.sh"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
failures=0
fail() { echo "FAIL: $*"; failures=$((failures + 1)); }
ok() { echo "ok: $*"; }

mkdir -p "$work/bin" "$work/gh"
# gh: secrets are files under $work/gh/<env>/<name>, variables under vars/.
cat >"$work/bin/gh" <<EOF
#!/usr/bin/env bash
echo "gh \$*" >>"$work/calls.log"
store="$work/gh"
env=repo
for ((i = 1; i <= \$#; i++)); do [ "\${!i}" = --env ] && { j=\$((i + 1)); env=\${!j}; }; done
case "\$1 \$2" in
  "secret list") mkdir -p "\$store/\$env"; ls "\$store/\$env" ;;
  "secret set") mkdir -p "\$store/\$env"; cat >"\$store/\$env/\$3" ;;
  "variable list") mkdir -p "\$store/vars"; ls "\$store/vars" ;;
  "variable set")
    mkdir -p "\$store/vars"
    for ((i = 1; i <= \$#; i++)); do [ "\${!i}" = --body ] && { j=\$((i + 1)); printf '%s' "\${!j}" >"\$store/vars/\$3"; }; done ;;
  *) echo "gh stub: unexpected \$*" >&2; exit 1 ;;
esac
exit 0
EOF
# curl: answers the Cloudflare and Auth0 calls mint.sh makes.
cat >"$work/bin/curl" <<EOF
#!/usr/bin/env bash
method=GET url="" body=""
while [ \$# -gt 0 ]; do
  case "\$1" in
    -X) method=\$2; shift ;;
    -H | -w) shift ;;
    --data-binary) body=\$(cat); shift ;;
    http*) url=\$1 ;;
  esac
  shift
done
echo "curl \$method \$url \$body" >>"$work/curl.log"
auth0() { printf '%s\n200' "\$1"; }
case "\$method \$url" in
  "GET https://cf/accounts/acct/tokens/permission_groups"*)
    echo '{"success":true,"result":[{"id":"p1","name":"Pages Write"},{"id":"w1","name":"Workers Scripts Write"},{"id":"d1","name":"D1 Write"},{"id":"a1","name":"Account Analytics Read"},{"id":"r1","name":"Account Web Analytics Read"},{"id":"e1","name":"Account Web Analytics Write"}]}' ;;
  "GET https://cf/accounts/acct/tokens?"*)
    echo '{"success":true,"result":[{"id":"tok-deploy","name":"jimhillr-deploy"}]}' ;;
  "POST https://cf/accounts/acct/tokens")
    name=\$(jq -r .name <<<"\$body"); echo "{\"success\":true,\"result\":{\"value\":\"cf-value-\$name\"}}" ;;
  "PUT https://cf/accounts/acct/tokens/tok-deploy") echo '{"success":true,"result":{}}' ;;
  "PUT https://cf/accounts/acct/tokens/tok-deploy/value") echo '{"success":true,"result":"cf-rolled-deploy"}' ;;
  "GET https://cf/accounts/acct/workers/subdomain") echo '{"success":true,"result":{"subdomain":"jim"}}' ;;
  "POST https://a0.example/oauth/token") echo '{"access_token":"mgmt-token"}' ;;
  "GET https://a0.example/api/v2/clients?"*) auth0 '[{"client_id":"id-read","name":"infra-read"}]' ;;
  "POST https://a0.example/api/v2/clients/id-read/rotate-secret") auth0 '{"client_secret":"rotated-read"}' ;;
  "POST https://a0.example/api/v2/clients") auth0 '{"client_id":"id-apply","client_secret":"new-apply"}' ;;
  "GET https://a0.example/api/v2/client-grants?client_id=id-read"*) auth0 '[{"id":"g-read"}]' ;;
  "GET https://a0.example/api/v2/client-grants?"*) auth0 '[]' ;;
  "PATCH https://a0.example/api/v2/client-grants/g-read" | "POST https://a0.example/api/v2/client-grants") auth0 '{}' ;;
  *) echo "curl stub: unexpected \$method \$url" >&2; exit 22 ;;
esac
EOF
cat >"$work/bin/keytool" <<EOF
#!/usr/bin/env bash
echo "keytool \$*" >>"$work/calls.log"
for ((i = 1; i <= \$#; i++)); do [ "\${!i}" = -keystore ] && { j=\$((i + 1)); echo "keystore with \$KS_PASS" >"\${!j}"; }; done
exit 0
EOF
chmod +x "$work/bin/"*
export PATH="$work/bin:$PATH"

reset() { rm -rf "$work/gh" "$work/calls.log" "$work/curl.log" "$work/out" "$work/summary"; mkdir -p "$work/gh"; }
mint() {
  env -i PATH="$PATH" HOME="$HOME" GH_TOKEN=app-token GITHUB_REPOSITORY=jimhillr/monorepo \
    AUTH0_INFRA_CREDENTIALS_DOMAIN=a0.example AUTH0_INFRA_CREDENTIALS_CLIENT_ID=boot-id \
    AUTH0_INFRA_CREDENTIALS_CLIENT_SECRET=boot-secret CLOUDFLARE_INFRA_CREDENTIALS_TOKEN=boot-cf \
    CLOUDFLARE_ACCOUNT_ID=acct CF_API=https://cf GITHUB_OUTPUT="$work/out" GITHUB_STEP_SUMMARY="$work/summary" \
    "$@"
}
secret() { cat "$work/gh/$1/$2" 2>/dev/null; }

# ready names what is missing.
code=0
out=$(env -i PATH="$PATH" GH_TOKEN=x bash "$script" ready) || code=$?
[ "$code" = 3 ] && grep -q CLOUDFLARE_INFRA_CREDENTIALS_TOKEN <<<"$out" && grep -q AUTH0_INFRA_CREDENTIALS_CLIENT_SECRET <<<"$out" &&
  ok "ready names the missing bootstrap credentials" || fail "ready: code $code, $out"
code=0
env -i PATH="$PATH" bash "$script" run all >/dev/null 2>&1 || code=$?
[ "$code" = 3 ] && ok "run stops when the bootstrap isn't set up" || fail "run without bootstrap: code $code"

# A first run makes everything, each in its place.
reset
mint bash "$script" run all >"$work/run.log" 2>&1 || fail "first run: $(grep -v add-mask "$work/run.log" | tail -15)"
[ -n "$(secret repo TOFU_STATE_READ_PASSWORD)" ] && [ -n "$(secret infra TOFU_STATE_WRITE_PASSWORD)" ] &&
  [ -n "$(secret repo TOFU_STATE_PASSPHRASE)" ] && ok "state passwords: read and passphrase in the repo, write in infra" || fail "state secrets"
[ ! -e "$work/gh/repo/TOFU_STATE_WRITE_PASSWORD" ] && ok "the write password is never a repository secret" || fail "write password in repo"
[ "$(secret repo TOFU_STATE_READ_PASSWORD)" != "$(secret infra TOFU_STATE_WRITE_PASSWORD)" ] &&
  [ "$(secret repo TOFU_STATE_READ_PASSWORD | wc -c)" -ge 40 ] && ok "passwords are long and distinct" || fail "password quality"
[ "$(secret repo CLOUDFLARE_INFRA_READ_TOKEN)" = cf-value-jimhillr-infra-read ] &&
  [ "$(secret infra CLOUDFLARE_INFRA_APPLY_TOKEN)" = cf-value-jimhillr-infra-apply ] && ok "Cloudflare read token in the repo, apply token in infra" || fail "cloudflare tokens"
[ "$(secret repo CLOUDFLARE_API_TOKEN)" = cf-rolled-deploy ] && grep -q "PUT https://cf/accounts/acct/tokens/tok-deploy " "$work/curl.log" &&
  ok "an existing Cloudflare token is updated and rolled, not duplicated" || fail "deploy token roll: $(secret repo CLOUDFLARE_API_TOKEN)"
grep -q '"permission_groups":\[{"id":"p1"},{"id":"w1"},{"id":"d1"},{"id":"a1"}\]' "$work/curl.log" && ok "the deploy token gets Pages, Workers, D1 and Analytics" || fail "deploy permissions"
[ "$(cat "$work/gh/vars/TOFU_STATE_ORIGIN")" = https://jimhillr-tofu-state.jim.workers.dev ] && ok "origin variable from the workers.dev subdomain" || fail "origin"
[ "$(secret repo AUTH0_INFRA_READ_CLIENT_SECRET)" = rotated-read ] && [ "$(secret repo AUTH0_INFRA_READ_CLIENT_ID)" = id-read ] &&
  ok "an existing Auth0 app gets its secret rotated" || fail "auth0 read"
[ "$(secret infra AUTH0_INFRA_APPLY_CLIENT_SECRET)" = new-apply ] && [ "$(secret infra AUTH0_INFRA_APPLY_DOMAIN)" = a0.example ] &&
  [ ! -e "$work/gh/repo/AUTH0_INFRA_APPLY_CLIENT_SECRET" ] && ok "a new Auth0 apply app, only in infra" || fail "auth0 apply"
grep -q 'PATCH https://a0.example/api/v2/client-grants/g-read {"scope":\["read:clients"\]}' "$work/curl.log" &&
  grep -q 'POST https://a0.example/api/v2/client-grants .*"scope":\["read:clients","update:clients"\]' "$work/curl.log" &&
  ok "grants are set to exactly the listed scopes" || fail "grants: $(grep client-grants "$work/curl.log")"
grep -q "keystore with $(secret repo ANDROID_KEYSTORE_PASSWORD)" <(secret repo ANDROID_KEYSTORE_BASE64 | base64 -d) &&
  ok "the Android keystore matches its password" || fail "android"
grep -q "deploy=true" "$work/out" && ok "a changed password asks for a Worker redeploy" || fail "deploy output"
grep -q "TOFU_STATE_WRITE_PASSWORD" "$work/summary" && ok "summary lists what was set" || fail "summary"
! grep -rqF "$(secret infra TOFU_STATE_WRITE_PASSWORD)" "$work/calls.log" "$work/curl.log" "$work/summary" &&
  ok "no value on a command line or in the summary" || fail "a value leaked into a log"
grep -q "::add-mask::$(secret infra TOFU_STATE_WRITE_PASSWORD)" "$work/run.log" && ok "values are masked" || fail "mask"

# A second run changes nothing.
before=$(secret repo TOFU_STATE_READ_PASSWORD)
: >"$work/calls.log"; : >"$work/curl.log"; : >"$work/out"
mint bash "$script" run all >/dev/null 2>&1 || fail "second run"
[ "$(secret repo TOFU_STATE_READ_PASSWORD)" = "$before" ] && ! grep -q "secret set" "$work/calls.log" &&
  [ ! -s "$work/curl.log" ] && grep -q "deploy=false" "$work/out" && ok "a repeat run makes nothing" || fail "second run changed things"

# MINT_ROTATE remakes only what it names.
mint MINT_ROTATE="TOFU_STATE_READ_PASSWORD" bash "$script" run state >/dev/null 2>&1 || fail "rotate run"
[ "$(secret repo TOFU_STATE_READ_PASSWORD)" != "$before" ] && ok "rotate remakes the named secret" || fail "rotate"
code=0
mint MINT_ROTATE="TOFU_STATE_PASSPHRASE" bash "$script" run state >/dev/null 2>&1 || code=$?
[ "$code" != 0 ] && ok "the state passphrase is never rotated here" || fail "passphrase rotated"
code=0
mint MINT_ROTATE="GH_INFRA_CREDENTIALS_APP_KEY" bash "$script" run state >/dev/null 2>&1 || code=$?
[ "$code" != 0 ] && ok "rotate refuses names it doesn't make" || fail "unknown rotate accepted"

# A missing Cloudflare permission group fails with the close matches.
reset
sed -i 's/{"id":"r1","name":"Account Web Analytics Read"},//' "$work/bin/curl"
code=0
out=$(mint bash "$script" run cloudflare 2>&1) || code=$?
[ "$code" != 0 ] && grep -q "groups like it: .*Account Analytics Read" <<<"$out" && ok "a renamed permission group fails clearly" || fail "missing group: $out"

if [ "$failures" -gt 0 ]; then
  echo "mint.sh tests: $failures failed"
  exit 1
fi
echo "mint.sh tests: all passed"
