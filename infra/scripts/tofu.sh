#!/usr/bin/env bash
# Plan and apply one infra stack with OpenTofu, for the workflows (decisions
# 0007, 0009 and 0010). The workflows stay thin; the steps are here so they
# can be tested.
#
#   tofu.sh ready   STACK ROLE         exit 3, naming what is missing, if the
#                                      secrets for STACK and ROLE aren't set
#   tofu.sh plan    STACK ROLE OUT     plan into OUT/: plan.tfplan (encrypted),
#                                      plan.txt, report.json, plan.sha256
#   tofu.sh apply   STACK ROLE DIR     apply exactly DIR/plan.tfplan after
#                                      checking DIR/plan.sha256
#   tofu.sh drift   STACK              refresh-only plan; exit 2 on drift
#   tofu.sh rekey   STACK              re-encrypt STACK's state with the
#                                      current passphrase, after a rotation
#
# ROLE picks the vendor credentials: `read` (PR plans, plans on main and
# drift), `apply` (the infra environment) or `credentials` (the
# infra-credentials environment, decision 0010). They come from the
# environment under their GitHub secret names and are mapped to the names the
# provider reads; nothing is written to disk.
#
# Always needs TOFU_STATE_ORIGIN, TF_HTTP_PASSWORD (the read password for PR
# plans and drift, the write password otherwise) and TOFU_STATE_PASSPHRASE.
# A PR plan (TOFU_LOCK=false) reads state without locking.
#
# The passphrase has generations (infra/docs/issues/iac-rotate-state-passphrase.md).
# TOFU_STATE_KEY_GEN (default 1) is the current one; from 2 on, the state is
# encrypted under that generation's key name and TOFU_STATE_PASSPHRASE_OLD,
# the previous passphrase, is the fallback, so state written before a
# rotation still reads until `rekey` re-encrypts it.
set -euo pipefail

here=$(cd "$(dirname "$0")/.." && pwd)
die() { echo "tofu.sh: $*" >&2; exit 1; }

mode=${1:-}
stack=${2:-}
[ -n "$mode" ] && [ -n "$stack" ] || die "usage: tofu.sh ready|plan|apply|drift|rekey STACK [ROLE] [DIR]"
[[ "$stack" =~ ^[a-z0-9_-]+$ ]] || die "bad stack name '$stack'"
[ -d "$here/out/$stack" ] || die "no stack $stack in infra/out/"

role_upper() {
  case "$1" in
    read) echo READ ;;
    apply) echo APPLY ;;
    credentials) echo CREDENTIALS ;;
    *) die "unknown role '$1' (read, apply or credentials)" ;;
  esac
}

# The GitHub secret names STACK needs for ROLE, one per line.
needed() {
  local r
  r=$(role_upper "$2")
  echo TOFU_STATE_ORIGIN
  echo TF_HTTP_PASSWORD
  echo TOFU_STATE_PASSPHRASE
  case "$1" in
    auth0)
      echo "AUTH0_INFRA_${r}_DOMAIN"
      echo "AUTH0_INFRA_${r}_CLIENT_ID"
      echo "AUTH0_INFRA_${r}_CLIENT_SECRET"
      ;;
    cloudflare)
      echo "CLOUDFLARE_INFRA_${r}_TOKEN"
      echo CLOUDFLARE_ACCOUNT_ID
      ;;
    github)
      # Every github change is a credential change (decision 0010), so it
      # has no everyday apply tier. The tokens are minted per run from the
      # infra GitHub Apps by actions/create-github-app-token.
      [ "$r" != APPLY ] || die "the github stack applies only in infra-credentials (decision 0010)"
      echo "GH_INFRA_${r}_TOKEN"
      ;;
    *) die "no credentials known for stack $1; add them to infra/scripts/tofu.sh" ;;
  esac
}

ready() {
  local missing="" name
  for name in $(needed "$1" "$2"); do
    [ -n "${!name:-}" ] || missing="$missing $name"
  done
  if [ -n "$missing" ]; then
    echo "tofu.sh: $1 ($2) is not set up yet; missing:$missing"
    return 3
  fi
}

# Map the role's secrets to the provider's own variable names.
credentials() {
  local r
  r=$(role_upper "$2")
  case "$1" in
    auth0)
      local d="AUTH0_INFRA_${r}_DOMAIN" i="AUTH0_INFRA_${r}_CLIENT_ID" s="AUTH0_INFRA_${r}_CLIENT_SECRET"
      export AUTH0_DOMAIN=${!d} AUTH0_CLIENT_ID=${!i} AUTH0_CLIENT_SECRET=${!s}
      ;;
    cloudflare)
      local t="CLOUDFLARE_INFRA_${r}_TOKEN"
      export CLOUDFLARE_API_TOKEN=${!t}
      export TF_VAR_cloudflare_account_id=$CLOUDFLARE_ACCOUNT_ID
      ;;
    github)
      local t="GH_INFRA_${r}_TOKEN"
      export GITHUB_TOKEN=${!t}
      ;;
  esac
  passphrase
}

# The state passphrase, and from generation 2 the encryption block with the
# previous one as the fallback (OpenTofu merges TF_ENCRYPTION over the
# committed generation 1 block).
passphrase() {
  local gen=${TOFU_STATE_KEY_GEN:-1}
  [[ "$gen" =~ ^[1-9][0-9]{0,5}$ ]] || die "TOFU_STATE_KEY_GEN must be a whole number from 1, got '$gen'"
  export TF_VAR_tofu_state_passphrase=$TOFU_STATE_PASSPHRASE
  unset TF_ENCRYPTION TF_VAR_tofu_state_passphrase_old
  if [ "$gen" -gt 1 ]; then
    TF_ENCRYPTION=$(infra encryption "$gen")
    export TF_ENCRYPTION
    export TF_VAR_tofu_state_passphrase_old=${TOFU_STATE_PASSPHRASE_OLD:-}
  fi
  export TF_IN_AUTOMATION=1 TF_INPUT=0
}

infra() { (cd "$here" && cargo run --quiet -- "$@"); }

init() {
  local args
  mapfile -t args < <(infra backend-config "$1" "$TOFU_STATE_ORIGIN")
  (cd "$here/out/$1" && tofu init -input=false -lockfile=readonly -no-color "${args[@]}" >/dev/null)
}

summary() { cat >>"${GITHUB_STEP_SUMMARY:-/dev/null}"; }

plan() {
  local out=$3 lock=${TOFU_LOCK:-true} code=0
  mkdir -p "$out"
  out=$(cd "$out" && pwd)
  init "$1"
  (cd "$here/out/$1" && tofu plan -no-color -lock="$lock" -detailed-exitcode -out="$out/plan.tfplan") \
    >"$out/plan.log" 2>&1 || code=$?
  if [ "$code" -eq 1 ]; then
    cat "$out/plan.log" >&2
    die "plan failed for $1"
  fi
  (cd "$here/out/$1" && tofu show -no-color "$out/plan.tfplan") >"$out/plan.txt"
  (cd "$here/out/$1" && tofu show -json "$out/plan.tfplan") >"$out/plan.json"
  local guard=0
  infra guard "$out/plan.json" --report "$out/report.json" >"$out/guard.txt" 2>&1 || guard=$?
  # The JSON plan is only for the guards; it isn't kept or uploaded.
  rm -f "$out/plan.json"
  (cd "$out" && sha256sum plan.tfplan >plan.sha256)
  local changes=no credential=no
  [ "$code" -eq 2 ] && changes=yes
  jq -e '.credential_change == true' "$out/report.json" >/dev/null 2>&1 && credential=yes
  echo "changes=$changes" >"$out/result"
  echo "credential_change=$credential" >>"$out/result"
  {
    echo "### Plan: $1"
    echo
    [ "$credential" = yes ] && echo "**Credential change**: applies only after Jim approves it in \`infra-credentials\` (decision 0010)." && echo
    echo "| | |"
    echo "| --- | --- |"
    echo "| Changes | $changes |"
    echo "| Credential change | $credential |"
    echo "| Guards | $([ "$guard" -eq 0 ] && echo pass || echo FAIL) |"
    echo "| Plan SHA-256 | \`$(cut -d' ' -f1 "$out/plan.sha256")\` |"
    echo
    echo '<details><summary>Plan</summary>'
    echo
    echo '```'
    cat "$out/plan.txt"
    echo '```'
    echo
    echo '</details>'
    echo
    echo '```'
    cat "$out/guard.txt"
    echo '```'
  } | summary
  cat "$out/plan.txt"
  cat "$out/guard.txt"
  [ "$guard" -eq 0 ] || die "the plan guards failed for $1"
}

apply() {
  local dir
  dir=$(cd "$3" && pwd)
  (cd "$dir" && sha256sum --check --quiet plan.sha256) || die "plan.tfplan doesn't match plan.sha256; not applying"
  init "$1"
  # Exactly the saved plan: OpenTofu refuses it if state changed since.
  (cd "$here/out/$1" && tofu apply -no-color -input=false "$dir/plan.tfplan")
  echo "### Applied: $1 (plan \`$(cut -d' ' -f1 "$dir/plan.sha256")\`)" | summary
}

drift() {
  init "$1"
  local code=0
  (cd "$here/out/$1" && tofu plan -no-color -refresh-only -lock=false -detailed-exitcode) || code=$?
  case "$code" in
    0) echo "### Drift: $1: none" | summary ;;
    2) echo "### Drift: $1: the live settings differ from state; see the log" | summary; return 2 ;;
    *) die "drift plan failed for $1" ;;
  esac
}

# Re-encrypt a stack's state under the current generation's key. The key
# names in the state's `meta` are plain text, so a stack already on the
# current key, or with no state yet, is left alone: each push is a new
# version in D1, and the Worker keeps only the last 10.
rekey() {
  local key raw
  key="key_provider.pbkdf2.$(jq -r '.state.method' <<<"${TF_ENCRYPTION:-$(infra encryption 1)}" | sed 's/^method\.aes_gcm\.//')"
  raw=$(curl -sS --fail --max-time 30 -K- "$TOFU_STATE_ORIGIN/states/$1" <<<"user = \"tofu:$TF_HTTP_PASSWORD\"") ||
    die "can't read the $1 state"
  if [ -z "$raw" ]; then
    echo "### Rekey: $1: no state yet" | summary
    return 0
  fi
  if jq -e --arg key "$key" '.meta | has($key)' <<<"$raw" >/dev/null; then
    echo "### Rekey: $1: already on $key" | summary
    return 0
  fi
  init "$1"
  # The decrypted state goes through the pipe only, never to a file.
  (cd "$here/out/$1" && tofu state pull | tofu state push -no-color -) >/dev/null ||
    die "rekey failed for $1"
  echo "### Rekey: $1: re-encrypted with $key" | summary
}

# The state secrets alone: rekey touches no vendor.
state_ready() {
  local missing="" name
  for name in TOFU_STATE_ORIGIN TF_HTTP_PASSWORD TOFU_STATE_PASSPHRASE; do
    [ -n "${!name:-}" ] || missing="$missing $name"
  done
  [ -z "$missing" ] || { echo "tofu.sh: rekey $1 is not set up yet; missing:$missing" >&2; return 3; }
}

case "$mode" in
  ready) ready "$stack" "${3:?role}" ;;
  plan) ready "$stack" "${3:?role}" >&2 || exit 3; credentials "$stack" "$3"; plan "$stack" "$3" "${4:?out dir}" ;;
  apply) ready "$stack" "${3:?role}" >&2 || exit 3; credentials "$stack" "$3"; apply "$stack" "$3" "${4:?plan dir}" ;;
  drift) ready "$stack" read >&2 || exit 3; credentials "$stack" read; drift "$stack" ;;
  rekey) state_ready "$stack" || exit 3; passphrase; rekey "$stack" ;;
  *) die "unknown mode '$mode'" ;;
esac
