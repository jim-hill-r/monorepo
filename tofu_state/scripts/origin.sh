#!/usr/bin/env bash
# Print the state Worker's origin, https://jimhillr-tofu-state.<subdomain>.workers.dev,
# from the account's workers.dev subdomain, so no workflow hard-codes it.
# Needs CLOUDFLARE_API_TOKEN (Workers Scripts: Read) and CLOUDFLARE_ACCOUNT_ID.
set -euo pipefail
: "${CLOUDFLARE_API_TOKEN:?CLOUDFLARE_API_TOKEN is not set}"
: "${CLOUDFLARE_ACCOUNT_ID:?CLOUDFLARE_ACCOUNT_ID is not set}"
api=${CF_API:-https://api.cloudflare.com/client/v4}
body=$(curl -fsS -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
  "$api/accounts/$CLOUDFLARE_ACCOUNT_ID/workers/subdomain")
sub=$(jq -er '.result.subdomain' <<<"$body") || { echo "origin: no workers.dev subdomain in the account" >&2; exit 1; }
[[ "$sub" =~ ^[a-z0-9-]+$ ]] || { echo "origin: unexpected subdomain '$sub'" >&2; exit 1; }
echo "https://jimhillr-tofu-state.$sub.workers.dev"
