#!/usr/bin/env bash
# Tests the `e2e` task in standards/tasks/dioxus-web.toml: where it runs the
# Playwright tests, where it skips them with a note, and where it fails.
# npm, npx and dx are stubs, so this needs only mise. Run from anywhere:
#   bash standards/tests/e2e-task.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
mise_dir=$(dirname "$(command -v mise)")
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# A project that includes the shared task file, with stub tools that log
# their calls and fail when FAIL_<TOOL> names their first argument.
mkdir -p "$work/project" "$work/stubs" "$work/stubs-no-dx"
cat >"$work/project/mise.toml" <<TOML
[task_config]
includes = ["$root/standards/tasks/dioxus-web.toml"]
TOML
for tool in npm npx dx; do
  cat >"$work/stubs/$tool" <<SH
#!/bin/sh
echo "$tool \$*" >>"$work/calls"
case " \${FAIL_$tool:-} " in *" \$1 "*) exit 1 ;; esac
exit 0
SH
  chmod +x "$work/stubs/$tool"
  [ "$tool" = dx ] || cp "$work/stubs/$tool" "$work/stubs-no-dx/"
done

failures=0

# run_case NAME EXPECTED_STATUS STUB_DIR [VAR=value ...]
# Runs `mise run e2e` and leaves its output in $out and the tool calls in
# $calls.
run_case() {
  local name=$1 want=$2 stubs=$3
  shift 3
  : >"$work/calls"
  out=$(cd "$work/project" && env -u CI -u FAIL_npm -u FAIL_npx -u FAIL_dx \
    PATH="$stubs:$mise_dir:/usr/bin:/bin" \
    MISE_TRUSTED_CONFIG_PATHS="$work/project" MISE_YES=1 "$@" \
    mise run --quiet e2e 2>&1)
  local got=$?
  calls=$(cat "$work/calls")
  if [ "$got" -ne "$want" ]; then
    echo "FAIL $name: exit $got, want $want"
    echo "$out" | sed 's/^/    /'
    failures=$((failures + 1))
    return 1
  fi
}

# expect NAME TEXT HAYSTACK: fail unless HAYSTACK contains TEXT.
expect() {
  case "$3" in
    *"$2"*) ;;
    *) echo "FAIL $1: missing '$2' in:"; echo "$3" | sed 's/^/    /'; failures=$((failures + 1)) ;;
  esac
}
refuse() {
  case "$3" in
    *"$2"*) echo "FAIL $1: unexpected '$2' in:"; echo "$3" | sed 's/^/    /'; failures=$((failures + 1)) ;;
  esac
}

# No package.json: nothing to run.
run_case no-package 0 "$work/stubs" && refuse no-package "npm" "$calls"

touch "$work/project/package.json"

# Everything installs: the tests run.
run_case all-tools 0 "$work/stubs" && expect all-tools "npm test" "$calls" \
  && expect all-tools "--reporter=list,json" "$calls"

# A failing test fails the task, in CI or not.
run_case test-fails 1 "$work/stubs" FAIL_npm=test
run_case test-fails-ci 1 "$work/stubs" CI=true FAIL_npm=test

# Outside CI, a tool that can't be installed skips the tests with a note.
run_case no-dx 0 "$work/stubs-no-dx" \
  && expect no-dx "dx (dioxus-cli) is not installed" "$out" \
  && expect no-dx "PR CI runs them" "$out" \
  && refuse no-dx "npm test" "$calls"
run_case no-browser 0 "$work/stubs" FAIL_npx=playwright \
  && expect no-browser "couldn't install Playwright's browsers" "$out" \
  && refuse no-browser "npm test" "$calls"
run_case no-npm-packages 0 "$work/stubs" FAIL_npm=ci \
  && expect no-npm-packages "couldn't install the npm packages" "$out"

# In CI the same gaps fail the task.
run_case no-dx-ci 1 "$work/stubs-no-dx" CI=true
run_case no-browser-ci 1 "$work/stubs" CI=true FAIL_npx=playwright

if [ "$failures" -gt 0 ]; then
  echo "e2e-task: $failures failure(s)"
  exit 1
fi
echo "e2e-task: all cases passed"
