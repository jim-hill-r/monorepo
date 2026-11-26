#!/usr/bin/env bash
# Keep automation inside the free tiers of the services the repo uses.
# Limits, budgets and the reasons for them: standards/docs/free-tiers.md.
#
#   free-tiers.sh check
#       Static checks on the repo, no credentials. Run by `mise run //:audit`,
#       so every CI run enforces them.
#         FT-001  Workflows only use standard GitHub-hosted runners (larger
#                 runners are billed even on public repos).
#         FT-002  No workflow uses the RapidAPI key (homer must stay on the
#                 free plan, run by hand only).
#         FT-003  Cloudflare deploy projects (wrangler.toml files) stay within
#                 the Pages and Workers project budgets.
#         FT-004  Cron Triggers across all wrangler.toml files stay within
#                 the budget.
#         FT-005  No wrangler.toml turns on a paid-only setting.
#
#   free-tiers.sh pages-project
#       Run from a Cloudflare project directory (the deploy task does this).
#       Creates the Pages project named in ./wrangler.toml if it does not
#       exist, but only while the account stays within the Pages project
#       budget. Otherwise it stops with an error and creates nothing.
#
#   free-tiers.sh usage
#       Live usage check (Cloudflare and GitHub). Writes a table to
#       $GITHUB_STEP_SUMMARY when set and exits non-zero when anything is at
#       or over its warning level, so the scheduled run goes red.
#
# Budgets can be lowered with environment variables, never raised past the
# free limit: the script clamps them.
set -euo pipefail

# Free limits and our budgets, well inside them, from standards/radar/limits.toml
# (the one place they are kept; see standards/docs/free-tiers.md). Found next to
# this script, so fixture repos in the tests use the real numbers.
LIMITS_FILE=${FREE_TIERS_LIMITS_FILE:-"$(cd "$(dirname "${BASH_SOURCE[0]}")/../radar" && pwd)/limits.toml"}

# limit KEY FIELD: the whole-number FIELD (free or budget) of the [[limit]]
# whose key is KEY. Every field is on its own `name = value` line.
limit() {
  local value
  value=$(awk -v key="$1" -v field="$2" '
    /^\[\[limit\]\]/ { current = "" }
    $1 == "key" { gsub(/"/, "", $3); current = $3 }
    current == key && $1 == field { print $3; exit }
  ' "$LIMITS_FILE")
  [[ "$value" =~ ^[0-9]+$ ]] || { echo "free-tiers: no whole-number $2 for limit $1 in $LIMITS_FILE" >&2; exit 1; }
  echo "$value"
}

[ -f "$LIMITS_FILE" ] || { echo "free-tiers: $LIMITS_FILE not found" >&2; exit 1; }
PAGES_PROJECTS_LIMIT=$(limit pages_projects free)
WORKERS_LIMIT=$(limit workers free)
CRON_TRIGGERS_LIMIT=$(limit cron_triggers free)
PAGES_PROJECTS_BUDGET=${FREE_TIERS_PAGES_PROJECTS_BUDGET:-$(limit pages_projects budget)}
WORKERS_BUDGET=${FREE_TIERS_WORKERS_BUDGET:-$(limit workers budget)}
CRON_TRIGGERS_BUDGET=${FREE_TIERS_CRON_TRIGGERS_BUDGET:-$(limit cron_triggers budget)}
# GitHub: 10 GB cache per repo (evicted, never billed, unless the limit is
# raised) and 500 MB artifact storage on the Free plan if the repo is ever
# private. Warn at our budget, well before either.
GITHUB_CACHE_WARN_BYTES=${FREE_TIERS_GITHUB_CACHE_WARN_BYTES:-$(limit github_cache_bytes budget)}
GITHUB_ARTIFACTS_WARN_BYTES=${FREE_TIERS_GITHUB_ARTIFACTS_WARN_BYTES:-$(limit github_artifact_bytes budget)}

clamp() { if [ "$1" -gt "$2" ]; then echo "$2"; else echo "$1"; fi; }
PAGES_PROJECTS_BUDGET=$(clamp "$PAGES_PROJECTS_BUDGET" "$PAGES_PROJECTS_LIMIT")
WORKERS_BUDGET=$(clamp "$WORKERS_BUDGET" "$WORKERS_LIMIT")
CRON_TRIGGERS_BUDGET=$(clamp "$CRON_TRIGGERS_BUDGET" "$CRON_TRIGGERS_LIMIT")

CF_API=${FREE_TIERS_CF_API:-https://api.cloudflare.com/client/v4}
GH_API=${FREE_TIERS_GH_API:-https://api.github.com}

die() { echo "free-tiers: $*" >&2; exit 1; }

# ---------------------------------------------------------------- check ----

check() {
  local root=${FREE_TIERS_ROOT:-$(git rev-parse --show-toplevel)}
  cd "$root"
  local errors=0
  err() { echo "error[$1] $2"; errors=$((errors + 1)); }

  # Standard runner labels: ubuntu-24.04, ubuntu-latest, ubuntu-24.04-arm,
  # windows-2025, macos-15 and so on. Larger runners have custom labels or a
  # size suffix (macos-15-xlarge) and are billed per minute.
  local std='^(ubuntu|windows|macos)-(latest|slim|[0-9]+(\.[0-9]+)?)(-arm)?$'
  local f line label
  for f in .github/workflows/*.yml .github/workflows/*.yaml; do
    [ -f "$f" ] || continue
    while IFS= read -r line; do
      label=$(sed -E 's/^[^:]*runs-on:[[:space:]]*//; s/[[:space:]]+#.*$//; s/^["'\'']//; s/["'\'']$//; s/[[:space:]]+$//' <<<"$line")
      [[ "$label" == *'${{'* ]] && continue  # set from a matrix or input; not checkable here
      [[ "$label" =~ $std ]] || err FT-001 "$f uses runner '$label'; only standard GitHub-hosted runners are free"
    done < <(grep -E '^[[:space:]]*runs-on:' "$f" || true)
    if grep -qiE 'RAPIDAPI' "$f"; then
      err FT-002 "$f uses the RapidAPI key; homer runs by hand only, to stay on the free plan"
    fi
  done

  local wranglers
  wranglers=$(git ls-files -- '*wrangler.toml' '*wrangler.jsonc' '*wrangler.json')
  local pages=0 workers=0 crons=0 n
  for f in $wranglers; do
    if grep -qE '^[[:space:]]*"?pages_build_output_dir"?' "$f"; then
      pages=$((pages + 1))
    else
      workers=$((workers + 1))
    fi
    # crons = ["0 6 * * 2", ...] (one line or several)
    n=$( (sed -n '/crons/,/]/p' "$f" | grep -oE '"[^"]*\*[^"]*"' || true) | wc -l)
    crons=$((crons + n))
    if grep -qE '^[[:space:]]*"?usage_model"?[[:space:]]*[:=]' "$f"; then
      err FT-005 "$f sets usage_model, which is a paid plan setting"
    fi
    if grep -qE '^[[:space:]]*\[limits\]|"limits"[[:space:]]*:' "$f"; then
      err FT-005 "$f sets [limits] (CPU limits above the free 10 ms need Workers Paid)"
    fi
    if grep -qE '^[[:space:]]*"?logpush"?[[:space:]]*[:=][[:space:]]*true' "$f"; then
      err FT-005 "$f turns on logpush, which needs Workers Paid"
    fi
  done
  [ "$pages" -le "$PAGES_PROJECTS_BUDGET" ] ||
    err FT-003 "$pages Cloudflare Pages projects in the repo; the budget is $PAGES_PROJECTS_BUDGET (free limit $PAGES_PROJECTS_LIMIT)"
  [ "$workers" -le "$WORKERS_BUDGET" ] ||
    err FT-003 "$workers Cloudflare Workers in the repo; the budget is $WORKERS_BUDGET (free limit $WORKERS_LIMIT)"
  [ "$crons" -le "$CRON_TRIGGERS_BUDGET" ] ||
    err FT-004 "$crons Cron Triggers in wrangler files; the budget is $CRON_TRIGGERS_BUDGET (free limit $CRON_TRIGGERS_LIMIT per account)"

  if [ "$errors" -gt 0 ]; then
    echo "free-tiers: $errors error(s). See standards/docs/free-tiers.md"
    return 1
  fi
  echo "free-tiers: ok ($pages Pages projects, $workers Workers, $crons Cron Triggers in the repo)"
}

# ---------------------------------------------------------- cloudflare ----

need_cf() {
  [ -n "${CLOUDFLARE_API_TOKEN:-}" ] || die "CLOUDFLARE_API_TOKEN is not set"
  [ -n "${CLOUDFLARE_ACCOUNT_ID:-}" ] || die "CLOUDFLARE_ACCOUNT_ID is not set"
}

cf_get() {
  curl -fsS -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
    "$CF_API/accounts/$CLOUDFLARE_ACCOUNT_ID/$1"
}

# Every Pages project name in the account, one per line.
cf_pages_projects() {
  local page=1 body count
  while :; do
    body=$(cf_get "pages/projects?per_page=10&page=$page") || die "could not list Cloudflare Pages projects"
    jq -e '.success == true' >/dev/null <<<"$body" || die "Cloudflare API error: $(jq -c '.errors' <<<"$body")"
    jq -r '.result[].name' <<<"$body"
    count=$(jq '.result | length' <<<"$body")
    [ "$count" -ge 10 ] || break
    page=$((page + 1))
    [ "$page" -le 20 ] || break  # 200 projects is past any limit; stop paging
  done
}

cf_workers_count() {
  local body
  body=$(cf_get "workers/scripts") || die "could not list Cloudflare Workers"
  jq -e '.success == true' >/dev/null <<<"$body" || die "Cloudflare API error: $(jq -c '.errors' <<<"$body")"
  jq '.result | length' <<<"$body"
}

pages_project() {
  [ -f wrangler.toml ] || die "no wrangler.toml in $(pwd)"
  local name
  name=$(sed -nE 's/^name *= *"([^"]+)".*/\1/p' wrangler.toml | head -n1)
  [ -n "$name" ] || die "wrangler.toml has no name"
  need_cf
  local projects count
  projects=$(cf_pages_projects)
  if grep -qxF "$name" <<<"$projects"; then
    echo "free-tiers: Pages project $name exists"
    return 0
  fi
  count=$(grep -c . <<<"$projects" || true)
  if [ "$count" -ge "$PAGES_PROJECTS_BUDGET" ]; then
    die "not creating Pages project $name: the account has $count Pages projects and the budget is $PAGES_PROJECTS_BUDGET (free limit $PAGES_PROJECTS_LIMIT). Delete unused projects or ask Jim; see standards/docs/free-tiers.md"
  fi
  echo "free-tiers: creating Pages project $name ($((count + 1)) of $PAGES_PROJECTS_BUDGET budgeted)"
  wrangler pages project create "$name" --production-branch main
}

# --------------------------------------------------------------- usage ----

usage_check() {
  local rows=() over=0 value status

  row() {  # name, used, warn-at, limit, note
    status=ok
    if [ "$2" = "?" ]; then
      status=skipped
    elif [ "$2" -ge "$3" ]; then
      status=OVER
      over=$((over + 1))
    fi
    rows+=("| $1 | $2 | $3 | $4 | $status | $5 |")
  }

  if [ -n "${CLOUDFLARE_API_TOKEN:-}" ] && [ -n "${CLOUDFLARE_ACCOUNT_ID:-}" ]; then
    local projects
    projects=$(cf_pages_projects)  # exits the script if the API call fails
    value=$(grep -c . <<<"$projects" || true)
    row "Cloudflare Pages projects" "$value" "$PAGES_PROJECTS_BUDGET" "$PAGES_PROJECTS_LIMIT" "budget, not the hard limit"
    # The deploy token may not have Workers Scripts read access yet; report
    # that rather than failing the whole check.
    if value=$(cf_workers_count 2>/dev/null); then
      row "Cloudflare Workers" "$value" "$WORKERS_BUDGET" "$WORKERS_LIMIT" "budget, not the hard limit"
    else
      row "Cloudflare Workers" "?" "$WORKERS_BUDGET" "$WORKERS_LIMIT" "token cannot list Workers (needs Workers Scripts: Read)"
    fi
  else
    row "Cloudflare" "?" 0 - "CLOUDFLARE_API_TOKEN or CLOUDFLARE_ACCOUNT_ID not set"
  fi

  if [ -n "${GH_TOKEN:-}" ] && [ -n "${GITHUB_REPOSITORY:-}" ]; then
    local gh=(curl -fsS -H "Authorization: Bearer $GH_TOKEN" -H "Accept: application/vnd.github+json")
    value=$("${gh[@]}" "$GH_API/repos/$GITHUB_REPOSITORY/actions/cache/usage" | jq '.active_caches_size_in_bytes // 0') ||
      die "could not read the GitHub Actions cache usage"
    row "GitHub Actions cache (bytes)" "$value" "$GITHUB_CACHE_WARN_BYTES" "10 GB" "evicted past 10 GB, not billed"
    local page=1 total=0 body n
    while :; do
      body=$("${gh[@]}" "$GH_API/repos/$GITHUB_REPOSITORY/actions/artifacts?per_page=100&page=$page") ||
        die "could not list GitHub Actions artifacts"
      total=$((total + $(jq '[.artifacts[] | select(.expired | not) | .size_in_bytes] | add // 0' <<<"$body")))
      n=$(jq '.artifacts | length' <<<"$body")
      [ "$n" -ge 100 ] || break
      page=$((page + 1))
      [ "$page" -le 50 ] || break
    done
    row "GitHub Actions artifacts (bytes)" "$total" "$GITHUB_ARTIFACTS_WARN_BYTES" "500 MB" "free on a public repo; the limit applies if it goes private"
  else
    row "GitHub" "?" 0 - "GH_TOKEN or GITHUB_REPOSITORY not set"
  fi

  {
    echo "### Free tier usage"
    echo
    echo "| Item | Used | Warn at | Free limit | Status | Note |"
    echo "| --- | --- | --- | --- | --- | --- |"
    printf '%s\n' "${rows[@]}"
    echo
    echo "Limits and what to do when one is reached: standards/docs/free-tiers.md"
  } | tee -a "${GITHUB_STEP_SUMMARY:-/dev/null}"

  if [ "$over" -gt 0 ]; then
    echo "::error::$over item(s) at or over their free tier warning level. See standards/docs/free-tiers.md"
    return 1
  fi
}

case "${1:-}" in
  check) check ;;
  pages-project) pages_project ;;
  usage) usage_check ;;
  *) die "usage: free-tiers.sh check | pages-project | usage" ;;
esac
