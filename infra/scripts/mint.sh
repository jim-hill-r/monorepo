#!/usr/bin/env bash
# Mint the infra tokens and passwords straight into GitHub secrets, for the
# infra-mint workflow (Jim, 2026-11-27). OpenTofu never does this: a
# vendor-made secret would land in tofu state, which the rule "never commit
# secrets to git or store them in tf state" forbids. Here each value goes
# from the vendor's API response (or `openssl rand`) through a pipe into
# `gh secret set`, masked in the log, and is never written to git, state or
# a file, nor seen by anyone.
#
#   mint.sh ready              exit 3, naming what is missing, if the
#                              bootstrap credentials aren't set
#   mint.sh run WHAT           WHAT: all, cloudflare, state, origin, auth0
#                              or android
#
# Each secret is made only when it is missing from GitHub, or named in
# MINT_ROTATE (space-separated), so a run is safe to repeat. MINT_ROTATE=auto
# means every secret in ROTATE_AUTO: the scheduled quarterly rotation
# (standards/docs/secret-rotation.md).
# TOFU_STATE_PASSPHRASE is never rotated here: the state is encrypted with
# it, so changing it means re-encrypting state first.
#
# Needs, from the infra-credentials environment:
#   GH_TOKEN                   the infra GitHub App's installation token
#   GITHUB_REPOSITORY          owner/name
#   AUTH0_INFRA_CREDENTIALS_DOMAIN, _CLIENT_ID, _CLIENT_SECRET
#   CLOUDFLARE_INFRA_CREDENTIALS_TOKEN (Account API Tokens: Write)
#   CLOUDFLARE_ACCOUNT_ID
# and optionally CLOUDFLARE_API_TOKEN, the current deploy token, to find the
# state Worker's origin when this run doesn't mint a new one.
#
# Writes deploy=true to GITHUB_OUTPUT when the state passwords or the deploy
# token changed, so the workflow redeploys the state Worker.
set -euo pipefail

here=$(cd "$(dirname "$0")/.." && pwd)
root=$(cd "$here/.." && pwd)
die() { echo "mint.sh: $*" >&2; exit 1; }
log() { echo "mint: $*"; }
CF_API=${CF_API:-https://api.cloudflare.com/client/v4}
AUTH0_SCHEME=${AUTH0_SCHEME:-https}

# The secrets this script makes, and where each lives: "repo" or an
# environment name. Read credentials are repository secrets because PR plans
# use them; anything that can change a vendor is only in `infra`.
declare -A WHERE=(
  [TOFU_STATE_READ_PASSWORD]=repo
  [TOFU_STATE_PASSPHRASE]=repo
  [TOFU_STATE_WRITE_PASSWORD]=infra
  [CLOUDFLARE_API_TOKEN]=repo
  [CLOUDFLARE_INFRA_READ_TOKEN]=repo
  [CLOUDFLARE_INFRA_APPLY_TOKEN]=infra
  [AUTH0_INFRA_READ_DOMAIN]=repo
  [AUTH0_INFRA_READ_CLIENT_ID]=repo
  [AUTH0_INFRA_READ_CLIENT_SECRET]=repo
  [AUTH0_INFRA_APPLY_DOMAIN]=infra
  [AUTH0_INFRA_APPLY_CLIENT_ID]=infra
  [AUTH0_INFRA_APPLY_CLIENT_SECRET]=infra
  [ANDROID_KEYSTORE_BASE64]=repo
  [ANDROID_KEYSTORE_PASSWORD]=repo
)

# Cloudflare account-owned tokens: name and permission groups (the names the
# permission-groups API uses). A missing name fails the run and lists the
# close matches, since Cloudflare renames groups now and then.
# Web Analytics (the RUM API) has no permission group of its own: it takes
# Account Settings, so the infra tokens use that (checked against the live
# API, 2026-11-27).
CF_DEPLOY_PERMS=("Pages Write" "Workers Scripts Write" "D1 Write" "Account Analytics Read")
CF_READ_PERMS=("Account Settings Read")
CF_APPLY_PERMS=("Account Settings Write")

# What MINT_ROTATE=auto remakes. Never the state passphrase (it encrypts
# state), the Android keystore (a new one can't update the installed app)
# or TOFU_STATE_ORIGIN (not a secret).
ROTATE_AUTO="TOFU_STATE_READ_PASSWORD TOFU_STATE_WRITE_PASSWORD CLOUDFLARE_API_TOKEN \
CLOUDFLARE_INFRA_READ_TOKEN CLOUDFLARE_INFRA_APPLY_TOKEN AUTH0_INFRA_READ_CLIENT_SECRET AUTH0_INFRA_APPLY_CLIENT_SECRET"

# Minted Cloudflare tokens expire this many days after each mint, so a missed
# rotation fails closed; the quarterly rotation renews them well before.
CF_TOKEN_DAYS=${CF_TOKEN_DAYS:-180}

# Auth0 Machine to Machine apps for the Management API, and their scopes.
AUTH0_READ_SCOPES='["read:clients"]'
AUTH0_APPLY_SCOPES='["read:clients","update:clients"]'

bootstrap=(GH_TOKEN GITHUB_REPOSITORY AUTH0_INFRA_CREDENTIALS_DOMAIN AUTH0_INFRA_CREDENTIALS_CLIENT_ID
  AUTH0_INFRA_CREDENTIALS_CLIENT_SECRET CLOUDFLARE_INFRA_CREDENTIALS_TOKEN CLOUDFLARE_ACCOUNT_ID)

ready() {
  local missing="" name
  for name in "${bootstrap[@]}"; do
    [ -n "${!name:-}" ] || missing="$missing $name"
  done
  if [ -n "$missing" ]; then
    echo "mint.sh: the bootstrap credentials are not set up yet; missing:$missing"
    echo "  (standards/docs/runbooks/infra-credentials-environment.md and github-app-infra.md)"
    return 3
  fi
}

# --- GitHub -----------------------------------------------------------------

declare -A HAVE=()
load_existing() {
  local env name
  for env in repo infra; do
    if [ "$env" = repo ]; then
      names=$(gh secret list --repo "$GITHUB_REPOSITORY" --json name -q '.[].name')
    else
      names=$(gh secret list --repo "$GITHUB_REPOSITORY" --env "$env" --json name -q '.[].name')
    fi
    for name in $names; do HAVE["$env/$name"]=1; done
  done
}

# wanted NAME: true if NAME should be (re)made in this run.
wanted() {
  local name=$1
  [ -z "${HAVE["${WHERE[$name]}/$name"]:-}" ] && return 0
  [[ " ${MINT_ROTATE:-} " == *" $name "* ]]
}

changed=no
# put NAME VALUE: store VALUE as the GitHub secret NAME, without it touching
# a file or the command line.
put() {
  local name=$1 value=$2 env=${WHERE[$1]}
  [ -n "$value" ] || die "empty value for $name; not storing it"
  # Mask every line (a keystore or key can span several).
  while IFS= read -r line; do [ -n "$line" ] && echo "::add-mask::$line"; done <<<"$value"
  if [ "$env" = repo ]; then
    printf '%s' "$value" | gh secret set "$name" --repo "$GITHUB_REPOSITORY"
  else
    printf '%s' "$value" | gh secret set "$name" --repo "$GITHUB_REPOSITORY" --env "$env"
  fi
  HAVE["$env/$name"]=1
  log "set $name ($([ "$env" = repo ] && echo repository secret || echo "secret of $env"))"
  echo "| \`$name\` | $([ "$env" = repo ] && echo repository || echo "$env") |" >>"$summary_rows"
}

# --- generated passwords ----------------------------------------------------

password() { openssl rand -base64 48 | tr -d '\n/+='; }

mint_state() {
  for name in TOFU_STATE_READ_PASSWORD TOFU_STATE_WRITE_PASSWORD; do
    if wanted "$name"; then put "$name" "$(password)"; changed=yes; fi
  done
  if [ -z "${HAVE["repo/TOFU_STATE_PASSPHRASE"]:-}" ]; then
    put TOFU_STATE_PASSPHRASE "$(password)"
  elif [[ " ${MINT_ROTATE:-} " == *" TOFU_STATE_PASSPHRASE "* ]]; then
    die "TOFU_STATE_PASSPHRASE encrypts the state; rotating it needs the state re-encrypted first (decision 0009)"
  fi
}

mint_android() {
  wanted ANDROID_KEYSTORE_BASE64 || wanted ANDROID_KEYSTORE_PASSWORD || return 0
  local dir pw ks
  dir=$(mktemp -d)
  pw=$(password)
  # keytool reads the password from the environment, not the command line.
  KS_PASS=$pw keytool -genkeypair -noprompt -keystore "$dir/release.keystore" -alias release \
    -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=jimhillr playtest" \
    -storepass:env KS_PASS -keypass:env KS_PASS >/dev/null 2>&1 || { rm -rf "$dir"; die "keytool failed"; }
  ks=$(base64 -w0 "$dir/release.keystore")
  rm -rf "$dir"
  put ANDROID_KEYSTORE_PASSWORD "$pw"
  put ANDROID_KEYSTORE_BASE64 "$ks"
}

# --- Cloudflare -------------------------------------------------------------

cf() {
  local method=$1 path=$2 body=${3:-}
  local args=(-sS -X "$method" -H "Authorization: Bearer $CLOUDFLARE_INFRA_CREDENTIALS_TOKEN")
  [ -n "$body" ] && args+=(-H "Content-Type: application/json" --data-binary @-)
  local out
  out=$(printf '%s' "$body" | curl "${args[@]}" "$CF_API$path") || die "cloudflare $method $path: request failed"
  jq -e '.success == true' >/dev/null <<<"$out" ||
    die "cloudflare $method $path: $(jq -c '.errors // .' <<<"$out" 2>/dev/null || echo "$out" | head -c 300)"
  printf '%s' "$out"
}

cf_groups=""
cf_policies() {
  local acct="com.cloudflare.api.account.$CLOUDFLARE_ACCOUNT_ID" ids="[]" name id
  [ -n "$cf_groups" ] || cf_groups=$(cf GET "/accounts/$CLOUDFLARE_ACCOUNT_ID/tokens/permission_groups?per_page=1000")
  for name in "$@"; do
    id=$(jq -r --arg n "$name" '[.result[] | select(.name == $n)][0].id // empty' <<<"$cf_groups")
    if [ -z "$id" ]; then
      local word=${name%% *}
      die "no Cloudflare permission group '$name'; groups like it: $(jq -r --arg w "$word" '[.result[].name | select(test($w; "i"))] | join(", ")' <<<"$cf_groups")"
    fi
    ids=$(jq -c --arg id "$id" '. + [{id: $id}]' <<<"$ids")
  done
  jq -cn --arg acct "$acct" --argjson ids "$ids" '[{effect: "allow", resources: {($acct): "*"}, permission_groups: $ids}]'
}

# cf_token SECRET TOKEN_NAME PERMISSION...: make or roll an account-owned
# token and store its value. Prints the value for the caller's own use.
minted_deploy_token=""
cf_token() {
  local secret=$1 name=$2; shift 2
  wanted "$secret" || return 0
  local policies existing value base="/accounts/$CLOUDFLARE_ACCOUNT_ID/tokens"
  policies=$(cf_policies "$@")
  local expires body
  expires=$(date -u -d "+$CF_TOKEN_DAYS days" +%Y-%m-%dT%H:%M:%SZ)
  body=$(jq -cn --arg n "$name" --argjson p "$policies" --arg e "$expires" '{name: $n, policies: $p, expires_on: $e}')
  existing=$(cf GET "$base?per_page=100" | jq -r --arg n "$name" '[.result[] | select(.name == $n)][0].id // empty')
  if [ -n "$existing" ]; then
    cf PUT "$base/$existing" "$body" >/dev/null
    value=$(cf PUT "$base/$existing/value" '{}' | jq -r '.result')
  else
    value=$(cf POST "$base" "$body" | jq -r '.result.value')
  fi
  put "$secret" "$value"
  if [ "$secret" = CLOUDFLARE_API_TOKEN ]; then minted_deploy_token=$value; changed=yes; fi
}

mint_cloudflare() {
  cf_token CLOUDFLARE_API_TOKEN jimhillr-deploy "${CF_DEPLOY_PERMS[@]}"
  cf_token CLOUDFLARE_INFRA_READ_TOKEN jimhillr-infra-read "${CF_READ_PERMS[@]}"
  cf_token CLOUDFLARE_INFRA_APPLY_TOKEN jimhillr-infra-apply "${CF_APPLY_PERMS[@]}"
}

mint_origin() {
  local have token origin
  have=$(gh variable list --repo "$GITHUB_REPOSITORY" --json name -q '.[].name' | grep -cx TOFU_STATE_ORIGIN || true)
  if [ "$have" != 0 ] && [[ " ${MINT_ROTATE:-} " != *" TOFU_STATE_ORIGIN "* ]]; then return 0; fi
  token=${minted_deploy_token:-${CLOUDFLARE_API_TOKEN:-}}
  [ -n "$token" ] || die "TOFU_STATE_ORIGIN needs a deploy token; run with cloudflare first"
  origin=$(CLOUDFLARE_API_TOKEN=$token CF_API=$CF_API bash "$root/tofu_state/scripts/origin.sh")
  gh variable set TOFU_STATE_ORIGIN --repo "$GITHUB_REPOSITORY" --body "$origin"
  log "set variable TOFU_STATE_ORIGIN=$origin"
  echo "| \`TOFU_STATE_ORIGIN\` | repository variable: $origin |" >>"$summary_rows"
}

# --- Auth0 ------------------------------------------------------------------

auth0_token=""
a0() {
  local method=$1 path=$2 body=${3:-} out code
  local args=(-sS -X "$method" -H "Authorization: Bearer $auth0_token" -w '\n%{http_code}')
  [ -n "$body" ] && args+=(-H "Content-Type: application/json" --data-binary @-)
  out=$(printf '%s' "$body" | curl "${args[@]}" "$AUTH0_SCHEME://$AUTH0_INFRA_CREDENTIALS_DOMAIN/api/v2$path") ||
    die "auth0 $method $path: request failed"
  code=${out##*$'\n'}
  out=${out%$'\n'*}
  [[ "$code" == 2* ]] || die "auth0 $method $path: HTTP $code $(jq -c '{error, message}' <<<"$out" 2>/dev/null)"
  printf '%s' "$out"
}

auth0_login() {
  [ -n "$auth0_token" ] && return 0
  local d=$AUTH0_INFRA_CREDENTIALS_DOMAIN out
  # The bootstrap secret reaches jq through the environment, not its arguments.
  out=$(A0_SECRET=$AUTH0_INFRA_CREDENTIALS_CLIENT_SECRET jq -cn --arg id "$AUTH0_INFRA_CREDENTIALS_CLIENT_ID" \
    --arg aud "https://$d/api/v2/" '{grant_type: "client_credentials", client_id: $id, client_secret: env.A0_SECRET, audience: $aud}' |
    curl -sS -X POST -H "Content-Type: application/json" --data-binary @- "$AUTH0_SCHEME://$d/oauth/token") ||
    die "auth0 sign-in failed"
  auth0_token=$(jq -r '.access_token // empty' <<<"$out")
  [ -n "$auth0_token" ] || die "auth0 sign-in: $(jq -c '{error, error_description}' <<<"$out" 2>/dev/null)"
  echo "::add-mask::$auth0_token"
}

# auth0_app PREFIX NAME SCOPES: make the M2M app (or rotate its secret), set
# its grant on the Management API to exactly SCOPES, and store
# PREFIX_DOMAIN, PREFIX_CLIENT_ID and PREFIX_CLIENT_SECRET.
auth0_app() {
  local p=$1 name=$2 scopes=$3
  wanted "${p}_CLIENT_SECRET" || wanted "${p}_CLIENT_ID" || wanted "${p}_DOMAIN" || return 0
  auth0_login
  local aud="https://$AUTH0_INFRA_CREDENTIALS_DOMAIN/api/v2/" id secret grant out
  id=$(a0 GET "/clients?fields=client_id,name&include_fields=true&per_page=100" |
    jq -r --arg n "$name" '[.[] | select(.name == $n)][0].client_id // empty')
  if [ -n "$id" ]; then
    secret=$(a0 POST "/clients/$id/rotate-secret" | jq -r '.client_secret')
  else
    out=$(a0 POST /clients "$(jq -cn --arg n "$name" '{name: $n, app_type: "non_interactive",
      grant_types: ["client_credentials"], token_endpoint_auth_method: "client_secret_post"}')")
    id=$(jq -r '.client_id' <<<"$out")
    secret=$(jq -r '.client_secret' <<<"$out")
  fi
  grant=$(a0 GET "/client-grants?client_id=$id&audience=$(jq -rn --arg a "$aud" '$a | @uri')" | jq -r '.[0].id // empty')
  if [ -n "$grant" ]; then
    a0 PATCH "/client-grants/$grant" "$(jq -cn --argjson s "$scopes" '{scope: $s}')" >/dev/null
  else
    a0 POST /client-grants "$(jq -cn --arg c "$id" --arg a "$aud" --argjson s "$scopes" \
      '{client_id: $c, audience: $a, scope: $s}')" >/dev/null
  fi
  put "${p}_DOMAIN" "$AUTH0_INFRA_CREDENTIALS_DOMAIN"
  put "${p}_CLIENT_ID" "$id"
  put "${p}_CLIENT_SECRET" "$secret"
}

mint_auth0() {
  auth0_app AUTH0_INFRA_READ infra-read "$AUTH0_READ_SCOPES"
  auth0_app AUTH0_INFRA_APPLY infra-apply "$AUTH0_APPLY_SCOPES"
}

# --- main ---------------------------------------------------------------------

run() {
  local what=$1
  case "$what" in all | cloudflare | state | origin | auth0 | android) ;; *) die "unknown WHAT '$what'" ;; esac
  [ "${MINT_ROTATE:-}" = auto ] && MINT_ROTATE=$ROTATE_AUTO
  for name in ${MINT_ROTATE:-}; do
    [[ -n "${WHERE[$name]:-}" || "$name" == TOFU_STATE_ORIGIN ]] || die "MINT_ROTATE: '$name' isn't a secret this script makes"
  done
  summary_rows=$(mktemp)
  load_existing
  case "$what" in all | cloudflare) mint_cloudflare ;; esac
  case "$what" in all | state) mint_state ;; esac
  case "$what" in all | origin) mint_origin ;; esac
  case "$what" in all | auth0) mint_auth0 ;; esac
  case "$what" in all | android) mint_android ;; esac
  {
    echo "### Minted ($what)"
    echo
    if [ -s "$summary_rows" ]; then
      echo "| Set | Where |"
      echo "| --- | --- |"
      cat "$summary_rows"
    else
      echo "Nothing to do: every secret already exists. Name one in \`rotate\` to remake it."
    fi
  } >>"${GITHUB_STEP_SUMMARY:-/dev/null}"
  rm -f "$summary_rows"
  echo "deploy=$([ "$changed" = yes ] && echo true || echo false)" >>"${GITHUB_OUTPUT:-/dev/null}"
}

case "${1:-}" in
  ready) ready ;;
  run) ready >&2 || exit 3; run "${2:?what}" ;;
  *) die "usage: mint.sh ready | run all|cloudflare|state|origin|auth0|android" ;;
esac
