#!/usr/bin/env bash
# Eval engine steps for the shared mise tasks and CI (standards/docs/evals.md).
#
#   evals.sh report <name> <command...>
#       Runs the command, saves its output to target/evals-reports/<name>.log
#       in the current project, and exits with the command's status. The
#       shared `ci` tasks save `cargo test` this way.
#   evals.sh run
#       From a project directory: turns the reports this run saved
#       (target/evals-reports/cargo-test.log and playwright.json) into eval
#       results for the stage in $EVALS_STAGE (default ci), appended to
#       $EVALS_RESULTS (default target/evals-reports/evals-results.jsonl).
#       Does nothing when the top-level project has no evals.toml. This is
#       the shared `evals` task.
#       With EVALS_STAGE=cd (the step after `deploy` in mise-projects.yml)
#       it reads reports from target/evals-reports/cd/ instead (a `smoke`
#       task's), makes the requests of the `http` evals against
#       $DEPLOY_URL, and exits 1 when an eval of a `must` requirement
#       failed.
#   evals.sh publish
#       Writes the job summary for $EVALS_RESULTS ($GITHUB_STEP_SUMMARY, or
#       standard output). Does nothing when the file doesn't exist.
#   evals.sh store <results.jsonl...>
#       Commits the results to the long-lived evals-results data branch:
#       appends them to results/<project>/<yyyy-mm>.jsonl, moves months
#       older than 12 into results/<project>/archive/<yyyy>.jsonl.gz,
#       rewrites verdicts.json, and pushes one commit, retrying when another
#       run pushed first. Creates the branch on its first run. The
#       evals-store.yml workflow runs it with contents: write; it is the only push CI
#       makes, and only to evals-results (SEC-007), never to main.
#   evals.sh verdicts [--project <p>] [--json]
#       Fetches the evals-results branch and prints every requirement's
#       current verdict (evals verdicts).
#   evals.sh triage [--dry-run]
#       Files the work the verdicts call for (evals triage): in a worktree of
#       origin/main, writes a FIX or TODO issue file for each requirement
#       that is not green and has no open issue naming it, commits them, and
#       force-pushes that one commit to the evals-triage branch, which PR
#       triage opens a pull request from. Does nothing while a pull request
#       from evals-triage is open, and deletes a leftover evals-triage
#       branch when there is nothing to file. The evals-triage.yml workflow
#       runs it daily; its pushes go only to evals-triage (SEC-007).
#
# The evals CLI is built from evals/ with cargo the first time it is needed.
set -uo pipefail

root=$(git rev-parse --show-toplevel)
reports_dir="$PWD/target/evals-reports"
results=${EVALS_RESULTS:-$reports_dir/evals-results.jsonl}

evals_cli() {
  cargo run --quiet --manifest-path "$root/evals/Cargo.toml" -p evals_cli -- "$@"
}

case "${1:-}" in
  report)
    name=${2:?usage: evals.sh report <name> <command...>}
    shift 2
    mkdir -p "$reports_dir"
    "$@" 2>&1 | tee "$reports_dir/$name.log"
    exit "${PIPESTATUS[0]}"
    ;;
  run)
    rel=$(realpath --relative-to="$root" "$PWD")
    top=${rel%%/*}
    if [ "$rel" = . ] || [ ! -f "$root/$top/evals.toml" ]; then
      echo "evals: $top has no evals.toml, nothing to collect"
      exit 0
    fi
    stage=${EVALS_STAGE:-ci}
    args=(run --root "$root" --stage "$stage" --project "$top" --out "$results")
    # The ci task's reports are for the build in the runner; cd evals read
    # only what ran against the deployed site.
    if [ "$stage" = cd ]; then
      reports_dir="$reports_dir/cd"
      args+=(--fail-on-must)
    fi
    [ -f "$reports_dir/cargo-test.log" ] && args+=(--cargo-output "$reports_dir/cargo-test.log")
    [ -f "$reports_dir/playwright.json" ] && args+=(--playwright-report "$reports_dir/playwright.json")
    # A sub-project (cookbook/web), or a top-level project that has
    # sub-projects, reports only the evals its own tests cover; the other
    # parts' jobs report the rest.
    if [ "$rel" != "$top" ] || git -C "$root" ls-files -- "$top/*/mise.toml" | grep -q .; then
      args+=(--partial)
    fi
    mkdir -p "$(dirname "$results")"
    evals_cli "${args[@]}"
    ;;
  publish)
    if [ ! -f "$results" ]; then
      echo "evals: no results in $results"
      exit 0
    fi
    evals_cli publish --results "$results"
    ;;
  store)
    shift
    files=()
    for f in "$@"; do
      [ -s "$f" ] && files+=("$(realpath "$f")")
    done
    if [ ${#files[@]} -eq 0 ]; then
      echo "evals: no results to store"
      exit 0
    fi
    set -e
    branch=evals-results
    projects=$(cat "${files[@]}" | jq -rs 'map(.project) | unique | join(" ")')
    stages=$(cat "${files[@]}" | jq -rs 'map(.stage) | unique | join("+")')
    message="evals: ${stages} ${projects} run ${GITHUB_RUN_ID:-local}"
    # Build once, before the retry loop.
    cargo build --quiet --manifest-path "$root/evals/Cargo.toml" -p evals_cli
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
        git -C "$work" rm -rfq .
      fi
      args=(store --root "$root" --dir "$work")
      for f in "${files[@]}"; do args+=(--results "$f"); done
      evals_cli "${args[@]}"
      # Months older than 12 go into a yearly gzip archive (gzip members
      # concatenate, so zcat reads the whole year).
      cutoff=$(date -u -d "$(date -u +%Y-%m-01) -12 months" +%Y-%m)
      for f in "$work"/results/*/[0-9][0-9][0-9][0-9]-[0-9][0-9].jsonl; do
        [ -f "$f" ] || continue
        month=$(basename "$f" .jsonl)
        [[ "$month" < "$cutoff" ]] || continue
        mkdir -p "$(dirname "$f")/archive"
        gzip -9nc "$f" >> "$(dirname "$f")/archive/${month%%-*}.jsonl.gz"
        rm "$f"
      done
      git -C "$work" add -A
      git -C "$work" -c user.name='github-actions[bot]' \
        -c user.email='41898282+github-actions[bot]@users.noreply.github.com' \
        commit --quiet -m "$message"
      if git -C "$work" push --quiet origin HEAD:refs/heads/evals-results; then
        echo "evals: pushed \"$message\" to $branch"
        exit 0
      fi
      echo "evals: push to $branch lost a race (attempt $attempt); retrying"
      sleep $((attempt * 5))
    done
    echo "evals: could not push to $branch after 5 attempts" >&2
    exit 1
    ;;
  verdicts)
    shift
    work=$(mktemp -d)
    trap 'rm -rf "$work"' EXIT
    if git -C "$root" fetch --quiet --depth=1 origin "+refs/heads/evals-results:refs/remotes/origin/evals-results" 2>/dev/null; then
      git -C "$root" archive origin/evals-results | tar -x -C "$work"
    else
      echo "evals: no evals-results branch yet, so no results" >&2
    fi
    evals_cli verdicts --root "$root" --dir "$work" "$@"
    ;;
  triage)
    shift
    set -e
    branch=evals-triage
    summary=${GITHUB_STEP_SUMMARY:-/dev/stdout}
    dry=false
    [ "${1:-}" = --dry-run ] && dry=true
    repo=${GITHUB_REPOSITORY:-jimhillr/monorepo}
    if ! $dry && command -v gh >/dev/null; then
      open=$(gh api "repos/$repo/pulls?state=open&head=${repo%%/*}:$branch" --jq '.[0].number // empty')
      if [ -n "$open" ]; then
        echo "evals: pull request #$open from $branch is still open; filing nothing until it closes" | tee -a "$summary"
        exit 0
      fi
    fi
    results=$(mktemp -d)
    work=$(mktemp -d)
    trap 'rm -rf "$results"; git -C "$root" worktree remove --force "$work" 2>/dev/null || true' EXIT
    if ! git -C "$root" fetch --quiet --depth=1 origin "+refs/heads/evals-results:refs/remotes/origin/evals-results"; then
      echo "evals: no evals-results branch yet, so nothing to triage"
      exit 0
    fi
    git -C "$root" archive origin/evals-results | tar -x -C "$results"
    git -C "$root" fetch --quiet origin "+refs/heads/main:refs/remotes/origin/main"
    rmdir "$work"
    git -C "$root" worktree add --quiet --detach "$work" origin/main
    cargo build --quiet --manifest-path "$root/evals/Cargo.toml" -p evals_cli
    args=(triage --root "$work" --dir "$results")
    $dry || args+=(--write)
    report=$(evals_cli "${args[@]}")
    mapfile -t files < <(git -C "$work" status --porcelain --untracked-files=all | sed 's/^?? //')
    {
      echo "## Eval triage"
      echo
      if $dry; then
        echo "Dry run: nothing written."
      elif [ ${#files[@]} -eq 0 ]; then
        echo "Nothing new to file: every requirement is green, already has an open issue, or waits on a human check."
      else
        echo "New issue files:"
        echo
        printf -- '- `%s`\n' "${files[@]}"
      fi
      echo
      echo '```text'
      echo "$report"
      echo '```'
    } >>"$summary"
    if $dry; then
      exit 0
    fi
    if [ ${#files[@]} -eq 0 ]; then
      if git -C "$root" ls-remote --exit-code --heads origin "$branch" >/dev/null; then
        git -C "$work" push --quiet origin --delete refs/heads/evals-triage
        echo "evals: nothing to file; deleted the leftover $branch branch"
      fi
      exit 0
    fi
    ids=$(for f in "${files[@]}"; do basename "$f" .md; done \
      | sed -E 's/^eval-([a-z]+-[0-9]+)-.*/\1/' | tr a-z A-Z | sort -u | paste -sd, - | sed 's/,/, /g')
    git -C "$work" add -A
    git -C "$work" -c user.name='github-actions[bot]' \
      -c user.email='41898282+github-actions[bot]@users.noreply.github.com' \
      commit --quiet -F - <<EOF2
evals: triage files issues for $ids

Eval triage (evals-triage.yml run ${GITHUB_RUN_ID:-local}) found requirements
that are not green and have no open issue, and filed one issue file each.
Agents work them like any other issue. Only issue files change.

$report
EOF2
    git -C "$work" push --quiet --force origin HEAD:refs/heads/evals-triage
    echo "evals: pushed the triage commit to $branch for PR triage to open"
    ;;
  *)
    echo "usage: evals.sh report <name> <command...> | run | publish | store <results...> | verdicts | triage [--dry-run]" >&2
    exit 2
    ;;
esac
