#!/usr/bin/env bash
# SEC-005: zizmor over GitHub Actions workflows (standards/docs/security.md).
#
#   workflows-check.sh --files   the workflows among the changed files read
#                                from stdin (one path per line); every
#                                workflow when .github/zizmor.yml changed
#   workflows-check.sh --all     every workflow
#
# zizmor runs offline with its default rules and our .github/zizmor.yml,
# reporting medium and high findings: pull_request_target or workflow_run
# running PR code, template injection into run:, secrets reaching PR code,
# write-all permissions, credentials persisted by actions/checkout, and
# third-party actions not pinned to a commit. A change to anything else
# costs nothing.
#
# Warnings until phase 4 of the threat check plan: findings print as
# warning[SEC-005] and the script always exits 0. zizmor comes from PATH, or
# else `mise exec zizmor@<version>`; when neither works it says so and skips.
# In GitHub Actions, findings are also annotations on the workflow lines.
set -uo pipefail

# Pinned version. Bump it here only.
ZIZMOR_VERSION=1.33.1

mode=${1:?usage: workflows-check.sh --files|--all}
root=$(git rev-parse --show-toplevel)
cd "$root"

say() { printf 'workflows-check: %s\n' "$*" >&2; }

all_workflows() {
  git ls-files -- '.github/workflows/*.yml' '.github/workflows/*.yaml'
}

case "$mode" in
  --all)
    files=$(all_workflows)
    ;;
  --files)
    files=""
    while IFS= read -r f; do
      case "$f" in
        .github/zizmor.yml)
          files+=$(all_workflows)$'\n'
          ;;
        .github/workflows/*.yml | .github/workflows/*.yaml)
          [ -f "$f" ] && files+="$f"$'\n'
          ;;
      esac
    done
    ;;
  *)
    say "unknown mode '$mode' (want --files or --all)"
    exit 2
    ;;
esac

files=$(printf '%s' "$files" | sed '/^$/d' | sort -u)
if [ -z "$files" ]; then
  exit 0
fi

# Run it, not `command -v`: a mise shim with no version set can't run.
if zizmor --version >/dev/null 2>&1; then
  cmd=(zizmor)
elif command -v mise >/dev/null 2>&1 \
  && mise exec "zizmor@$ZIZMOR_VERSION" -- zizmor --version >/dev/null 2>&1; then
  cmd=(mise exec "zizmor@$ZIZMOR_VERSION" -- zizmor)
else
  say "zizmor is not installed and mise couldn't download it, so SEC-005 was"
  say "not checked here; PR CI checks it. Install it with"
  say "'cargo install zizmor --version $ZIZMOR_VERSION --locked'."
  exit 0
fi

mapfile -t list <<<"$files"
say "zizmor on ${#list[@]} workflow(s) (SEC-005)"
format=plain
[ "${GITHUB_ACTIONS:-}" = true ] && format=github
# zizmor exits 0 when clean, 10 and up when it finds something, and
# anything else when it couldn't run.
"${cmd[@]}" --offline --no-progress --min-severity medium --format "$format" \
  "${list[@]}"
status=$?
case "$status" in
  0) say "no findings" ;;
  1[0-9])
    echo "warning[SEC-005] zizmor found problems in the workflows (above). Fix them (security.md, \"Workflows\"), or add an ignore with a reason to .github/zizmor.yml."
    ;;
  *) say "zizmor could not run (exit $status); see the output above" ;;
esac
exit 0
