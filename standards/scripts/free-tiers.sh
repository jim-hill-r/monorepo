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
#         FT-006  Every Pages project is named jimhillr-<project>, because
#                 *.pages.dev names are global (standards/docs/naming.md).
#
#   free-tiers.sh pages-project
#       Run from a Cloudflare project directory (the deploy task does this).
#       Creates the Pages project named in ./wrangler.toml if it does not
#       exist, but only while the account stays within the Pages project
#       budget and the name has the jimhillr- prefix. Otherwise it stops with
#       an error and creates nothing.
#
#   free-tiers.sh worker
#       Run from a Worker project directory before its first deploy. Passes
#       if the Worker named in ./wrangler.toml exists, or if one more stays
#       within the Workers budget; otherwise stops.
#
#   free-tiers.sh d1-database NAME
#       Prints the ID of the D1 database NAME, creating it first if it is
#       missing and the account has fewer than the free limit of databases.
#       Otherwise it stops and creates nothing. Messages go to stderr.
#
#   free-tiers.sh usage
#       Live usage check (Cloudflare and GitHub), run daily. Reads Pages
#       projects, Workers, Cron Triggers, D1 databases and storage, R2
#       buckets, the busiest day's Workers requests and D1 rows (GraphQL
#       analytics), repo visibility, and Actions cache and artifacts. Rates
#       each green, yellow or red (the ladder in level() below), writes a
#       table to $GITHUB_STEP_SUMMARY and, with FREE_TIERS_JSON set, the
#       same rows as JSON. Yellow is a warning; any red exits non-zero, so
#       the scheduled run goes red.
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
D1_DATABASES_LIMIT=$(limit d1_databases free)
D1_STORAGE_LIMIT_BYTES=$(limit d1_storage_bytes free)
D1_ROWS_READ_LIMIT=$(limit d1_rows_read_per_day free)
D1_ROWS_WRITTEN_LIMIT=$(limit d1_rows_written_per_day free)
WORKERS_REQUESTS_LIMIT=$(limit workers_requests_per_day free)
GITHUB_CACHE_LIMIT_BYTES=$(limit github_cache_bytes free)
GITHUB_ARTIFACTS_LIMIT_BYTES=$(limit github_artifact_bytes free)
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

# Pages project names: jimhillr-<project>, lowercase letters, digits and
# hyphens (standards/docs/naming.md, "Cloudflare Pages projects").
PAGES_NAME_RE='^jimhillr-[a-z0-9]+(-[a-z0-9]+)*$'
wrangler_name() { sed -nE 's/^[[:space:]]*"?name"?[[:space:]]*[:=][[:space:]]*"([^"]+)".*/\1/p' "$1" | head -n1; }

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
      n=$(wrangler_name "$f")
      [[ "$n" =~ $PAGES_NAME_RE ]] ||
        err FT-006 "$f names Pages project '$n'; Pages projects must be named jimhillr-<project> (standards/docs/naming.md)"
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
  name=$(wrangler_name wrangler.toml)
  [ -n "$name" ] || die "wrangler.toml has no name"
  [[ "$name" =~ $PAGES_NAME_RE ]] ||
    die "not creating Pages project $name: Pages projects must be named jimhillr-<project> (standards/docs/naming.md)"
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

# Every Worker script name in the account, one per line.
cf_workers() {
  local body
  body=$(cf_get "workers/scripts") || die "could not list Cloudflare Workers"
  jq -e '.success == true' >/dev/null <<<"$body" || die "Cloudflare API error: $(jq -c '.errors' <<<"$body")"
  jq -r '.result[].id' <<<"$body"
}

worker() {
  [ -f wrangler.toml ] || die "no wrangler.toml in $(pwd)"
  local name workers count
  name=$(wrangler_name wrangler.toml)
  [ -n "$name" ] || die "wrangler.toml has no name"
  need_cf
  workers=$(cf_workers)
  if grep -qxF "$name" <<<"$workers"; then
    echo "free-tiers: Worker $name exists"
    return 0
  fi
  count=$(grep -c . <<<"$workers" || true)
  if [ "$count" -ge "$WORKERS_BUDGET" ]; then
    die "not deploying new Worker $name: the account has $count Workers and the budget is $WORKERS_BUDGET (free limit $WORKERS_LIMIT). Delete unused Workers or ask Jim; see standards/docs/free-tiers.md"
  fi
  echo "free-tiers: Worker $name will be created ($((count + 1)) of $WORKERS_BUDGET budgeted)"
}

# "<uuid> <name>" for every D1 database in the account, one per line.
cf_d1_databases() {
  local body
  body=$(cf_get "d1/database?per_page=100") || die "could not list D1 databases"
  jq -e '.success == true' >/dev/null <<<"$body" || die "Cloudflare API error: $(jq -c '.errors' <<<"$body")"
  jq -r '.result[] | "\(.uuid) \(.name)"' <<<"$body"
}

d1_database() {
  local name=${1:-} dbs id count
  [[ "$name" =~ ^jimhillr-[a-z0-9-]+$ ]] || die "D1 database names are jimhillr-<name>, not '$name'"
  need_cf
  dbs=$(cf_d1_databases)
  id=$(awk -v n="$name" '$2 == n { print $1; exit }' <<<"$dbs")
  if [ -n "$id" ]; then
    echo "free-tiers: D1 database $name exists" >&2
    echo "$id"
    return 0
  fi
  count=$(grep -c . <<<"$dbs" || true)
  if [ "$count" -ge "$D1_DATABASES_LIMIT" ]; then
    die "not creating D1 database $name: the account has $count and the free limit is $D1_DATABASES_LIMIT. See standards/docs/free-tiers.md"
  fi
  echo "free-tiers: creating D1 database $name ($((count + 1)) of $D1_DATABASES_LIMIT free)" >&2
  wrangler d1 create "$name" >&2
  id=$(cf_d1_databases | awk -v n="$name" '$2 == n { print $1; exit }')
  [ -n "$id" ] || die "created D1 database $name but can't find its ID"
  echo "$id"
}

# --------------------------------------------------------------- usage ----

# level USED BUDGET FREE: green, yellow or red on the ladder in
# standards/docs/plans/2026-11-cost-tracking.md ("Thresholds"). With a
# budget: yellow from 80% of it, red from 100%. Without one ("-"): yellow
# from 50% of the free limit, red from 80%. A budget of 0 means "never":
# any use at all is red. Prints "yellow_at red_at level".
level() {
  local used=$1 budget=$2 free=$3 yellow red
  if [ "$budget" != "-" ]; then
    yellow=$(( (budget * 80 + 99) / 100 )); red=$budget
    [ "$budget" -gt 0 ] || { yellow=1; red=1; }
  elif [ "$free" != "-" ]; then
    yellow=$(( (free * 50 + 99) / 100 )); red=$(( (free * 80 + 99) / 100 ))
  else
    echo "- - green"; return
  fi
  if [ "$used" -ge "$red" ]; then echo "$yellow $red red"
  elif [ "$used" -ge "$yellow" ]; then echo "$yellow $red yellow"
  else echo "$yellow $red green"; fi
}

# cf_graphql QUERY: POST a GraphQL query (no variables) to Cloudflare's
# analytics API. Needs Account Analytics Read, which the deploy token has.
cf_graphql() {
  curl -fsS -X POST -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
    -H "Content-Type: application/json" \
    --data "$(jq -nc --arg q "$1" '{query: $q}')" "$CF_API/graphql"
}

# The busiest day of the last seven (daily limits reset at 00:00 UTC):
# "<workers requests> <d1 rows read> <d1 rows written>", Pages Functions
# counted with Workers as Cloudflare does.
cf_busiest_day() {
  local since body
  since=$(date -u -d '7 days ago' +%Y-%m-%d 2>/dev/null || date -u -v-7d +%Y-%m-%d)
  body=$(cf_graphql "{ viewer { accounts(filter: {accountTag: \"$CLOUDFLARE_ACCOUNT_ID\"}) {
    workers: workersInvocationsAdaptive(limit: 10000, filter: {date_geq: \"$since\"}) { sum { requests } dimensions { date } }
    pages: pagesFunctionsInvocationsAdaptiveGroups(limit: 10000, filter: {date_geq: \"$since\"}) { sum { requests } dimensions { date } }
    d1: d1AnalyticsAdaptiveGroups(limit: 10000, filter: {date_geq: \"$since\"}) { sum { rowsRead rowsWritten } dimensions { date } }
  } } }") || return 1
  jq -e '(.errors // []) | length == 0' >/dev/null <<<"$body" || return 1
  jq -r '
    .data.viewer.accounts[0] as $a
    | def busiest(groups; f): [groups // [] | group_by(.dimensions.date)[] | map(f) | add] | max // 0;
      "\(busiest(($a.workers // []) + ($a.pages // []); .sum.requests))"
      + " \(busiest($a.d1; .sum.rowsRead)) \(busiest($a.d1; .sum.rowsWritten))"' <<<"$body"
}

# Cron Triggers on every Worker in the account (Pages can't have any).
cf_cron_count() {
  local names name body total=0
  names=$(cf_workers) || return 1
  for name in $names; do
    body=$(cf_get "workers/scripts/$name/schedules") || return 1
    total=$((total + $(jq '.result.schedules // [] | length' <<<"$body")))
  done
  echo "$total"
}

# R2 buckets in the account. R2 not enabled at all answers error 10042,
# which is what we want: 0 buckets.
cf_r2_buckets() {
  local body
  body=$(curl -sS -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
    "$CF_API/accounts/$CLOUDFLARE_ACCOUNT_ID/r2/buckets") || return 1
  if jq -e '.success == true' >/dev/null <<<"$body"; then
    jq '.result.buckets // [] | length' <<<"$body"
  elif jq -e '[.errors[]?.code] | index(10042)' >/dev/null <<<"$body"; then
    echo 0
  else
    return 1
  fi
}

usage_check() {
  local rows=() items=() red=0 yellow=0 value status

  # row NAME USED BUDGET FREE NOTE [FREE_SHOWN]: one line of the table.
  # USED is a whole number or "?" (not read, so skipped). BUDGET and FREE
  # are whole numbers or "-" for none. FREE_SHOWN is how the free limit is
  # printed (say "10 GB"), FREE itself by default.
  row() {
    local yellow_at=- red_at=- shown=${6:-$4} l
    if [ "$2" = "?" ]; then
      status=skipped
    else
      read -r yellow_at red_at status < <(level "$2" "$3" "$4")
      case $status in
        red) red=$((red + 1)) ;;
        yellow) yellow=$((yellow + 1)); echo "::warning::$1 is at $2, past its yellow level of $yellow_at. See standards/docs/free-tiers.md" ;;
      esac
    fi
    rows+=("| $1 | $2 | $yellow_at | $red_at | $shown | $status | $5 |")
    items+=("$(jq -nc --arg item "$1" --arg used "$2" --arg budget "$3" --arg free "$4" \
      --arg yellow "$yellow_at" --arg red "$red_at" --arg limit "$shown" \
      --arg status "$status" --arg note "$5" \
      '{item: $item, used: ($used | tonumber? // null),
        budget: ($budget | tonumber? // null), free: ($free | tonumber? // null),
        yellow_at: ($yellow | tonumber? // null), red_at: ($red | tonumber? // null),
        limit: $limit, status: $status, note: $note}')")
  }

  if [ -n "${CLOUDFLARE_API_TOKEN:-}" ] && [ -n "${CLOUDFLARE_ACCOUNT_ID:-}" ]; then
    local projects
    projects=$(cf_pages_projects)  # exits the script if the API call fails
    value=$(grep -c . <<<"$projects" || true)
    row "Cloudflare Pages projects" "$value" "$PAGES_PROJECTS_BUDGET" "$PAGES_PROJECTS_LIMIT" "budget, not the hard limit"
    # The token may not have every read permission; a read it can't make is
    # skipped with a note rather than failing the whole check.
    if value=$(cf_workers_count 2>/dev/null); then
      row "Cloudflare Workers" "$value" "$WORKERS_BUDGET" "$WORKERS_LIMIT" "budget, not the hard limit"
    else
      row "Cloudflare Workers" "?" "$WORKERS_BUDGET" "$WORKERS_LIMIT" "token cannot list Workers (needs Workers Scripts: Read)"
    fi
    if value=$(cf_cron_count 2>/dev/null); then
      row "Cron Triggers" "$value" "$CRON_TRIGGERS_BUDGET" "$CRON_TRIGGERS_LIMIT" "per account"
    else
      row "Cron Triggers" "?" "$CRON_TRIGGERS_BUDGET" "$CRON_TRIGGERS_LIMIT" "token cannot read Worker schedules (needs Workers Scripts: Read)"
    fi
    local dbs
    if dbs=$(cf_get "d1/database?per_page=100" 2>/dev/null) && jq -e '.success == true' >/dev/null <<<"$dbs"; then
      row "D1 databases" "$(jq '.result | length' <<<"$dbs")" - "$D1_DATABASES_LIMIT" "no budget, so the ladder uses the free limit"
      row "D1 storage (bytes)" "$(jq '[.result[].file_size // 0] | add // 0' <<<"$dbs")" - "$D1_STORAGE_LIMIT_BYTES" "all databases together" "5 GB"
    else
      row "D1 databases" "?" - "$D1_DATABASES_LIMIT" "token cannot list D1 databases (needs D1: Read)"
    fi
    if value=$(cf_r2_buckets 2>/dev/null); then
      row "R2 buckets" "$value" 0 - "R2 is on hold (budget 0); a bucket means R2 is on, an overrun switch"
    else
      row "R2 buckets" "?" 0 - "token cannot list R2 buckets (needs Workers R2 Storage: Read)"
    fi
    local busiest rows_read rows_written
    if busiest=$(cf_busiest_day 2>/dev/null) && [ -n "$busiest" ]; then
      read -r value rows_read rows_written <<<"$busiest"
      row "Workers requests, busiest day of 7" "$value" - "$WORKERS_REQUESTS_LIMIT" "includes Pages Functions; resets 00:00 UTC"
      row "D1 rows read, busiest day of 7" "$rows_read" - "$D1_ROWS_READ_LIMIT" "resets 00:00 UTC"
      row "D1 rows written, busiest day of 7" "$rows_written" - "$D1_ROWS_WRITTEN_LIMIT" "resets 00:00 UTC"
    else
      row "Workers and D1 daily use" "?" - "$WORKERS_REQUESTS_LIMIT" "analytics API not readable (needs Account Analytics: Read)"
    fi
  else
    row "Cloudflare" "?" - - "CLOUDFLARE_API_TOKEN or CLOUDFLARE_ACCOUNT_ID not set"
  fi

  if [ -n "${GH_TOKEN:-}" ] && [ -n "${GITHUB_REPOSITORY:-}" ]; then
    local gh=(curl -fsS -H "Authorization: Bearer $GH_TOKEN" -H "Accept: application/vnd.github+json")
    # A private repo starts counting Actions minutes against a limit, so
    # the repo going private is an overrun switch: red at once.
    value=$("${gh[@]}" "$GH_API/repos/$GITHUB_REPOSITORY" | jq 'if .private then 1 else 0 end') ||
      die "could not read the repository"
    row "GitHub repo private (1 = yes)" "$value" 0 - "an overrun switch: Actions minutes are billed on a private repo"
    value=$("${gh[@]}" "$GH_API/repos/$GITHUB_REPOSITORY/actions/cache/usage" | jq '.active_caches_size_in_bytes // 0') ||
      die "could not read the GitHub Actions cache usage"
    row "GitHub Actions cache (bytes)" "$value" "$GITHUB_CACHE_WARN_BYTES" "$GITHUB_CACHE_LIMIT_BYTES" "evicted past 10 GB, not billed" "10 GB"
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
    row "GitHub Actions artifacts (bytes)" "$total" "$GITHUB_ARTIFACTS_WARN_BYTES" "$GITHUB_ARTIFACTS_LIMIT_BYTES" "free on a public repo; the limit applies if it goes private" "500 MB"
  else
    row "GitHub" "?" - - "GH_TOKEN or GITHUB_REPOSITORY not set"
  fi

  {
    echo "### Free tier usage"
    echo
    echo "Yellow from 80% of our budget (50% of the free limit where there is no budget), red from 100% (80%)."
    echo
    echo "| Item | Used | Yellow at | Red at | Free limit | Status | Note |"
    echo "| --- | --- | --- | --- | --- | --- | --- |"
    printf '%s\n' "${rows[@]}"
    echo
    echo "Limits and what to do when one is reached: standards/docs/free-tiers.md"
  } | tee -a "${GITHUB_STEP_SUMMARY:-/dev/null}"

  # The same rows as JSON, for pane's Money page (pane/issues: order 340).
  # The workflow uploads this file as the run's `free-tier-usage` artifact.
  if [ -n "${FREE_TIERS_JSON:-}" ]; then
    printf '%s\n' "${items[@]}" | jq -s \
      --arg taken_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --argjson red "$red" --argjson yellow "$yellow" \
      '{version: 2, taken_at: $taken_at, red: $red, yellow: $yellow, items: .}' >"$FREE_TIERS_JSON"
  fi

  if [ "$red" -gt 0 ]; then
    echo "::error::$red item(s) red: at or over their budget, or past 80% of a free limit with no budget. See standards/docs/free-tiers.md"
    return 1
  fi
}

case "${1:-}" in
  check) check ;;
  pages-project) pages_project ;;
  worker) worker ;;
  d1-database) d1_database "${2:-}" ;;
  usage) usage_check ;;
  *) die "usage: free-tiers.sh check | pages-project | worker | d1-database NAME | usage" ;;
esac
