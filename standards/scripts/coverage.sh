#!/usr/bin/env bash
# Code coverage (COV-001, standards/docs/testing.md#code-coverage).
#
#   coverage.sh run
#       In a project directory: runs its tests under cargo-llvm-cov and
#       writes one JSON line {project, commit, date, lines: {count, covered,
#       percent}} to $COVERAGE_RESULT (default target/coverage/coverage.json).
#       A project without a Cargo.toml has nothing to measure and is skipped.
#       Line coverage under $COVERAGE_MIN percent (default 80) is a warning,
#       never a failure, until Jim makes COV-001 an error.
#   coverage.sh projects
#       Prints the projects to measure as a JSON list, for the workflow's
#       matrix: each project with a Cargo.toml, except one inside another
#       project that has one (it is a member of that Cargo workspace, so
#       measuring both would count its lines twice).
#   coverage.sh badge <label> <percent|none>
#       Prints a flat SVG badge: green from 80%, yellow from 60%, red below.
#   coverage.sh summarize <results.json...>
#       Prints the repo summary: every project's result and the total line
#       coverage over all of them (covered lines / lines).
#   coverage.sh store <results.json...>
#       Commits the results to the long-lived coverage-results data branch:
#       projects/<project>.json (latest), history/<yyyy-mm>.jsonl (every run),
#       summary.json, and the README badges badge.svg (total) and
#       badges/<project>.svg. nightly-coverage.yml runs it with contents:
#       write; like evals.sh store, its only push is to that branch (SEC-007),
#       never to main.
set -uo pipefail

root=$(git rev-parse --show-toplevel)
min=${COVERAGE_MIN:-80}
# cargo-llvm-cov comes from PATH, or else mise downloads its release (`github:taiki-e/cargo-llvm-cov`).
# Pinned here, not in mise.toml [tools], so only coverage runs install it.
CARGO_LLVM_COV_VERSION=0.9.2

color() {
  awk -v p="$1" 'BEGIN { if (p == "none") print "#9f9f9f"; else if (p >= 80) print "#4c1"; else if (p >= 60) print "#dfb317"; else print "#e05d44" }'
}

badge() {
  local label=$1 pct=$2 value fill lw vw
  if [ "$pct" = none ]; then value="no data"; else value=$(printf '%.0f%%' "$pct"); fi
  fill=$(color "$pct")
  # About 6.5px per character in Verdana 11px, plus padding.
  lw=$(( ${#label} * 13 / 2 + 10 ))
  vw=$(( ${#value} * 13 / 2 + 10 ))
  cat <<SVG
<svg xmlns="http://www.w3.org/2000/svg" width="$((lw + vw))" height="20" role="img" aria-label="$label: $value"><title>$label: $value</title><linearGradient id="s" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient><clipPath id="r"><rect width="$((lw + vw))" height="20" rx="3" fill="#fff"/></clipPath><g clip-path="url(#r)"><rect width="$lw" height="20" fill="#555"/><rect x="$lw" width="$vw" height="20" fill="$fill"/><rect width="$((lw + vw))" height="20" fill="url(#s)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" font-size="11"><text x="$((lw / 2))" y="14">$label</text><text x="$((lw + vw / 2))" y="14">$value</text></g></svg>
SVG
}

summarize() {
  cat "$@" | jq -s '
    (map(.lines.count) | add // 0) as $count
    | (map(.lines.covered) | add // 0) as $covered
    | {
        lines: {
          count: $count,
          covered: $covered,
          percent: (if $count > 0 then ($covered * 10000 / $count | round) / 100 else null end)
        },
        projects: (sort_by(.project) | map({project, percent: .lines.percent}))
      }'
}

slug() { echo "${1//\//-}"; }

case "${1:-}" in
  run)
    if [ ! -f Cargo.toml ]; then
      echo "coverage: no Cargo.toml in $PWD, nothing to measure"
      exit 0
    fi
    project=${PROJECT:-$(realpath --relative-to="$root" "$PWD")}
    result=${COVERAGE_RESULT:-target/coverage/coverage.json}
    mkdir -p "$(dirname "$result")"
    raw=$(mktemp)
    if command -v rustup >/dev/null; then
      rustup component add llvm-tools-preview >/dev/null 2>&1 || true
    fi
    if cargo llvm-cov --version >/dev/null 2>&1; then
      cmd=(cargo llvm-cov)
    else
      cmd=(mise exec "github:taiki-e/cargo-llvm-cov@$CARGO_LLVM_COV_VERSION" -- cargo llvm-cov)
    fi
    # The same tests as the project's ci task (`cargo test`, so a workspace's
    # default members), and bevy games need a display.
    cmd+=(--summary-only --json --output-path "$raw")
    if command -v xvfb-run >/dev/null; then cmd=(xvfb-run -a "${cmd[@]}"); fi
    if ! "${cmd[@]}"; then
      echo "coverage: tests failed under cargo llvm-cov in $project" >&2
      exit 1
    fi
    jq -c --arg project "$project" --arg commit "${GITHUB_SHA:-$(git rev-parse HEAD)}" \
      --arg date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '
      .data[0].totals.lines as $l
      | {project: $project, commit: $commit, date: $date,
         lines: {count: $l.count, covered: $l.covered, percent: (($l.percent * 100 | round) / 100)}}' \
      "$raw" >"$result"
    rm -f "$raw"
    pct=$(jq -r '.lines.percent' "$result")
    echo "coverage: $project $pct% of lines"
    if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
      { echo "| Project | Line coverage | Lines |"; echo "| --- | --- | --- |"; } >>"$GITHUB_STEP_SUMMARY"
      echo "| $project | $pct% | $(jq -r '"\(.lines.covered) of \(.lines.count)"' "$result") |" >>"$GITHUB_STEP_SUMMARY"
    fi
    if awk -v p="$pct" -v m="$min" 'BEGIN { exit !(p < m) }'; then
      echo "::warning::COV-001 $project line coverage is $pct%, under the $min% minimum (standards/docs/testing.md)"
    fi
    ;;
  projects)
    cd "$root"
    rust=$(mise run --quiet //:projects | jq -r '.[]' | while read -r p; do
      if [ -f "$p/Cargo.toml" ]; then echo "$p"; fi
    done)
    while read -r p; do
      [ -n "$p" ] || continue
      nested=0
      while read -r q; do
        if [ -n "$q" ] && [[ "$p" == "$q/"* ]]; then nested=1; fi
      done <<<"$rust"
      if [ "$nested" -eq 0 ]; then echo "$p"; fi
    done <<<"$rust" | jq -R . | jq -cs .
    ;;
  badge)
    badge "${2:?label}" "${3:?percent}"
    ;;
  summarize)
    shift
    summarize "$@"
    ;;
  store)
    shift
    files=()
    for f in "$@"; do
      [ -s "$f" ] && files+=("$(realpath "$f")")
    done
    if [ ${#files[@]} -eq 0 ]; then
      echo "coverage: no results to store"
      exit 0
    fi
    set -e
    branch=coverage-results
    message="coverage: nightly run ${GITHUB_RUN_ID:-local}"
    work=$(mktemp -d)
    trap 'git -C "$root" worktree remove --force "$work" 2>/dev/null || true' EXIT
    for attempt in 1 2 3 4 5; do
      git -C "$root" worktree remove --force "$work" 2>/dev/null || rm -rf "$work"
      if git -C "$root" ls-remote --exit-code --heads origin "$branch" >/dev/null; then
        git -C "$root" fetch --quiet --depth=1 origin "+refs/heads/$branch:refs/remotes/origin/$branch"
        git -C "$root" worktree add --quiet --detach "$work" "origin/$branch"
      else
        # First run: an orphan branch that shares no history with main.
        git -C "$root" worktree add --quiet --detach "$work"
        git -C "$work" checkout --quiet --orphan "$branch"
        git -C "$work" rm -rfq --ignore-unmatch .
      fi
      mkdir -p "$work/projects" "$work/history" "$work/badges"
      month=$(date -u +%Y-%m)
      for f in "${files[@]}"; do
        project=$(jq -r '.project' "$f")
        cp "$f" "$work/projects/$(slug "$project").json"
        cat "$f" >>"$work/history/$month.jsonl"
        badge "coverage" "$(jq -r '.lines.percent // "none"' "$f")" >"$work/badges/$(slug "$project").svg"
      done
      # The total covers every project with a stored result, so a project
      # that failed tonight keeps its last number.
      summarize "$work"/projects/*.json >"$work/summary.json"
      badge "coverage" "$(jq -r '.lines.percent // "none"' "$work/summary.json")" >"$work/badge.svg"
      cat >"$work/README.md" <<'MD'
# coverage-results

Data branch written by `nightly-coverage.yml` through
`standards/scripts/coverage.sh store`. Never merge it into `main`.
See `standards/docs/testing.md` on `main`.
MD
      git -C "$work" add -A
      git -C "$work" -c user.name='github-actions[bot]' \
        -c user.email='41898282+github-actions[bot]@users.noreply.github.com' \
        commit --quiet -m "$message"
      if git -C "$work" push --quiet origin HEAD:refs/heads/coverage-results; then
        echo "coverage: pushed \"$message\" to $branch"
        exit 0
      fi
      echo "coverage: push to $branch lost a race (attempt $attempt); retrying"
      sleep $((attempt * 5))
    done
    echo "coverage: could not push to $branch after 5 attempts" >&2
    exit 1
    ;;
  *)
    echo "usage: coverage.sh run | badge <label> <percent|none> | summarize <results...> | store <results...>" >&2
    exit 2
    ;;
esac
