#!/usr/bin/env bash
# PR-003, the reviewer account (decision 0014,
# standards/docs/decisions/0014-faster-pr-approvals.md, "The reviewer
# account" in standards/docs/pull-requests.md). `jimhillr-reviewer` approves
# an agent PR only when every rule below holds; anything else waits for Jim.
# Each approval's review body records the rules it passed, and Jim reads
# every merged PR afterwards on pane's Acknowledge page.
#
#   bash auto-review.sh decide PR.json FILES.txt CHECKS.json REVIEWS.json
#       Prints "approve: <record>" or "skip: <reason>". Exit 0 either way.
#       PR.json: GET /repos/{repo}/pulls/{n}. FILES.txt: changed paths, one
#       per line. CHECKS.json: {"check_runs":[...]} for the head commit,
#       plus "statuses" from its combined status. REVIEWS.json: the
#       PR's reviews.
#   GH_REPO=owner/repo bash auto-review.sh run [--dry-run]
#       Fetches every open PR with `gh`, decides, and approves the ones that
#       pass as the token's account (GH_TOKEN: jimhillr-reviewer's). Writes
#       a table to GITHUB_STEP_SUMMARY when it is set.
#
# Rules version, written into every approval.
set -euo pipefail

RULES=v2
REVIEWER=jimhillr-reviewer
AUTHOR=jimhillr-agt
MAX_LINES=1500
MAX_FILES=100
# Labels with a red or orange attention badge (pane_model::queue).
BLOCK_LABELS='needs-human credential-change security cost'
# Red and orange attention tags (`chores badges`, "Attention tags" in
# standards/docs/pull-requests.md). A description that declares one, or has
# no Attention section, waits for Jim (Jim, 2026-11-27: "no prs with red
# attention flags get auto merged. Cost changes must be reviewed by jim").
BLOCK_TAGS='must-requirement-changed cost-changed threat-risk-changed'
# Paths the reviewer never approves: secrets, credentials and workflows,
# threat models, requirements and evals, the rules agents follow, decisions,
# vendors and budgets, security headers and deploy settings, auth.
# Extended regular expressions, matched against the whole path.
BLOCK_PATHS='^\.github/
^\.agents/
^\.claude/
^AGENTS\.md$
^CLAUDE\.md$
^SECURITY\.md$
^\.gitleaks
^lefthook\.yml$
^mise\.toml$
^infra/
^tofu_state/
^threat/
^auth_sdk/
^jimhillr/
^standards/
(^|/)docs/threat\.toml$
(^|/)docs/requirements\.md$
(^|/)docs/cost\.toml$
(^|/)evals\.toml$
(^|/)royalty\.toml$
(^|/)wrangler\.toml$
(^|/)wrangler\.jsonc?$
(^|/)_headers$
(^|/)\.env'
# The checks that must have run and passed, on top of "every check passed".
REQUIRED_CHECKS='CI result
PR summary
Jim review'
# This workflow's own job, ignored when reading the head's checks.
SELF_CHECK='Auto review'

decide() {
  local pr=$1 files=$2 checks=$3 reviews=$4
  local number draft base author sha adds dels nfiles
  number=$(jq -r '.number' "$pr")
  draft=$(jq -r '.draft' "$pr")
  base=$(jq -r '.base.ref' "$pr")
  author=$(jq -r '.user.login' "$pr")
  sha=$(jq -r '.head.sha' "$pr")
  adds=$(jq -r '.additions // 0' "$pr")
  dels=$(jq -r '.deletions // 0' "$pr")
  nfiles=$(grep -c . "$files" || true)

  [ "$draft" = false ] || { echo "skip: draft"; return; }
  [ "$base" = main ] || { echo "skip: based on $base, not main"; return; }
  [ "$author" = "$AUTHOR" ] || { echo "skip: author $author is not $AUTHOR"; return; }

  local label
  for label in $BLOCK_LABELS; do
    if jq -e --arg l "$label" 'any(.labels[]?; .name == $l)' "$pr" >/dev/null; then
      echo "skip: labelled $label"
      return
    fi
  done

  # The tags on the description's Attention section bullets, one per line.
  local tags tag
  tags=$(jq -r '.body // ""' "$pr" | tr -d '\r' | awk '
    /^#+[ \t]/ { h = $0; sub(/^#+[ \t]+/, "", h); on = (tolower(h) ~ /^attention[ \t]*$/); if (on) seen = 1; next }
    on && /^[ \t]*- / { for (i = 2; i <= NF; i++) if ($i ~ /[A-Za-z0-9]/) { t = $i; sub(/:$/, "", t); gsub(/`/, "", t); print t; break } }
    END { if (!seen) print "<none>" }')
  if [ "$tags" = "<none>" ]; then
    echo "skip: no Attention section in the description"
    return
  fi
  for tag in $BLOCK_TAGS; do
    if grep -qx -- "$tag" <<<"$tags"; then
      echo "skip: attention tag $tag needs Jim"
      return
    fi
  done

  [ "$nfiles" -gt 0 ] || { echo "skip: no changed files listed"; return; }
  [ "$nfiles" -le "$MAX_FILES" ] || { echo "skip: $nfiles files, over $MAX_FILES"; return; }
  [ $((adds + dels)) -le "$MAX_LINES" ] || { echo "skip: $((adds + dels)) changed lines, over $MAX_LINES"; return; }

  local blocked
  blocked=$(grep -E -f <(printf '%s\n' "$BLOCK_PATHS") "$files" | head -n1 || true)
  [ -z "$blocked" ] || { echo "skip: touches $blocked"; return; }

  # Every check run on the head finished and passed (skipped and neutral
  # count as passed), and every commit status is success.
  local bad
  bad=$(jq -r --arg self "$SELF_CHECK" '
    [(.check_runs[]? | select(.name != $self)
      | select(.status != "completed" or ((.conclusion // "") | IN("success", "skipped", "neutral") | not))
      | "\(.name) is \(.conclusion // .status)"),
     (.statuses[]? | select(.state != "success") | "\(.context) is \(.state)")]
    | first // empty' "$checks")
  [ -z "$bad" ] || { echo "skip: check $bad"; return; }
  local need
  while IFS= read -r need; do
    if ! jq -e --arg n "$need" 'any(.check_runs[]?; .name == $n and .conclusion == "success")' "$checks" >/dev/null; then
      echo "skip: check $need has not passed"
      return
    fi
  done <<<"$REQUIRED_CHECKS"

  if jq -e 'any(.[]?; .state == "CHANGES_REQUESTED")' "$reviews" >/dev/null; then
    echo "skip: changes requested"
    return
  fi
  if jq -e --arg r "$REVIEWER" --arg sha "$sha" \
    'any(.[]?; .user.login == $r and .state == "APPROVED" and .commit_id == $sha)' "$reviews" >/dev/null; then
    echo "skip: already approved at ${sha:0:7}"
    return
  fi

  echo "approve: $REVIEWER rules $RULES (decision 0014): checks green on ${sha:0:7}, no red or orange label or attention tag, no protected path, $nfiles files, +$adds/-$dels. Jim reviews it after merge on pane's Acknowledge page."
}

run() {
  local dry=${1:-}
  local tmp
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' RETURN
  local summary=${GITHUB_STEP_SUMMARY:-/dev/null}
  printf '| PR | Decision |\n| --- | --- |\n' >>"$summary"
  local n
  for n in $(gh api "repos/$GH_REPO/pulls?state=open&per_page=100" --paginate \
    --jq ".[] | select(.draft == false and .user.login == \"$AUTHOR\") | .number"); do
    gh api "repos/$GH_REPO/pulls/$n" >"$tmp/pr.json"
    gh api "repos/$GH_REPO/pulls/$n/files" --paginate --jq '.[].filename' >"$tmp/files.txt"
    local sha
    sha=$(jq -r '.head.sha' "$tmp/pr.json")
    jq -n --slurpfile runs <(gh api "repos/$GH_REPO/commits/$sha/check-runs" --paginate --jq '.check_runs[]') \
      --slurpfile st <(gh api "repos/$GH_REPO/commits/$sha/status" --jq '.statuses[]') \
      '{check_runs: $runs, statuses: $st}' >"$tmp/checks.json"
    gh api "repos/$GH_REPO/pulls/$n/reviews" --paginate --jq '.[]' | jq -s . >"$tmp/reviews.json"
    local verdict
    verdict=$(decide "$tmp/pr.json" "$tmp/files.txt" "$tmp/checks.json" "$tmp/reviews.json")
    echo "#$n: $verdict"
    printf '| #%s | %s |\n' "$n" "${verdict//|/\\|}" >>"$summary"
    if [[ $verdict == approve:* ]] && [ "$dry" != --dry-run ]; then
      # commit_id pins the approval to the commit that was checked.
      gh api -X POST "repos/$GH_REPO/pulls/$n/reviews" -f event=APPROVE \
        -f commit_id="$sha" -f body="${verdict#approve: }" >/dev/null
    fi
  done
}

case "${1:-}" in
  decide) shift; decide "$@" ;;
  run) shift; run "$@" ;;
  *) echo "usage: auto-review.sh decide PR.json FILES.txt CHECKS.json REVIEWS.json | run [--dry-run]" >&2; exit 2 ;;
esac
