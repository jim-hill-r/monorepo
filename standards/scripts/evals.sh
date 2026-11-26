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
#   evals.sh publish
#       Writes the job summary for $EVALS_RESULTS ($GITHUB_STEP_SUMMARY, or
#       standard output). Does nothing when the file doesn't exist.
#   evals.sh store <results.jsonl...>
#       Commits the results to the long-lived evals-results data branch:
#       appends them to results/<project>/<yyyy-mm>.jsonl, moves months
#       older than 12 into results/<project>/archive/<yyyy>.jsonl.gz,
#       rewrites verdicts.json, and pushes one commit, retrying when another
#       run pushed first. Creates the branch on its first run. CI's
#       store-evals job runs it with contents: write; it is the only push CI
#       makes, and only to evals-results (SEC-007), never to main.
#   evals.sh verdicts [--project <p>] [--json]
#       Fetches the evals-results branch and prints every requirement's
#       current verdict (evals verdicts).
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
    args=(run --root "$root" --stage "${EVALS_STAGE:-ci}" --project "$top" --out "$results")
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
    message="evals: ${EVALS_STAGE:-ci} ${projects} run ${GITHUB_RUN_ID:-local}"
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
  *)
    echo "usage: evals.sh report <name> <command...> | run | publish | store <results...> | verdicts" >&2
    exit 2
    ;;
esac
