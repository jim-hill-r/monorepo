#!/usr/bin/env bash
# Runs a deploying project's `@smoke` Playwright tests against its live site
# (stage cd, standards/docs/evals.md). Run from the deploying project's
# directory, such as cookbook/cloudflare:
#
#   DEPLOY_URL=https://jimhillr-cookbook.pages.dev bash ../../standards/scripts/smoke.sh
#
# The project names the Playwright project that holds its tests with
# SMOKE_DIR in its mise.toml [env], relative to itself (cookbook/cloudflare
# sets "../web"). That project's playwright.config.ts must read BASE_URL:
# when it is set, run against it with no dev server. SMOKE_PROJECT picks the
# Playwright project to run (default chromium); only Chromium is installed,
# so it must be a Chromium one. One browser keeps the run to a minute or two:
# the other browsers already ran every test in PR CI.
#
# Writes the JSON report to target/evals-reports/cd/playwright.json in the
# deploying project, where `evals.sh run` with EVALS_STAGE=cd reads it, and
# each test's video, screenshot and trace to
# target/evals-reports/cd/test-results/.
#
# Exits 0 with a note when the project has no SMOKE_DIR (no smoke tests
# yet), 2 when DEPLOY_URL is missing, and otherwise with Playwright's status.
# Outside CI (CI unset), a tool that can't be installed, such as Playwright's
# browsers in a cloud sandbox, skips the run with a note instead.
set -uo pipefail

out="$PWD/target/evals-reports/cd"

if [ -z "${SMOKE_DIR:-}" ]; then
  echo "smoke: no smoke tests yet (set SMOKE_DIR in this project's mise.toml [env])"
  exit 0
fi
if [ -z "${DEPLOY_URL:-}" ]; then
  echo "smoke: DEPLOY_URL is not set" >&2
  exit 2
fi
if [ ! -f "$SMOKE_DIR/package.json" ]; then
  echo "smoke: SMOKE_DIR $SMOKE_DIR has no package.json" >&2
  exit 2
fi

skip() {
  echo "smoke: $1" >&2
  if [ -n "${CI:-}" ]; then exit 1; fi
  echo "smoke: smoke tests not run here; the deploy job runs them" >&2
  exit 0
}

mkdir -p "$out"
rm -rf "$out/playwright.json" "$out/test-results"
cd "$SMOKE_DIR" || exit 2
npm ci || skip "couldn't install the npm packages"
project=${SMOKE_PROJECT:-chromium}
npx playwright install --with-deps chromium || skip "couldn't install Playwright's Chromium"

echo "smoke: running the @smoke tests in $SMOKE_DIR ($project) against $DEPLOY_URL"
BASE_URL="$DEPLOY_URL" PLAYWRIGHT_JSON_OUTPUT_FILE="$out/playwright.json" \
  npx playwright test --grep @smoke --project "$project" \
  --output "$out/test-results" --reporter=list,json
