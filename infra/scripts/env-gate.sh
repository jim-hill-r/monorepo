#!/usr/bin/env bash
# Stop a credential job unless its environment really waits for a person
# (decision 0010). The first Infra Mint runs, on 2026-11-27, went through
# with no approval because `infra-credentials` had no required reviewer
# saved; GitHub doesn't warn about that.
#
# Usage: env-gate.sh <environment>. Needs GH_TOKEN (GITHUB_TOKEN with
# actions: read is enough) and GITHUB_REPOSITORY.
set -euo pipefail

env_name=${1:?usage: env-gate.sh <environment>}
die() {
  echo "::error::$env_name isn't protected: $* (standards/docs/runbooks/infra-credentials-environment.md, step 2)"
  exit 1
}

json=$(gh api "repos/$GITHUB_REPOSITORY/environments/$env_name") || die "can't read its settings"

reviewers=$(jq '[.protection_rules[]? | select(.type == "required_reviewers") | .reviewers[]?] | length' <<<"$json")
[ "$reviewers" -gt 0 ] || die "it has no required reviewer"
[ "$(jq -r '.can_admins_bypass' <<<"$json")" = "false" ] || die "administrators can bypass its rules"
[ "$(jq -r '.deployment_branch_policy != null' <<<"$json")" = "true" ] || die "any branch can deploy to it"

echo "$env_name waits for $reviewers required reviewer(s), with no admin bypass and limited branches."
