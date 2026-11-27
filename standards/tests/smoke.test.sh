#!/usr/bin/env bash
# Tests standards/scripts/smoke.sh: when it runs the @smoke Playwright tests
# against DEPLOY_URL, when it skips with a note, and when it fails. npm and
# npx are stubs, so this needs only bash. Run from anywhere:
#   bash standards/tests/smoke.test.sh
set -uo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# A deploying project next to a web app, and stub tools that log their calls
# (with BASE_URL and PLAYWRIGHT_JSON_OUTPUT_FILE) and fail when FAIL_<tool>
# is part of their arguments.
mkdir -p "$work/cloudflare" "$work/web" "$work/stubs"
for tool in npm npx; do
  cat >"$work/stubs/$tool" <<SH
#!/bin/sh
echo "$tool \$* BASE_URL=\${BASE_URL:-} JSON=\${PLAYWRIGHT_JSON_OUTPUT_FILE:-} in \$(basename "\$PWD")" >>"$work/calls"
if [ -n "\${FAIL_$tool:-}" ]; then
  case "\$*" in *"\${FAIL_$tool}"*) exit 1 ;; esac
fi
exit 0
SH
  chmod +x "$work/stubs/$tool"
done

failures=0

# run_case NAME EXPECTED_STATUS [VAR=value ...]
# Runs smoke.sh from the deploying project and leaves its output in $out and
# the tool calls in $calls.
run_case() {
  local name=$1 want=$2
  shift 2
  : >"$work/calls"
  out=$(cd "$work/cloudflare" && env -u CI -u SMOKE_DIR -u DEPLOY_URL -u SMOKE_PROJECT \
    -u FAIL_npm -u FAIL_npx PATH="$work/stubs:/usr/bin:/bin" "$@" \
    bash "$root/standards/scripts/smoke.sh" 2>&1)
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

url=https://jimhillr-example.pages.dev

# No SMOKE_DIR: no smoke tests yet, nothing runs.
run_case no-smoke-dir 0 DEPLOY_URL=$url \
  && expect no-smoke-dir "no smoke tests yet" "$out" && refuse no-smoke-dir "npm" "$calls"

# SMOKE_DIR without DEPLOY_URL, or without a package.json, is a mistake.
run_case no-url 2 SMOKE_DIR=../web && expect no-url "DEPLOY_URL is not set" "$out"
run_case no-package 2 SMOKE_DIR=../web DEPLOY_URL=$url && expect no-package "no package.json" "$out"

touch "$work/web/package.json"

# Everything installs: the @smoke tests run in the web app against the URL,
# with the report in the deploying project's target/evals-reports/cd.
run_case runs 0 SMOKE_DIR=../web DEPLOY_URL=$url \
  && expect runs "npx playwright install --with-deps chromium" "$calls" \
  && expect runs "npx playwright test --grep @smoke --project chromium" "$calls" \
  && expect runs "BASE_URL=$url" "$calls" \
  && expect runs "JSON=$work/cloudflare/target/evals-reports/cd/playwright.json in web" "$calls" \
  && expect runs "--output $work/cloudflare/target/evals-reports/cd/test-results" "$calls"
[ -d "$work/cloudflare/target/evals-reports/cd" ] \
  || { echo "FAIL runs: no target/evals-reports/cd"; failures=$((failures + 1)); }

# SMOKE_PROJECT picks another Playwright project.
run_case project 0 SMOKE_DIR=../web DEPLOY_URL=$url SMOKE_PROJECT="Mobile Chrome" \
  && expect project "--project Mobile Chrome" "$calls"

# A failing test fails the run, in CI or not.
run_case test-fails 1 SMOKE_DIR=../web DEPLOY_URL=$url FAIL_npx="test --grep"
run_case test-fails-ci 1 SMOKE_DIR=../web DEPLOY_URL=$url CI=true FAIL_npx="test --grep"

# Outside CI, a tool that can't be installed skips with a note.
run_case no-browser 0 SMOKE_DIR=../web DEPLOY_URL=$url FAIL_npx=install \
  && expect no-browser "couldn't install Playwright's Chromium" "$out" \
  && refuse no-browser "--grep @smoke" "$calls"
run_case no-npm-packages 0 SMOKE_DIR=../web DEPLOY_URL=$url FAIL_npm=ci \
  && expect no-npm-packages "couldn't install the npm packages" "$out"

# In CI the same gaps fail.
run_case no-browser-ci 1 SMOKE_DIR=../web DEPLOY_URL=$url CI=true FAIL_npx=install

if [ "$failures" -gt 0 ]; then
  echo "smoke: $failures failure(s)"
  exit 1
fi
echo "smoke: all cases passed"
