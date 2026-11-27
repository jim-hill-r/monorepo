#!/usr/bin/env bash
# Export fan feedback (the telemetry Worker's `feedback` and `exposure`
# events) from Analytics Engine into files on the feedback-data branch, so
# the repo, not Analytics Engine, is the record (files before databases;
# bear_down/docs/plans/2026-11-fan-feedback.md, step 5).
#
#   feedback.sh query PROJECT [DAYS]
#       Print the project's feedback and exposure events from the last DAYS
#       days (default 8) as JSON Lines, oldest first. Needs
#       CLOUDFLARE_API_TOKEN (Account Analytics read) and
#       CLOUDFLARE_ACCOUNT_ID. FEEDBACK_CF_API overrides the API base URL.
#
#   feedback.sh summarize FILE.jsonl
#       Print a Markdown summary of one week's events, per A/B arm.
#
#   feedback.sh store PROJECT ROWS.jsonl
#       Merge ROWS into feedback-data:<project>/<season>-week-<nn>.jsonl
#       (one line per event, duplicates dropped), rewrite each touched
#       week's <season>-week-<nn>.md summary, and push. Its only push is to
#       feedback-data, never main (SEC-007). Retries a lost push race.
#
# Run weekly by .github/workflows/telemetry-feedback-export.yml; tested by
# standards/tests/feedback.test.sh.
set -uo pipefail

root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
CF_API=${FEEDBACK_CF_API:-https://api.cloudflare.com/client/v4}
branch=feedback-data

die() { echo "feedback: $*" >&2; exit 1; }

check_project() {
  [[ "${1:-}" =~ ^[a-z0-9_.-]{1,32}$ ]] || die "project must be letters, digits, _, - or . (got '${1:-}')"
}

query() {
  local project=$1 days=${2:-8} body
  check_project "$project"
  [[ "$days" =~ ^[0-9]{1,3}$ ]] || die "days must be a number (got '$days')"
  [ -n "${CLOUDFLARE_API_TOKEN:-}" ] || die "CLOUDFLARE_API_TOKEN is not set"
  [ -n "${CLOUDFLARE_ACCOUNT_ID:-}" ] || die "CLOUDFLARE_ACCOUNT_ID is not set"
  # Column layout: telemetry/README.md, "What is stored".
  local sql="SELECT timestamp, blob1 AS kind, blob4 AS arm, blob5 AS tab, blob6 AS game, blob7 AS verdict, blob8 AS vote, blob9 AS note, double1 AS season, double2 AS week FROM telemetry WHERE index1 = '$project' AND blob1 IN ('feedback', 'exposure') AND timestamp > NOW() - INTERVAL '$days' DAY ORDER BY timestamp FORMAT JSON"
  body=$(curl -fsS -X POST -H "Authorization: Bearer $CLOUDFLARE_API_TOKEN" \
    --data-binary "$sql" \
    "$CF_API/accounts/$CLOUDFLARE_ACCOUNT_ID/analytics_engine/sql") \
    || die "the Analytics Engine query failed (the token needs Account Analytics read)"
  jq -c '.data[]
    | {ts: .timestamp, kind, arm, season: (.season | tonumber | floor),
       week: (.week | tonumber | floor), tab}
      + (if .kind == "feedback" then {game, verdict, vote} else {} end)
      + (if .kind == "feedback" and (.note // "") != "" then {note} else {} end)' <<<"$body" \
    || die "unexpected Analytics Engine answer: $(head -c 300 <<<"$body")"
}

summarize() {
  local file=$1
  jq -rs '
    def pct(a; b): if b == 0 then "-" else "\((a * 100 / b) | round)%" end;
    def nopick: . == "doesnt_matter" or . == "too_close" or . == "cant_matter";
    (map(select(.kind == "feedback" and .vote != "note"))) as $votes
    | (map(select(.kind == "exposure"))) as $seen
    | (map(select(.kind == "feedback" and .vote == "note"))) as $notes
    | (.[0] // {}) as $first
    | "# Fan feedback, \($first.season // "?") week \($first.week // "?")",
      "",
      "Votes on \"Does this verdict look right?\" per A/B arm. Arm A uses the",
      "published 0.05-point \"does not matter\" line, arm B 0.02 points",
      "(bear_down/docs/plans/2026-11-fan-feedback.md).",
      "",
      "| Arm | Visits (tab views) | Votes | Yes | No | Confusing | Yes share | Notes |",
      "| --- | --- | --- | --- | --- | --- | --- | --- |",
      (["a", "b"][] as $arm
        | ($votes | map(select(.arm == $arm))) as $v
        | ($v | map(select(.vote == "yes")) | length) as $yes
        | "| \($arm | ascii_upcase) | \($seen | map(select(.arm == $arm)) | length) | \($v | length) | \($yes) | \($v | map(select(.vote == "no")) | length) | \($v | map(select(.vote == "confusing")) | length) | \(pct($yes; $v | length)) | \($notes | map(select(.arm == $arm)) | length) |"),
      "",
      "## By verdict",
      "",
      "| Arm | Verdict | Votes | Yes | No | Confusing |",
      "| --- | --- | --- | --- | --- | --- |",
      (["a", "b"][] as $arm | ["pick", "no pick"][] as $class
        | ($votes | map(select(.arm == $arm and ((.verdict | nopick) == ($class == "no pick"))))) as $v
        | "| \($arm | ascii_upcase) | \($class) | \($v | length) | \($v | map(select(.vote == "yes")) | length) | \($v | map(select(.vote == "no")) | length) | \($v | map(select(.vote == "confusing")) | length) |"),
      "",
      "## Notes",
      "",
      (if ($notes | length) == 0 then "None this week."
       else ($notes[] | "- \(.arm | ascii_upcase), \(.tab), \(.game) (\(.verdict)): \(.note | gsub("[\r\n]+"; " "))")
       end)
  ' "$file"
}

store() {
  local project=$1 rows=$2 work attempt message weeks key
  check_project "$project"
  [ -f "$rows" ] || die "no rows file $rows"
  rows=$(realpath "$rows")
  if [ ! -s "$rows" ]; then
    echo "feedback: no events to store"
    return 0
  fi
  message="feedback: $project export ${GITHUB_RUN_ID:-local}"
  work=$(mktemp -d)
  trap 'git -C "$root" worktree remove --force "$work" 2>/dev/null || true' EXIT
  for attempt in 1 2 3 4 5; do
    git -C "$root" worktree remove --force "$work" 2>/dev/null || rm -rf "$work"
    if git -C "$root" ls-remote --exit-code --heads origin "$branch" >/dev/null; then
      git -C "$root" fetch --quiet --depth=1 origin "+refs/heads/$branch:refs/remotes/origin/$branch" || return 1
      git -C "$root" worktree add --quiet --detach "$work" "origin/$branch" || return 1
    else
      # First run: an orphan branch that shares no history with main.
      git -C "$root" worktree add --quiet --detach "$work" || return 1
      git -C "$work" checkout --quiet --orphan "$branch"
      git -C "$work" rm -rfq --ignore-unmatch .
    fi
    mkdir -p "$work/$project"
    weeks=$(jq -r '"\(.season)-week-\(.week | tostring | if length < 2 then "0" + . else . end)"' "$rows" | sort -u)
    for key in $weeks; do
      local file="$work/$project/$key.jsonl"
      {
        [ -f "$file" ] && cat "$file"
        jq -c --arg key "$key" 'select("\(.season)-week-\(.week | tostring | if length < 2 then "0" + . else . end)" == $key)' "$rows"
      } | jq -sc 'unique_by([.ts, .kind, .arm, .tab, .game, .vote, .note]) | sort_by(.ts) | .[]' >"$file.new"
      mv "$file.new" "$file"
      summarize "$file" >"$work/$project/$key.md"
    done
    cat >"$work/README.md" <<'MD'
# feedback-data

Data branch written by `telemetry-feedback-export.yml` through
`standards/scripts/feedback.sh store`: each week's fan votes and A/B
exposures as JSON Lines (`<project>/<season>-week-<nn>.jsonl`), with a
Markdown summary per arm beside each. Never merge it into `main`.
See `bear_down/docs/plans/2026-11-fan-feedback.md` on `main`.
MD
    git -C "$work" add -A
    if git -C "$work" diff --cached --quiet; then
      echo "feedback: nothing new for $project"
      return 0
    fi
    git -C "$work" -c user.name='github-actions[bot]' \
      -c user.email='41898282+github-actions[bot]@users.noreply.github.com' \
      commit --quiet -m "$message"
    if git -C "$work" push --quiet origin HEAD:refs/heads/feedback-data; then
      echo "feedback: pushed \"$message\" to $branch"
      return 0
    fi
    echo "feedback: push to $branch lost a race (attempt $attempt); retrying"
    sleep $((attempt * 5))
  done
  die "could not push to $branch after 5 attempts"
}

case "${1:-}" in
  query) query "${2:-}" "${3:-8}" ;;
  summarize) [ -n "${2:-}" ] || die "usage: feedback.sh summarize FILE.jsonl"; summarize "$2" ;;
  store) store "${2:-}" "${3:-}" ;;
  *) echo "usage: feedback.sh query PROJECT [DAYS] | summarize FILE.jsonl | store PROJECT ROWS.jsonl" >&2; exit 2 ;;
esac
