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
  *)
    echo "usage: evals.sh report <name> <command...> | run | publish" >&2
    exit 2
    ;;
esac
